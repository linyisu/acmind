use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    state::AppState,
    submission::{
        model::{CreateSubmissionReq, SubmissionResp},
        service::SubmissionService,
    },
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json, Router,
    routing::get,
};
use serde::Deserialize;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/submissions", get(list).post(create))
        .route("/submissions/:id", get(get_one))
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "problem_id")]
    problem_id: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<SubmissionResp>>> {
    let svc = SubmissionService::new(&state);
    Ok(Json(svc.list(ctx.user_id, q.problem_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<SubmissionResp>> {
    let svc = SubmissionService::new(&state);
    Ok(Json(svc.get(ctx.user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateSubmissionReq>,
) -> AppResult<Json<SubmissionResp>> {
    let svc = SubmissionService::new(&state);
    Ok(Json(svc.create(ctx.user_id, req).await?))
}
