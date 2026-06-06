use crate::error::AppResult;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn connect(database_url: &str) -> AppResult<DatabaseConnection> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}

/// Run all pending migrations on startup.
pub async fn run_migrations(db: &DatabaseConnection) -> AppResult<()> {
    acmind_migration::Migrator::up(db, None)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("migration: {e}")))?;
    Ok(())
}
