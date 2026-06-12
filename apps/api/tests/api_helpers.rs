use acmind_api::{ai::provider::NoopLlmProvider, auth::jwt, state::AppState};
use acmind_migration::MigratorTrait;
use axum::extract::connect_info::MockConnectInfo;
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::OnceCell;

static TEST_STATE: OnceCell<AppState> = OnceCell::const_new();

async fn init_test_state() -> AppState {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set for integration tests");
    let db = Database::connect(&db_url)
        .await
        .expect("failed to connect to test database");
    acmind_migration::Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");
    AppState {
        db,
        jwt_secret: Arc::new("test-secret-for-integration".into()),
        jwt_expires_in: 3600,
        allow_register: true,
        rate_limit_per_second: 100,
        rate_limit_burst: 200,
        llm: Arc::new(NoopLlmProvider),
    }
}

pub async fn test_state() -> AppState {
    TEST_STATE.get_or_init(init_test_state).await.clone()
}

pub fn test_router(state: AppState) -> axum::Router {
    acmind_api::build_router(state).layer(MockConnectInfo(test_client_addr()))
}

fn test_client_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 3000))
}

/// Issue a valid JWT token for the given user.
pub fn auth_token(secret: &str, user_id: i64, username: &str) -> String {
    jwt::issue(secret, user_id, username, 3600).expect("failed to issue test token")
}

/// Check if TEST_DATABASE_URL is set. If not, return false.
pub fn has_test_db() -> bool {
    std::env::var("TEST_DATABASE_URL").is_ok()
}
