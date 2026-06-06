use crate::{error::AppResult, tag::model::TagRow};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

pub async fn insert(db: &DatabaseConnection, user_id: i64, name: &str) -> AppResult<TagRow> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO tag (user_id, name) VALUES ({}, '{}')
               ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
               RETURNING id, user_id, name"#,
            user_id,
            name.replace('\'', "''")
        ),
    );
    let row = db
        .query_one(stmt)
        .await?
        .ok_or_else(|| crate::error::AppError::Internal("tag insert returned no row".into()))?;
    row_to_tag(row).ok_or_else(|| crate::error::AppError::Internal("tag row parse".into()))
}

pub async fn list_by_user(db: &DatabaseConnection, user_id: i64) -> AppResult<Vec<TagRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT id, user_id, name FROM tag WHERE user_id = {} ORDER BY name",
            user_id
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_tag).collect())
}

pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!("DELETE FROM tag WHERE id = {} AND user_id = {}", id, user_id),
    );
    let res = db.execute(stmt).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn exists_and_owned(
    db: &DatabaseConnection,
    user_id: i64,
    tag_id: i64,
) -> AppResult<bool> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT 1 AS x FROM tag WHERE id = {} AND user_id = {}",
            tag_id, user_id
        ),
    );
    Ok(db.query_one(stmt).await?.is_some())
}

pub fn row_to_tag(row: sea_orm::QueryResult) -> Option<TagRow> {
    Some(TagRow {
        id: row.try_get_by::<i64, _>("id").ok()?,
        user_id: row.try_get_by::<i64, _>("user_id").ok()?,
        name: row.try_get_by::<String, _>("name").ok()?,
    })
}
