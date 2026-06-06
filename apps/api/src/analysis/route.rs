use crate::{
    analysis::service::{AnalysisService, DifficultyBucket, SummaryResp, TimelinePoint},
    auth::middleware::UserContext,
    error::AppResult,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    routing::get,
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/analysis/submissions/summary", get(summary))
        .route("/analysis/submissions/timeline", get(timeline))
        .route(
            "/analysis/problems/difficulty-distribution",
            get(difficulty_dist),
        )
}

async fn summary(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<SummaryResp>> {
    let svc = AnalysisService::new(&state);
    Ok(Json(svc.submissions_summary(ctx.user_id).await?))
}

#[derive(Deserialize)]
struct TimelineQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

async fn timeline(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
    Query(q): Query<TimelineQuery>,
) -> AppResult<Json<Vec<TimelinePoint>>> {
    let svc = AnalysisService::new(&state);
    Ok(Json(svc.submissions_timeline(ctx.user_id, q.from, q.to).await?))
}

async fn difficulty_dist(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<Vec<DifficultyBucket>>> {
    let svc = AnalysisService::new(&state);
    Ok(Json(svc.difficulty_distribution(ctx.user_id).await?))
}
