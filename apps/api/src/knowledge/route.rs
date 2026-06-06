use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    knowledge::{
        model::{CreateKnowledgeReq, KnowledgeResp, UpdateKnowledgeReq},
        service::KnowledgeService,
    },
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json, Router,
    routing::get,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/knowledge", get(list).post(create))
        .route(
            "/knowledge/:id",
            get(get_one).patch(update).delete(remove),
        )
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<KnowledgeResp>>> {
    let svc = KnowledgeService::new(&state);
    Ok(Json(svc.list(ctx.user_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<KnowledgeResp>> {
    let svc = KnowledgeService::new(&state);
    Ok(Json(svc.get(ctx.user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateKnowledgeReq>,
) -> AppResult<Json<KnowledgeResp>> {
    let svc = KnowledgeService::new(&state);
    Ok(Json(svc.create(ctx.user_id, req).await?))
}

async fn update(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateKnowledgeReq>,
) -> AppResult<Json<KnowledgeResp>> {
    let svc = KnowledgeService::new(&state);
    Ok(Json(svc.update(ctx.user_id, id, req).await?))
}

async fn remove(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<()>> {
    let svc = KnowledgeService::new(&state);
    svc.delete(ctx.user_id, id).await?;
    Ok(Json(()))
}
