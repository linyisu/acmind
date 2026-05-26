//! Local HTTP server that receives scraped data from the ACMind browser extension.
//! Listens on 127.0.0.1:18921 (loopback only, no external exposure).

use crate::db::models::{CreateProblemInput, CreateSubmissionInput};
use crate::db::repo;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::thread;
use tiny_http::{Header, Method, Request, Response, StatusCode};

const BIND_ADDR: &str = "127.0.0.1:18921";

#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    imported: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_problems: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_synced: Option<usize>,
}

// ---- Request payloads (matching extension output) ----

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ImportSubmissionsPayload {
    username: String,
    items: Vec<SubmissionItem>,
    #[serde(default)]
    include_source: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmissionItem {
    #[serde(alias = "runId")]
    run_id: i64,
    oj: String,
    #[serde(alias = "probNum")]
    prob_num: String,
    status: String,
    language: String,
    #[serde(default)]
    runtime: Option<i32>,
    #[serde(default)]
    memory: Option<i32>,
    time: i64,
    #[serde(default)]
    #[serde(alias = "sourceProblemId")]
    source_problem_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportProblemPayload {
    #[serde(alias = "sourceProblemId")]
    source_problem_id: String,
    oj: String,
    #[serde(alias = "probNum")]
    prob_num: String,
    title: String,
    url: Option<String>,
    statement: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportSubmissionPayload {
    #[serde(alias = "runId")]
    run_id: i64,
    oj: String,
    #[serde(alias = "probNum")]
    prob_num: String,
    status: String,
    language: String,
    code: Option<String>,
    runtime: Option<i32>,
    memory: Option<i32>,
    #[serde(alias = "submitTime")]
    submit_time: Option<i64>,
}

// ---- Server state ----

struct ServerState {
    pool: SqlitePool,
    storage: Storage,
}

/// Start the import HTTP server on a background thread.
/// Returns a handle that signals shutdown when dropped.
pub fn start_import_server(pool: SqlitePool, storage: Storage) -> ImportServerHandle {
    let state = Arc::new(ServerState { pool, storage });
    let server = tiny_http::Server::http(BIND_ADDR).expect("Failed to start ACMind import server");

    tracing::info!(target: "app_lib::import_server", "Import server listening on {}", BIND_ADDR);

    let handle = ImportServerHandle::new();

    thread::spawn(move || {
        for request in server.incoming_requests() {
            let state = Arc::clone(&state);
            thread::spawn(move || handle_request(state, request));
        }
    });

    handle
}

pub struct ImportServerHandle {
    // Keep alive marker — future: add graceful shutdown channel
}

impl ImportServerHandle {
    fn new() -> Self {
        Self {}
    }
}

// ---- Request routing ----

fn handle_request(state: Arc<ServerState>, mut request: Request) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or(&url);

    // Always add CORS headers
    let cors_headers = vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Allow-Methods"[..],
            &b"GET, POST, OPTIONS"[..],
        )
        .unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap(),
    ];

    // Handle CORS preflight
    if method == Method::Options {
        let resp = Response::new(StatusCode(204), cors_headers, &[] as &[u8], None, None);
        let _ = request.respond(resp);
        return;
    }

    // Read body
    let mut body = String::new();
    {
        let reader = request.as_reader();
        let _ = reader.read_to_string(&mut body);
    }

    let result = match (&method, path) {
        (&Method::Get, "/health") => Ok(health_response()),
        (&Method::Post, "/vjudge/import/submissions") => handle_import_submissions(&state, &body),
        (&Method::Post, "/vjudge/import/problem") => handle_import_problem(&state, &body),
        (&Method::Post, "/vjudge/import/submission") => handle_import_submission(&state, &body),
        _ => Ok(json_response(
            StatusCode(404),
            &ApiResponse {
                success: false,
                message: Some("Not found".into()),
                imported: None,
                skipped: None,
                created_problems: None,
                source_synced: None,
            },
        )),
    };

    let resp = match result {
        Ok(response) => response,
        Err(err) => json_response(
            StatusCode(500),
            &ApiResponse {
                success: false,
                message: Some(err),
                imported: None,
                skipped: None,
                created_problems: None,
                source_synced: None,
            },
        ),
    };

    let _ = request.respond(resp);
}

// ---- Handlers ----

fn health_response() -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(
        StatusCode(200),
        &serde_json::json!({"status": "ok", "app": "acmind"}),
    )
}

fn handle_import_submissions(
    state: &ServerState,
    body: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, String> {
    let payload: ImportSubmissionsPayload =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;

    if payload.items.is_empty() {
        return Ok(json_response(
            StatusCode(200),
            &ApiResponse {
                success: true,
                message: Some("No submissions to import".into()),
                imported: Some(0),
                skipped: Some(0),
                created_problems: Some(0),
                source_synced: Some(0),
            },
        ));
    }

    tracing::info!(target: "app_lib::import_server",
        "Importing {} submissions for user {} from browser extension",
        payload.items.len(),
        payload.username
    );

    // Save username setting
    if let Err(e) = futures::executor::block_on(repo::set_setting(
        &state.pool,
        "vjudge_username",
        &payload.username,
    )) {
        tracing::warn!(target: "app_lib::import_server", "Failed to save username: {}", e);
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut created_problems = 0usize;

    for item in &payload.items {
        match import_single_submission(state, item) {
            Ok(result) => {
                if result.created {
                    imported += 1;
                } else {
                    skipped += 1;
                }
                if result.problem_created {
                    created_problems += 1;
                }
            }
            Err(e) => {
                tracing::warn!(target: "app_lib::import_server",
                    "Failed to import submission run#{}: {}",
                    item.run_id, e
                );
                skipped += 1;
            }
        }
    }

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(format!(
                "Imported {} submissions ({} new, {} skipped, {} new problems)",
                payload.items.len(),
                imported,
                skipped,
                created_problems
            )),
            imported: Some(imported),
            skipped: Some(skipped),
            created_problems: Some(created_problems),
            source_synced: None,
        },
    ))
}

struct ImportResult {
    created: bool,
    problem_created: bool,
}

fn import_single_submission(
    state: &ServerState,
    item: &SubmissionItem,
) -> Result<ImportResult, String> {
    let external_run_id = format!("vjudge:{}", item.run_id);

    // Check if already exists
    match futures::executor::block_on(repo::submission_by_external_run_id(
        &state.pool,
        &external_run_id,
    )) {
        Ok(Some(_)) => {
            return Ok(ImportResult {
                created: false,
                problem_created: false,
            })
        }
        Ok(None) => {}
        Err(e) => return Err(format!("DB error: {}", e)),
    }

    // Find or create problem
    let source_problem_id = if item.source_problem_id.is_empty() {
        format!("{}-{}", item.oj, item.prob_num)
    } else {
        item.source_problem_id.clone()
    };

    let (problem_id, problem_created) = match futures::executor::block_on(
        repo::find_problem_by_source_id(&state.pool, "VJudge", &source_problem_id),
    ) {
        Ok(Some(problem)) => (problem.id, false),
        Ok(None) => {
            let created = futures::executor::block_on(repo::create_problem(
                &state.pool,
                &CreateProblemInput {
                    source: "VJudge".into(),
                    source_problem_id: source_problem_id.clone(),
                    title: source_problem_id.clone(),
                    url: Some(format!("https://vjudge.net/problem/{}", source_problem_id)),
                    difficulty: None,
                    tags: vec![item.oj.clone()],
                    statement: None,
                },
                None,
            ))
            .map_err(|e| format!("Failed to create problem: {}", e))?;
            (created.id, true)
        }
        Err(e) => return Err(format!("DB error: {}", e)),
    };

    let status = normalize_status(&item.status);
    let submitted_at = crate::vjudge::timestamp_millis(item.time)
        .ok()
        .unwrap_or_else(chrono::Utc::now);

    let code_path = String::new(); // source code fetched separately

    let _sub = futures::executor::block_on(repo::create_submission(
        &state.pool,
        &CreateSubmissionInput {
            problem_id: problem_id.clone(),
            status: status.into(),
            language: item.language.clone(),
            code_text: String::new(),
            runtime: item.runtime,
            memory: item.memory,
            note: Some(format!(
                "VJudge run #{}，原始状态：{}",
                item.run_id, item.status
            )),
            external_run_id: Some(external_run_id),
            submitted_at: Some(submitted_at),
        },
        &code_path,
    ))
    .map_err(|e| format!("Failed to create submission: {}", e))?;

    Ok(ImportResult {
        created: true,
        problem_created,
    })
}

fn handle_import_problem(
    state: &ServerState,
    body: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, String> {
    let payload: ImportProblemPayload =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;

    let source_problem_id = if payload.source_problem_id.is_empty() {
        format!("{}-{}", payload.oj, payload.prob_num)
    } else {
        payload.source_problem_id.clone()
    };

    tracing::info!(target: "app_lib::import_server",
        "Importing problem {} from browser extension",
        source_problem_id
    );

    // Check if already exists
    let existing = futures::executor::block_on(repo::find_problem_by_source_id(
        &state.pool,
        "VJudge",
        &source_problem_id,
    ))
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(ref problem) = existing {
        // Update if we have a statement
        if let Some(ref statement) = payload.statement {
            let path = state
                .storage
                .save_statement(&problem.id, statement)
                .map_err(|e| format!("Failed to save statement: {}", e))?;

            futures::executor::block_on(repo::update_problem(
                &state.pool,
                &problem.id,
                &crate::db::models::UpdateProblemInput {
                    source: None,
                    source_problem_id: None,
                    title: Some(payload.title.clone()),
                    url: payload.url.clone(),
                    difficulty: None,
                    tags: Some(payload.tags.clone()),
                    statement: None,
                },
                Some(&path),
            ))
            .map_err(|e| format!("Failed to update problem: {}", e))?;
        }

        return Ok(json_response(
            StatusCode(200),
            &ApiResponse {
                success: true,
                message: Some(format!(
                    "Problem {} already exists, updated",
                    source_problem_id
                )),
                imported: Some(0),
                skipped: None,
                created_problems: Some(if existing.is_some() { 0 } else { 1 }),
                source_synced: None,
            },
        ));
    }

    // Save statement if provided
    let statement_path: Option<String> = None;

    let problem = futures::executor::block_on(repo::create_problem(
        &state.pool,
        &CreateProblemInput {
            source: "VJudge".into(),
            source_problem_id: source_problem_id.clone(),
            title: payload.title.clone(),
            url: payload.url.clone(),
            difficulty: None,
            tags: payload.tags.clone(),
            statement: None,
        },
        statement_path.as_deref(),
    ))
    .map_err(|e| format!("Failed to create problem: {}", e))?;

    // Save statement if provided
    if let Some(ref statement) = payload.statement {
        if let Ok(path) = state.storage.save_statement(&problem.id, statement) {
            let _ = futures::executor::block_on(repo::update_problem(
                &state.pool,
                &problem.id,
                &crate::db::models::UpdateProblemInput {
                    source: None,
                    source_problem_id: None,
                    title: None,
                    url: None,
                    difficulty: None,
                    tags: None,
                    statement: None,
                },
                Some(&path),
            ));
        }
    }

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(format!("Problem {} imported", source_problem_id)),
            imported: Some(1),
            skipped: None,
            created_problems: Some(1),
            source_synced: None,
        },
    ))
}

fn handle_import_submission(
    state: &ServerState,
    body: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, String> {
    let payload: ImportSubmissionPayload =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;

    let external_run_id = format!("vjudge:{}", payload.run_id);

    tracing::info!(target: "app_lib::import_server",
        "Importing single submission run#{} with source code from browser extension",
        payload.run_id
    );

    // Check if already exists
    let existing = futures::executor::block_on(repo::submission_by_external_run_id(
        &state.pool,
        &external_run_id,
    ))
    .map_err(|e| format!("DB error: {}", e))?;

    let source_problem_id = format!("{}-{}", payload.oj, payload.prob_num);

    // Find or create problem
    let problem_id = match futures::executor::block_on(repo::find_problem_by_source_id(
        &state.pool,
        "VJudge",
        &source_problem_id,
    )) {
        Ok(Some(problem)) => problem.id,
        Ok(None) => {
            let created = futures::executor::block_on(repo::create_problem(
                &state.pool,
                &CreateProblemInput {
                    source: "VJudge".into(),
                    source_problem_id: source_problem_id.clone(),
                    title: source_problem_id.clone(),
                    url: Some(format!("https://vjudge.net/problem/{}", source_problem_id)),
                    difficulty: None,
                    tags: vec![payload.oj.clone()],
                    statement: None,
                },
                None,
            ))
            .map_err(|e| format!("Failed to create problem: {}", e))?;
            created.id
        }
        Err(e) => return Err(format!("DB error: {}", e)),
    };

    let status = normalize_status(&payload.status);
    let submitted_at = payload
        .submit_time
        .and_then(|t| crate::vjudge::timestamp_millis(t).ok())
        .unwrap_or_else(chrono::Utc::now);

    // Save source code if provided
    let code = payload.code.unwrap_or_default();
    let code_path = if !code.is_empty() {
        let temp_id = uuid::Uuid::new_v4().to_string();
        state
            .storage
            .save_submission(&problem_id, &temp_id, status, &payload.language, &code)
            .unwrap_or_default()
    } else {
        String::new()
    };

    if let Some(ref existing_sub) = existing {
        // Update existing with source code
        if !code_path.is_empty() && existing_sub.code_path.is_empty() {
            let _ = futures::executor::block_on(repo::set_submission_code_path(
                &state.pool,
                &existing_sub.id,
                &code_path,
            ));
        }
        return Ok(json_response(
            StatusCode(200),
            &ApiResponse {
                success: true,
                message: Some(format!(
                    "Submission #{} already exists, source updated",
                    payload.run_id
                )),
                imported: Some(0),
                skipped: None,
                created_problems: None,
                source_synced: Some(if !code_path.is_empty() { 1 } else { 0 }),
            },
        ));
    }

    let _sub = futures::executor::block_on(repo::create_submission(
        &state.pool,
        &CreateSubmissionInput {
            problem_id: problem_id.clone(),
            status: status.into(),
            language: payload.language.clone(),
            code_text: code,
            runtime: payload.runtime,
            memory: payload.memory,
            note: Some(format!("VJudge run #{}，从浏览器扩展导入", payload.run_id)),
            external_run_id: Some(external_run_id),
            submitted_at: Some(submitted_at),
        },
        &code_path,
    ))
    .map_err(|e| format!("Failed to create submission: {}", e))?;

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(format!(
                "Submission #{} imported with source code",
                payload.run_id
            )),
            imported: Some(1),
            skipped: None,
            created_problems: None,
            source_synced: Some(if !code_path.is_empty() { 1 } else { 0 }),
        },
    ))
}

// ---- JSON response helper ----

fn json_response<T: Serialize>(status: StatusCode, data: &T) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_vec(data).unwrap_or_else(|_| b"{}".to_vec());
    let len = body.len();

    let cors_headers = vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Allow-Methods"[..],
            &b"GET, POST, OPTIONS"[..],
        )
        .unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap(),
    ];

    Response::new(
        status,
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            Header::from_bytes(&b"Content-Length"[..], len.to_string().as_bytes()).unwrap(),
        ]
        .into_iter()
        .chain(cors_headers)
        .collect::<Vec<_>>(),
        std::io::Cursor::new(body),
        None,
        None,
    )
}

fn normalize_status(status: &str) -> &str {
    match status {
        "Accepted" => "AC",
        "Wrong answer" | "Wrong Answer" => "WA",
        "Time limit exceeded" | "Time Limit Exceeded" | "Time Limit Exceed" => "TLE",
        "Runtime error" | "Runtime Error" => "RE",
        "Memory limit exceeded" | "Memory Limit Exceeded" | "Memory Limit Exceed" => "MLE",
        "Compile error" | "Compilation error" | "Compile Error" => "CE",
        _ => "WA",
    }
}
