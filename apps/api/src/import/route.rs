use crate::{
    auth::middleware::UserContext,
    error::AppResult,
    import::{
        model::{ImportProblemFullReq, ImportProblemFullResp},
        service::ImportService,
    },
    state::AppState,
};
use axum::{extract::State, routing::post, Extension, Json, Router};

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/import/vjudge/problem-full", post(import_problem_full))
}

async fn import_problem_full(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<ImportProblemFullReq>,
) -> AppResult<Json<ImportProblemFullResp>> {
    let svc = ImportService::new(&state);
    let resp = svc.import_problem_full(ctx.user_id, &req).await?;
    Ok(Json(resp))
}
