use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    import::{
        model::{ImportProblemReq, ImportResp, ImportSingleSubmissionReq, ImportSubmissionResp, ImportSubmissionsReq},
        service::ImportService,
    },
    state::AppState,
};
use axum::{
    extract::State,
    routing::post,
    Extension, Json, Router,
};

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/import/vjudge/problem", post(import_problem))
        .route("/import/vjudge/submission", post(import_submission))
        .route("/import/vjudge/submissions", post(import_submissions_bulk))
}

async fn import_problem(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<ImportProblemReq>,
) -> AppResult<Json<ImportResp>> {
    let svc = ImportService::new(&state);
    let (_problem_id, is_new) = svc.import_problem(ctx.user_id, &req).await?;
    Ok(Json(ImportResp {
        created: if is_new { 1 } else { 0 },
        skipped: if is_new { 0 } else { 1 },
        errors: vec![],
    }))
}

async fn import_submission(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<ImportSingleSubmissionReq>,
) -> AppResult<Json<ImportSubmissionResp>> {
    let svc = ImportService::new(&state);
    let resp = svc.import_single_submission(ctx.user_id, &req).await?;
    Ok(Json(resp))
}

async fn import_submissions_bulk(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<ImportSubmissionsReq>,
) -> AppResult<Json<ImportResp>> {
    let svc = ImportService::new(&state);
    let resp = svc.import_submissions_bulk(ctx.user_id, &req).await?;
    Ok(Json(resp))
}
