use crate::{auth::jwt, error::{AppError, AppResult}, state::AppState};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, middleware, routing::get, Router};
    use sea_orm::Database;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn dummy_handler() -> &'static str {
        "ok"
    }

    async fn test_state_no_db() -> AppState {
        // Connect to in-memory SQLite — middleware never touches the DB
        let db = Database::connect("sqlite::memory:").await.unwrap();
        AppState {
            db,
            jwt_secret: Arc::new("test-middleware-secret".into()),
            jwt_expires_in: 3600,
            allow_register: true,
        }
    }

    fn build_test_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(dummy_handler))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                require_auth,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn valid_token_passes_through() {
        let state = test_state_no_db().await;
        let token = jwt::issue(&state.jwt_secret, 1, "testuser", 3600).unwrap();
        let app = build_test_router(state);
        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let state = test_state_no_db().await;
        let app = build_test_router(state);
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let state = test_state_no_db().await;
        let app = build_test_router(state);
        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, "Bearer garbage.token.value")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_secret_returns_401() {
        let state = test_state_no_db().await;
        let token = jwt::issue("wrong-secret", 1, "testuser", 3600).unwrap();
        let app = build_test_router(state);
        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn expired_token_returns_401() {
        let state = test_state_no_db().await;
        // Issue a token that expired 1 hour ago
        let token = jwt::issue(&state.jwt_secret, 1, "testuser", -3600).unwrap();
        let app = build_test_router(state);
        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}

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
