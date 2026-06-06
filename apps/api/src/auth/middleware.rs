use crate::{auth::jwt, error::{AppError, AppResult}, state::AppState};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: i64,
    pub username: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let claims = jwt::verify(token, &state.jwt_secret)?;
    let ctx = UserContext {
        user_id: claims.sub,
        username: claims.username,
    };
    req.extensions_mut().insert(ctx);
    Ok(next.run(req).await)
}
