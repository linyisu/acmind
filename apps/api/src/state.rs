use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: Arc<String>,
    pub jwt_expires_in: i64,
    pub allow_register: bool,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
}
