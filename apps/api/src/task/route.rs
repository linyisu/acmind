use crate::{
    auth::middleware::UserContext,
    error::{AppError, AppResult},
    state::AppState,
    task::{model::TaskResp, repo},
};
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Extension, Json, Router,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list))
        .route("/tasks/{id}", get(get_one))
        .route("/tasks/{id}", delete(cancel))
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<TaskResp>>> {
    let rows = repo::list_by_user(&state.db, ctx.user_id, 20).await?;
    Ok(Json(rows.into_iter().map(TaskResp::from_model).collect()))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<TaskResp>> {
    let row = repo::find_by_id(&state.db, ctx.user_id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(TaskResp::from_model(row)))
}

/// Cancel a running task. The background worker periodically polls
/// `repo::is_cancelled` and exits early when set.
async fn cancel(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<bool>> {
    // Verify task belongs to user before cancelling
    let row = repo::find_by_id(&state.db, ctx.user_id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if row.status == "completed" || row.status == "failed" || row.status == "cancelled" {
        return Ok(Json(false));
    }
    let cancelled = repo::mark_cancelled(&state.db, id).await?;
    Ok(Json(cancelled))
}
