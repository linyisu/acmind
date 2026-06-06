use crate::{
    error::{AppError, AppResult},
    problem::model::ProblemRow,
};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    source: &str,
    external_id: Option<&str>,
    title: &str,
    url: Option<&str>,
    difficulty: Option<i32>,
    statement: Option<&str>,
) -> AppResult<ProblemRow> {
    let now = Utc::now();
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO problem (user_id, source, external_id, title, url, difficulty, statement, created_at, updated_at)
               VALUES ({}, '{}', {}, '{}', {}, {}, {}, '{}', '{}')
               RETURNING id, user_id, source, external_id, title, url, difficulty, statement, created_at, updated_at"#,
            user_id,
            esc(source),
            opt_str(external_id),
            esc(title),
            opt_str(url),
            opt_i32(difficulty),
            opt_str(statement),
            now.to_rfc3339(),
            now.to_rfc3339(),
        ),
    );
    let row = db
        .query_one(stmt)
        .await?
        .ok_or_else(|| AppError::Internal("problem insert returned no row".into()))?;
    row_to_problem(row).ok_or_else(|| AppError::Internal("problem row parse failed".into()))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<ProblemRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, source, external_id, title, url, difficulty, statement, created_at, updated_at
               FROM problem WHERE id = {} AND user_id = {}"#,
            id, user_id
        ),
    );
    let row = db.query_one(stmt).await?;
    Ok(row.and_then(row_to_problem))
}

pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
    tag_id: Option<i64>,
) -> AppResult<Vec<ProblemRow>> {
    let tag_filter = match tag_id {
        Some(t) => format!(
            " AND id IN (SELECT problem_id FROM problem_tag WHERE tag_id = {})",
            t
        ),
        None => String::new(),
    };
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, source, external_id, title, url, difficulty, statement, created_at, updated_at
               FROM problem WHERE user_id = {}{} ORDER BY id DESC"#,
            user_id, tag_filter
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_problem).collect())
}

pub async fn update(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
    source: Option<&str>,
    external_id: Option<&str>,
    title: Option<&str>,
    url: Option<&str>,
    difficulty: Option<i32>,
    statement: Option<&str>,
) -> AppResult<Option<ProblemRow>> {
    // Build dynamic SET clause; None entries are skipped.
    let mut sets: Vec<String> = vec![];
    if let Some(s) = source { sets.push(format!("source = '{}'", esc(s))); }
    if let Some(e) = external_id { sets.push(format!("external_id = {}", opt_str(Some(e)))); }
    if let Some(t) = title { sets.push(format!("title = '{}'", esc(t))); }
    if let Some(u) = url { sets.push(format!("url = {}", opt_str(Some(u)))); }
    if let Some(d) = difficulty { sets.push(format!("difficulty = {}", opt_i32(Some(d)))); }
    if let Some(s) = statement { sets.push(format!("statement = {}", opt_str(Some(s)))); }
    if sets.is_empty() { return find_by_id(db, user_id, id).await; }
    let now = Utc::now();
    sets.push(format!("updated_at = '{}'", now.to_rfc3339()));
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"UPDATE problem SET {} WHERE id = {} AND user_id = {}
               RETURNING id, user_id, source, external_id, title, url, difficulty, statement, created_at, updated_at"#,
            sets.join(", "), id, user_id
        ),
    );
    let row = db.query_one(stmt).await?;
    Ok(row.and_then(row_to_problem))
}

pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"DELETE FROM problem WHERE id = {} AND user_id = {}"#,
            id, user_id
        ),
    );
    let res = db.execute(stmt).await?;
    Ok(res.rows_affected() > 0)
}

pub fn row_to_problem(row: sea_orm::QueryResult) -> Option<ProblemRow> {
    Some(ProblemRow {
        id: row.try_get_by::<i64, _>("id").ok()?,
        user_id: row.try_get_by::<i64, _>("user_id").ok()?,
        source: row.try_get_by::<String, _>("source").ok()?,
        external_id: row.try_get_by::<Option<String>, _>("external_id").ok()?,
        title: row.try_get_by::<String, _>("title").ok()?,
        url: row.try_get_by::<Option<String>, _>("url").ok()?,
        difficulty: row.try_get_by::<Option<i32>, _>("difficulty").ok()?,
        statement: row.try_get_by::<Option<String>, _>("statement").ok()?,
        created_at: row.try_get_by::<chrono::DateTime<chrono::Utc>, _>("created_at").ok()?,
        updated_at: row.try_get_by::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok()?,
    })
}

// Helpers
fn esc(s: &str) -> String { s.replace('\'', "''") }
fn opt_str(s: Option<&str>) -> String {
    match s { Some(v) => format!("'{}'", esc(v)), None => "NULL".to_string() }
}
fn opt_i32(v: Option<i32>) -> String {
    match v { Some(n) => n.to_string(), None => "NULL".to_string() }
}
