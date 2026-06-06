use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    problem::{
        model::{CreateProblemReq, ProblemResp, UpdateProblemReq},
        service::ProblemService,
    },
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json, Router,
    routing::get,
};
use serde::Deserialize;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/problems", get(list).post(create))
        .route(
            "/problems/:id",
            get(get_one).patch(update).delete(remove),
        )
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "tag_id")]
    tag_id: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<ProblemResp>>> {
    let svc = ProblemService::new(&state);
    Ok(Json(svc.list(ctx.user_id, q.tag_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<ProblemResp>> {
    let svc = ProblemService::new(&state);
    Ok(Json(svc.get(ctx.user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateProblemReq>,
) -> AppResult<Json<ProblemResp>> {
    let svc = ProblemService::new(&state);
    Ok(Json(svc.create(ctx.user_id, req).await?))
}

async fn update(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateProblemReq>,
) -> AppResult<Json<ProblemResp>> {
    let svc = ProblemService::new(&state);
    Ok(Json(svc.update(ctx.user_id, id, req).await?))
}

async fn remove(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<()>> {
    let svc = ProblemService::new(&state);
    svc.delete(ctx.user_id, id).await?;
    Ok(Json(()))
}
