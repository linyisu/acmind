use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    state::AppState,
    tag::{model::CreateTagReq, model::TagResp, service::TagService},
};
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Extension, Json, Router,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/tags", get(list).post(create))
        .route("/tags/{id}", delete(remove))
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<TagResp>>> {
    let svc = TagService::new(&state);
    Ok(Json(svc.list(ctx.user_id).await?))
}

async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateTagReq>,
) -> AppResult<Json<TagResp>> {
    let svc = TagService::new(&state);
    Ok(Json(svc.create(ctx.user_id, &req.name).await?))
}

async fn remove(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<()>> {
    let svc = TagService::new(&state);
    svc.delete(ctx.user_id, id).await?;
    Ok(Json(()))
}
