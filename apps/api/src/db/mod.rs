use crate::error::AppResult;
use sea_orm::{Database, DatabaseConnection};

pub async fn connect(database_url: &str) -> AppResult<DatabaseConnection> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}

pub async fn run_migrations(db: &DatabaseConnection) -> AppResult<()> {
    // Phase 2 will add real migrations. For Phase 1, create the user table directly.
    use sea_orm::{ConnectionTrait, Statement};
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        CREATE TABLE IF NOT EXISTS "user" (
            id BIGSERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
        "#,
    );
    db.execute(stmt).await?;
    Ok(())
}
