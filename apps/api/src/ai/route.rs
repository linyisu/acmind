use crate::{
    ai::{orchestrator, service::AiService},
    auth::middleware::UserContext,
    error::AppResult,
    state::AppState,
    task::{model::TaskResp, repo as task_repo},
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/ai/analyze/{submission_id}", post(analyze))
        .route("/ai/analyze-problem/{problem_id}", post(analyze_problem))
        .route("/ai/analyses", get(list))
        .route("/ai/test", get(test_connection))
}

async fn analyze(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(submission_id): Path<i64>,
) -> AppResult<Json<crate::ai::model::AnalysisResp>> {
    let svc = AiService::new(&state);
    Ok(Json(
        svc.analyze_submission(ctx.user_id, submission_id).await?,
    ))
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<crate::ai::model::AnalysisResp>>> {
    let svc = AiService::new(&state);
    Ok(Json(svc.list(ctx.user_id).await?))
}

/// POST /ai/analyze-problem/{problem_id}
/// Creates a background task with agent-level progress and returns immediately.
async fn analyze_problem(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(problem_id): Path<i64>,
) -> AppResult<Json<TaskResp>> {
    let initial_progress = orchestrator::new_initial_progress();

    let task = task_repo::create(
        &state.db,
        ctx.user_id,
        "ai_full_analysis",
        "problem",
        problem_id,
        initial_progress,
    )
    .await?;

    // Spawn background task
    let state = std::sync::Arc::new(state);
    let db = state.db.clone();
    let task_id = task.id;
    tokio::spawn(async move {
        if let Err(e) = orchestrator::run_task(&state, task_id, ctx.user_id, problem_id).await {
            tracing::error!("[task-{}] 失败: {e}", task_id);
            let _ = task_repo::mark_failed(&db, task_id, &e.to_string()).await;
        }
    });

    Ok(Json(TaskResp::from_model(task)))
}

use serde::Serialize;

#[derive(Serialize)]
struct TestResp {
    ok: bool,
    message: String,
}

async fn test_connection(State(state): State<AppState>) -> AppResult<Json<TestResp>> {
    match state.llm.chat("Reply with exactly: OK", "ping").await {
        Ok(resp) => Ok(Json(TestResp {
            ok: true,
            message: format!(
                "AI responded: {}",
                resp.chars().take(100).collect::<String>()
            ),
        })),
        Err(e) => Ok(Json(TestResp {
            ok: false,
            message: format!("AI connection failed: {e}"),
        })),
    }
}
