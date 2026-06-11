use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> AppResult<Option<UserRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, username, email, password_hash, created_at, updated_at
               FROM "user" WHERE username = '{}'"#,
            username.replace('\'', "''")
        ),
    );
    let result = db.query_one(stmt).await?;
    Ok(result.and_then(row_to_user))
}

pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> AppResult<Option<UserRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, username, email, password_hash, created_at, updated_at
               FROM "user" WHERE id = {}"#,
            id
        ),
    );
    let result = db.query_one(stmt).await?;
    Ok(result.and_then(row_to_user))
}

pub async fn insert(
    db: &DatabaseConnection,
    username: &str,
    email: &str,
    password_hash: &str,
) -> AppResult<UserRow> {
    let now = Utc::now();
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO "user" (username, email, password_hash, created_at, updated_at)
               VALUES ('{}', '{}', '{}', '{}', '{}')
               RETURNING id, username, email, password_hash, created_at, updated_at"#,
            username.replace('\'', "''"),
            email.replace('\'', "''"),
            password_hash.replace('\'', "''"),
            now.to_rfc3339(),
            now.to_rfc3339(),
        ),
    );
    let result = db
        .query_one(stmt)
        .await?
        .ok_or_else(|| crate::error::AppError::Internal("user insert returned no row".into()))?;
    row_to_user(result)
        .ok_or_else(|| crate::error::AppError::Internal("user row parse failed".into()))
}

fn row_to_user(row: sea_orm::QueryResult) -> Option<UserRow> {
    Some(UserRow {
        id: row.try_get_by::<i64, _>("id").ok()?,
        username: row.try_get_by::<String, _>("username").ok()?,
        email: row.try_get_by::<String, _>("email").ok()?,
        password_hash: row.try_get_by::<String, _>("password_hash").ok()?,
        created_at: row
            .try_get_by::<chrono::DateTime<chrono::Utc>, _>("created_at")
            .ok()?,
        updated_at: row
            .try_get_by::<chrono::DateTime<chrono::Utc>, _>("updated_at")
            .ok()?,
    })
}
