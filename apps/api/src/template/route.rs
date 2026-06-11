use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    state::AppState,
    template::{
        model::{
            CreateTemplateReq, ListTemplatesQuery, TemplateResp, TemplateStats, UpdateTemplateReq,
        },
        service::TemplateService,
    },
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/templates", get(list).post(create))
        .route("/templates/stats", get(stats))
        .route("/templates/{id}", get(get_one).patch(update).delete(remove))
        .route(
            "/templates/{id}/problems/{problem_id}",
            post(link_problem).delete(unlink_problem),
        )
}

async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListTemplatesQuery>,
) -> AppResult<Json<Vec<TemplateResp>>> {
    let svc = TemplateService::new(&state);
    Ok(Json(svc.list(ctx.user_id, &query).await?))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<TemplateResp>> {
    let svc = TemplateService::new(&state);
    Ok(Json(svc.get(ctx.user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateTemplateReq>,
) -> AppResult<Json<TemplateResp>> {
    let svc = TemplateService::new(&state);
    Ok(Json(svc.create(ctx.user_id, req).await?))
}

async fn update(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTemplateReq>,
) -> AppResult<Json<TemplateResp>> {
    let svc = TemplateService::new(&state);
    Ok(Json(svc.update(ctx.user_id, id, req).await?))
}

async fn remove(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<()>> {
    let svc = TemplateService::new(&state);
    svc.delete(ctx.user_id, id).await?;
    Ok(Json(()))
}

async fn link_problem(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path((id, problem_id)): Path<(i64, i64)>,
) -> AppResult<Json<()>> {
    let svc = TemplateService::new(&state);
    svc.link_problem(ctx.user_id, id, problem_id).await?;
    Ok(Json(()))
}

async fn unlink_problem(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Path((id, problem_id)): Path<(i64, i64)>,
) -> AppResult<Json<()>> {
    let svc = TemplateService::new(&state);
    svc.unlink_problem(ctx.user_id, id, problem_id).await?;
    Ok(Json(()))
}

async fn stats(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<TemplateStats>> {
    let svc = TemplateService::new(&state);
    Ok(Json(svc.stats(ctx.user_id).await?))
}
