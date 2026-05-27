//! Local HTTP server that receives scraped data from the ACMind browser extension.
//! Listens on 127.0.0.1:18921 (loopback only, no external exposure).

use crate::import_service::{
    import_problem, import_submission, import_submissions, ApiResponse, ImportNotifier,
    ImportProblemPayload, ImportState, ImportSubmissionPayload, ImportSubmissionsPayload,
};
use crate::storage::Storage;
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::thread;
use tauri::Emitter;
use tiny_http::{Header, Method, Request, Response, StatusCode};

const BIND_ADDR: &str = "127.0.0.1:18921";

pub struct ImportServerHandle {}

impl ImportServerHandle {
    fn new() -> Self {
        Self {}
    }
}

/// Start the import HTTP server on a background thread.
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
    let notify: ImportNotifier = Arc::new(move |action, detail| {
        let payload = serde_json::json!({
            "action": action,
            "detail": detail,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });
        if let Err(e) = app_handle.emit("vjudge-imported", &payload) {
            tracing::warn!(target: "app_lib::import_server", "Failed to emit vjudge-imported event: {}", e);
        }
    });
    let state = Arc::new(ImportState {
        pool,
        storage,
        rt: handle,
        notify,
    });
    let server = tiny_http::Server::http(BIND_ADDR).expect("Failed to start ACMind import server");

    tracing::info!(target: "app_lib::import_server", "Import server listening on {}", BIND_ADDR);

    let import_handle = ImportServerHandle::new();

    thread::spawn(move || {
        let _guard = rt.enter();
        for request in server.incoming_requests() {
            let state = Arc::clone(&state);
            thread::spawn(move || handle_request(state, request));
        }
    });

    import_handle
}

fn handle_request(state: Arc<ImportState>, mut request: Request) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or(&url);

    let cors_headers = cors_headers();

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
        _ => Ok(json_response(StatusCode(404), &error_response("Not found"))),
    };

    let resp = match result {
        Ok(response) => response,
        Err(err) => json_response(StatusCode(500), &error_response(&err)),
    };

    let _ = request.respond(resp);
}

fn health_response() -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(
        StatusCode(200),
        &serde_json::json!({"status": "ok", "app": "acmind"}),
    )
}

fn handle_import_submissions(
    state: &ImportState,
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

    tracing::info!(target: "app_lib::import_server", "Importing {} submissions for user {} from browser extension", payload.items.len(), payload.username);

    let (imported, skipped, created_problems) = import_submissions(state, &payload)?;

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(format!(
                "Imported {} submissions ({} new, {} skipped, {} new problems)",
                payload.items.len(), imported, skipped, created_problems
            )),
            imported: Some(imported),
            skipped: Some(skipped),
            created_problems: Some(created_problems),
            source_synced: None,
        },
    ))
}

fn handle_import_problem(
    state: &ImportState,
    body: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, String> {
    let payload: ImportProblemPayload =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;

    tracing::info!(target: "app_lib::import_server", "Importing problem {} from browser extension", payload.source_problem_id);

    let created = import_problem(state, &payload)?;
    let imported = usize::from(created);
    let action = if created { "imported" } else { "already exists, updated" };

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(format!("Problem {} {}", payload.source_problem_id, action)),
            imported: Some(imported),
            skipped: None,
            created_problems: Some(imported),
            source_synced: None,
        },
    ))
}

fn handle_import_submission(
    state: &ImportState,
    body: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, String> {
    let payload: ImportSubmissionPayload =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;

    tracing::info!(target: "app_lib::import_server", "Importing single submission run#{} with source code from browser extension", payload.run_id);

    let had_source = payload.code.as_ref().is_some_and(|code| !code.is_empty());
    let result = import_submission(state, &payload)?;

    Ok(json_response(
        StatusCode(200),
        &ApiResponse {
            success: true,
            message: Some(if result.created {
                format!("Submission #{} imported with source code", payload.run_id)
            } else if result.source_synced {
                format!("Submission #{} already exists, source synced", payload.run_id)
            } else if had_source {
                format!(
                    "Submission #{} already exists, source already present",
                    payload.run_id
                )
            } else {
                format!("Submission #{} already exists, no source provided", payload.run_id)
            }),
            imported: Some(usize::from(result.created)),
            skipped: None,
            created_problems: None,
            source_synced: Some(usize::from(result.source_synced)),
        },
    ))
}

fn error_response(message: &str) -> ApiResponse {
    ApiResponse {
        success: false,
        message: Some(message.into()),
        imported: None,
        skipped: None,
        created_problems: None,
        source_synced: None,
    }
}

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Allow-Methods"[..],
            &b"GET, POST, OPTIONS"[..],
        )
        .unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap(),
    ]
}

fn json_response<T: Serialize>(status: StatusCode, data: &T) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_vec(data).unwrap_or_else(|_| b"{}".to_vec());
    let len = body.len();

    Response::new(
        status,
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            Header::from_bytes(&b"Content-Length"[..], len.to_string().as_bytes()).unwrap(),
        ]
        .into_iter()
        .chain(cors_headers())
        .collect::<Vec<_>>(),
        std::io::Cursor::new(body),
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{CreateProblemInput, CreateSubmissionInput};
    use crate::db::repo;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test pool");

        sqlx::query(include_str!("../migrations/001_init.sql"))
            .execute(&pool)
            .await
            .expect("failed to init test schema");
        sqlx::query(include_str!("../migrations/002_settings.sql"))
            .execute(&pool)
            .await
            .expect("failed to init settings schema");
        sqlx::query("ALTER TABLE submissions ADD COLUMN external_run_id TEXT")
            .execute(&pool)
            .await
            .expect("failed to init external submissions column");
        sqlx::query(include_str!("../migrations/003_external_submissions.sql"))
            .execute(&pool)
            .await
            .expect("failed to init external submissions index");

        pool
    }

    fn test_state(
        pool: SqlitePool,
        notifications: Arc<Mutex<Vec<(String, String)>>>,
        rt: tokio::runtime::Handle,
    ) -> ImportState {
        ImportState {
            pool,
            storage: Storage::new(PathBuf::from(std::env::temp_dir()).join("acmind-import-tests")),
            rt,
            notify: Arc::new(move |action, detail| {
                notifications
                    .lock()
                    .unwrap()
                    .push((action.to_string(), detail.to_string()));
            }),
        }
    }

    #[test]
    fn existing_problem_import_notifies_frontend() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(test_pool());
        rt.block_on(repo::create_problem(
            &pool,
            &CreateProblemInput {
                source: "VJudge".into(),
                source_problem_id: "CodeForces-1A".into(),
                title: "Old title".into(),
                url: None,
                difficulty: None,
                tags: vec![],
                statement: None,
            },
            None,
        ))
        .unwrap();

        let notifications = Arc::new(Mutex::new(Vec::new()));
        let state = test_state(pool, Arc::clone(&notifications), rt.handle().clone());
        let body = serde_json::json!({
            "sourceProblemId": "CodeForces-1A",
            "oj": "CodeForces",
            "probNum": "1A",
            "title": "New title",
            "url": "https://vjudge.net/problem/CodeForces-1A",
            "statement": null,
            "tags": []
        })
        .to_string();

        handle_import_problem(&state, &body).unwrap();

        let notifications = notifications.lock().unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].0, "problem");
        assert!(notifications[0].1.contains("Updated CodeForces-1A"));
    }

    #[test]
    fn existing_submission_source_sync_notifies_frontend() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(test_pool());
        let problem = rt
            .block_on(repo::create_problem(
                &pool,
                &CreateProblemInput {
                    source: "VJudge".into(),
                    source_problem_id: "CodeForces-1A".into(),
                    title: "Problem".into(),
                    url: None,
                    difficulty: None,
                    tags: vec![],
                    statement: None,
                },
                None,
            ))
            .unwrap();
        rt.block_on(repo::create_submission(
            &pool,
            &CreateSubmissionInput {
                problem_id: problem.id,
                status: "AC".into(),
                language: "Rust".into(),
                code_text: "".into(),
                runtime: None,
                memory: None,
                note: None,
                external_run_id: Some("vjudge:123".into()),
                submitted_at: None,
            },
            "",
        ))
        .unwrap();

        let notifications = Arc::new(Mutex::new(Vec::new()));
        let state = test_state(pool, Arc::clone(&notifications), rt.handle().clone());
        let body = serde_json::json!({
            "runId": 123,
            "oj": "CodeForces",
            "probNum": "1A",
            "status": "Accepted",
            "language": "Rust",
            "code": "fn main() {}",
            "runtime": 31,
            "memory": 128,
            "submitTime": 1710000000000i64
        })
        .to_string();

        handle_import_submission(&state, &body).unwrap();

        let notifications = notifications.lock().unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].0, "submission");
        assert!(notifications[0].1.contains("#123 source synced"));
    }
}
