//! Local HTTP server that receives scraped data from the ACMind browser extension.
//! Listens on 127.0.0.1:18921 (loopback only, no external exposure).
//! Uses the tokio runtime handle captured at startup for all async DB operations.

use crate::db::models::{CreateProblemInput, CreateSubmissionInput};
use crate::db::repo;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::thread;
use tauri::Emitter;
use tiny_http::{Header, Method, Request, Response, StatusCode};
use tokio::runtime::Handle;

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
    rt: Handle,
    app_handle: tauri::AppHandle,
}

impl ServerState {
    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }

    fn notify_imported(&self, action: &str, detail: &str) {
        let payload = serde_json::json!({
            "action": action,
            "detail": detail,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });
        if let Err(e) = self.app_handle.emit("vjudge-imported", &payload) {
            tracing::warn!(target: "app_lib::import_server", "Failed to emit vjudge-imported event: {}", e);
        }
    }
}

/// Start the import HTTP server on a background thread.
/// Creates its own Tokio runtime for async DB operations.
pub fn start_import_server(
    pool: SqlitePool,
    storage: Storage,
    app_handle: tauri::AppHandle,
) -> ImportServerHandle {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime for import server");
    let handle = rt.handle().clone();
    let state = Arc::new(ServerState {
        pool,
        storage,
        rt: handle,
        app_handle,
    });
    let server = tiny_http::Server::http(BIND_ADDR).expect("Failed to start ACMind import server");

    tracing::info!(target: "app_lib::import_server", "Import server listening on {}", BIND_ADDR);

    let import_handle = ImportServerHandle::new();

    thread::spawn(move || {
        // Enter the runtime so Handle::block_on works on nested threads too
        let _guard = rt.enter();
        for request in server.incoming_requests() {
            let state = Arc::clone(&state);
            thread::spawn(move || handle_request(state, request));
        }
    });

    import_handle
}

pub struct ImportServerHandle {}

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

    let cors_headers = vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Allow-Methods"[..],
            &b"GET, POST, OPTIONS"[..],
        )
        .unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap(),
    ];

    if method == Method::Options {
        let resp = Response::new(StatusCode(204), cors_headers, &[] as &[u8], None, None);
        let _ = request.respond(resp);
        return;
    }

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

    if let Err(e) = state.block_on(repo::set_setting(
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

    state.notify_imported(
        "submissions",
        &format!(
            "{} imported ({} new, {} skipped)",
            payload.items.len(),
            imported,
            skipped
        ),
    );

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

    match state.block_on(repo::submission_by_external_run_id(
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

    let source_problem_id = if item.source_problem_id.is_empty() {
        format!("{}-{}", item.oj, item.prob_num)
    } else {
        item.source_problem_id.clone()
    };

    let (problem_id, problem_created) = match state.block_on(repo::find_problem_by_source_id(
        &state.pool,
        "VJudge",
        &source_problem_id,
    )) {
        Ok(Some(problem)) => (problem.id, false),
        Ok(None) => {
            let created = state
                .block_on(repo::create_problem(
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

    state
        .block_on(repo::create_submission(
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
            "",
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

    let existing = state
        .block_on(repo::find_problem_by_source_id(
            &state.pool,
            "VJudge",
            &source_problem_id,
        ))
        .map_err(|e| format!("DB error: {}", e))?;

    if let Some(ref problem) = existing {
        if let Some(ref statement) = payload.statement {
            let path = state
                .storage
                .save_statement(&problem.id, statement)
                .map_err(|e| format!("Failed to save statement: {}", e))?;

            state
                .block_on(repo::update_problem(
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
                created_problems: Some(0),
                source_synced: None,
            },
        ));
    }

    let problem = state
        .block_on(repo::create_problem(
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
            None,
        ))
        .map_err(|e| format!("Failed to create problem: {}", e))?;

    if let Some(ref statement) = payload.statement {
        if let Ok(path) = state.storage.save_statement(&problem.id, statement) {
            let _ = state.block_on(repo::update_problem(
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

    state.notify_imported("problem", &format!("Created {}", source_problem_id));

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

    let existing = state
        .block_on(repo::submission_by_external_run_id(
            &state.pool,
            &external_run_id,
        ))
        .map_err(|e| format!("DB error: {}", e))?;

    let source_problem_id = format!("{}-{}", payload.oj, payload.prob_num);

    let problem_id = match state.block_on(repo::find_problem_by_source_id(
        &state.pool,
        "VJudge",
        &source_problem_id,
    )) {
        Ok(Some(problem)) => problem.id,
        Ok(None) => {
            let created = state
                .block_on(repo::create_problem(
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
        if !code_path.is_empty() && existing_sub.code_path.is_empty() {
            let _ = state.block_on(repo::set_submission_code_path(
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

    state
        .block_on(repo::create_submission(
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

    state.notify_imported(
        "submission",
        &format!(
            "#{} {}-{} {}",
            payload.run_id, payload.oj, payload.prob_num, payload.status
        ),
    );

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
