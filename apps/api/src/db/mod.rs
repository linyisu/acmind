use crate::error::AppResult;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn connect(database_url: &str) -> AppResult<DatabaseConnection> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}

pub async fn run_migrations(db: &DatabaseConnection) -> AppResult<()> {
    let stmts = [
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
        r#"
        CREATE TABLE IF NOT EXISTS problem (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            source VARCHAR(64) NOT NULL,
            external_id VARCHAR(255),
            title VARCHAR(512) NOT NULL,
            url VARCHAR(1024),
            difficulty INTEGER,
            statement TEXT,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
        "#,
        r#"
        CREATE INDEX IF NOT EXISTS idx_problem_user ON problem(user_id)
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS submission (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            problem_id BIGINT NOT NULL REFERENCES problem(id) ON DELETE CASCADE,
            language VARCHAR(64) NOT NULL,
            code TEXT NOT NULL,
            verdict VARCHAR(16) NOT NULL,
            runtime_ms INTEGER,
            memory_kb INTEGER,
            notes TEXT,
            submitted_at TIMESTAMPTZ NOT NULL
        )
        "#,
        r#"
        CREATE INDEX IF NOT EXISTS idx_submission_user_problem
            ON submission(user_id, problem_id)
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS knowledge (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            problem_id BIGINT REFERENCES problem(id) ON DELETE SET NULL,
            kind VARCHAR(32) NOT NULL,
            title VARCHAR(512) NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS tag (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            name VARCHAR(64) NOT NULL,
            UNIQUE(user_id, name)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS problem_tag (
            problem_id BIGINT NOT NULL REFERENCES problem(id) ON DELETE CASCADE,
            tag_id BIGINT NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
            PRIMARY KEY (problem_id, tag_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS knowledge_tag (
            knowledge_id BIGINT NOT NULL REFERENCES knowledge(id) ON DELETE CASCADE,
            tag_id BIGINT NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
            PRIMARY KEY (knowledge_id, tag_id)
        )
        "#,
    ];
    for s in stmts {
        db.execute(Statement::from_string(DbBackend::Postgres, s)).await?;
    }
    Ok(())
}
