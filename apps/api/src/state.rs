use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: Arc<String>,
    pub jwt_expires_in: i64,
    pub allow_register: bool,
}
