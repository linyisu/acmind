mod api_helpers;

use api_helpers::{auth_token, has_test_db, test_router, test_state};
use axum::body::Body;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn make_body(json: &Value) -> Body {
    Body::from(serde_json::to_vec(json).unwrap())
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL; run with `cargo test -- --ignored`"]
async fn health_returns_200() {
    // Health endpoint doesn't need DB — construct a minimal router
    let state = test_state().await;
    let app = test_router(state);
    let req = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn register_and_login_flow() {
    if !has_test_db() {
        eprintln!("skipping: TEST_DATABASE_URL not set");
        return;
    }
    let state = test_state().await;
    let app = test_router(state.clone());

    // Register
    let body = serde_json::json!({
        "username": "testuser_auth_flow",
        "email": "test_auth_flow@example.com",
        "password": "securepassword123"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let user: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(user["username"], "testuser_auth_flow");
    assert!(user["id"].as_i64().unwrap() > 0);

    // Login
    let body = serde_json::json!({
        "username": "testuser_auth_flow",
        "password": "securepassword123"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let login: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(login["token"].as_str().unwrap().len() > 10);
    assert_eq!(login["user"]["username"], "testuser_auth_flow");

    // Me (with token)
    let token = login["token"].as_str().unwrap();
    let req = axum::http::Request::builder()
        .uri("/api/v1/auth/me")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let me: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(me["username"], "testuser_auth_flow");
}

#[tokio::test]
async fn register_duplicate_username_returns_409() {
    if !has_test_db() {
        eprintln!("skipping: TEST_DATABASE_URL not set");
        return;
    }
    let state = test_state().await;
    let app = test_router(state);

    let body = serde_json::json!({
        "username": "dup_user_409",
        "email": "dup_409@example.com",
        "password": "securepassword123"
    });

    // First registration
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    // Second registration (same username)
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_short_password_returns_400() {
    if !has_test_db() {
        eprintln!("skipping: TEST_DATABASE_URL not set");
        return;
    }
    let state = test_state().await;
    let app = test_router(state);

    let body = serde_json::json!({
        "username": "short_pw_user",
        "email": "short@example.com",
        "password": "abc"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    if !has_test_db() {
        eprintln!("skipping: TEST_DATABASE_URL not set");
        return;
    }
    let state = test_state().await;
    let app = test_router(state);

    // Register
    let body = serde_json::json!({
        "username": "wrong_pw_user",
        "email": "wrong_pw@example.com",
        "password": "correctpassword"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    // Login with wrong password
    let body = serde_json::json!({
        "username": "wrong_pw_user",
        "password": "wrongpassword"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(make_body(&body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL; run with `cargo test -- --ignored`"]
async fn me_without_token_returns_401() {
    let state = test_state().await;
    let app = test_router(state);
    let req = axum::http::Request::builder()
        .uri("/api/v1/auth/me")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL; run with `cargo test -- --ignored`"]
async fn me_with_invalid_token_returns_401() {
    let state = test_state().await;
    let app = test_router(state);
    let req = axum::http::Request::builder()
        .uri("/api/v1/auth/me")
        .header("authorization", "Bearer invalid.jwt.token")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL; run with `cargo test -- --ignored`"]
async fn protected_endpoint_without_token_returns_401() {
    let state = test_state().await;
    let app = test_router(state);
    let req = axum::http::Request::builder()
        .uri("/api/v1/problems")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_endpoint_with_valid_token_returns_200() {
    if !has_test_db() {
        eprintln!("skipping: TEST_DATABASE_URL not set");
        return;
    }
    let state = test_state().await;
    let token = auth_token(&state.jwt_secret, 999999, "phantom_user");
    let app = test_router(state);
    let req = axum::http::Request::builder()
        .uri("/api/v1/problems")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
}
