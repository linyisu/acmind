use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    ai::service::AiService,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Serialize;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/ai/analyze/{submission_id}", post(analyze))
        .route("/ai/analyses", get(list))
        .route("/ai/test", get(test_connection))
}

async fn analyze(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(submission_id): Path<i64>,
) -> AppResult<Json<crate::ai::model::AnalysisResp>> {
    let svc = AiService::new(&state);
    Ok(Json(svc.analyze_submission(ctx.user_id, submission_id).await?))
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<crate::ai::model::AnalysisResp>>> {
    let svc = AiService::new(&state);
    Ok(Json(svc.list(ctx.user_id).await?))
}

#[derive(Serialize)]
struct TestResp {
    ok: bool,
    message: String,
}

async fn test_connection(
    State(state): State<AppState>,
) -> AppResult<Json<TestResp>> {
    match state.llm.chat("Reply with exactly: OK", "ping").await {
        Ok(resp) => Ok(Json(TestResp {
            ok: true,
            message: format!("AI responded: {}", resp.chars().take(100).collect::<String>()),
        })),
        Err(e) => Ok(Json(TestResp {
            ok: false,
            message: format!("AI connection failed: {e}"),
        })),
    }
}
