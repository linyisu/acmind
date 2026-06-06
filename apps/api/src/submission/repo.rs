use crate::{error::AppResult, submission::model::SubmissionRow};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: i64,
    language: &str,
    code: &str,
    verdict: &str,
    runtime_ms: Option<i32>,
    memory_kb: Option<i32>,
    notes: Option<&str>,
) -> AppResult<SubmissionRow> {
    let now = Utc::now();
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO submission (user_id, problem_id, language, code, verdict, runtime_ms, memory_kb, notes, submitted_at)
               VALUES ({}, {}, '{}', '{}', '{}', {}, {}, {}, '{}')
               RETURNING id, user_id, problem_id, language, code, verdict, runtime_ms, memory_kb, notes, submitted_at"#,
            user_id,
            problem_id,
            esc(language),
            esc(code),
            esc(verdict),
            opt_i32(runtime_ms),
            opt_i32(memory_kb),
            opt_str(notes),
            now.to_rfc3339(),
        ),
    );
    let row = db
        .query_one(stmt)
        .await?
        .ok_or_else(|| crate::error::AppError::Internal("submission insert returned no row".into()))?;
    row_to_submission(row).ok_or_else(|| crate::error::AppError::Internal("submission row parse".into()))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<SubmissionRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, problem_id, language, code, verdict, runtime_ms, memory_kb, notes, submitted_at
               FROM submission WHERE id = {} AND user_id = {}"#,
            id, user_id
        ),
    );
    let row = db.query_one(stmt).await?;
    Ok(row.and_then(row_to_submission))
}

pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: Option<i64>,
) -> AppResult<Vec<SubmissionRow>> {
    let filter = match problem_id {
        Some(p) => format!(" AND problem_id = {}", p),
        None => String::new(),
    };
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, problem_id, language, code, verdict, runtime_ms, memory_kb, notes, submitted_at
               FROM submission WHERE user_id = {}{} ORDER BY submitted_at DESC"#,
            user_id, filter
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_submission).collect())
}

pub async fn problem_belongs_to_user(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: i64,
) -> AppResult<bool> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT 1 AS x FROM problem WHERE id = {} AND user_id = {}",
            problem_id, user_id
        ),
    );
    Ok(db.query_one(stmt).await?.is_some())
}

pub fn row_to_submission(row: sea_orm::QueryResult) -> Option<SubmissionRow> {
    Some(SubmissionRow {
        id: row.try_get_by::<i64, _>("id").ok()?,
        user_id: row.try_get_by::<i64, _>("user_id").ok()?,
        problem_id: row.try_get_by::<i64, _>("problem_id").ok()?,
        language: row.try_get_by::<String, _>("language").ok()?,
        code: row.try_get_by::<String, _>("code").ok()?,
        verdict: row.try_get_by::<String, _>("verdict").ok()?,
        runtime_ms: row.try_get_by::<Option<i32>, _>("runtime_ms").ok()?,
        memory_kb: row.try_get_by::<Option<i32>, _>("memory_kb").ok()?,
        notes: row.try_get_by::<Option<String>, _>("notes").ok()?,
        submitted_at: row.try_get_by::<chrono::DateTime<chrono::Utc>, _>("submitted_at").ok()?,
    })
}

fn esc(s: &str) -> String { s.replace('\'', "''") }
fn opt_str(s: Option<&str>) -> String {
    match s { Some(v) => format!("'{}'", esc(v)), None => "NULL".to_string() }
}
fn opt_i32(v: Option<i32>) -> String {
    match v { Some(n) => n.to_string(), None => "NULL".to_string() }
}
