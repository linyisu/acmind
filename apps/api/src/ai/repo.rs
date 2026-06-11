use crate::error::AppResult;
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde_json::Value;

pub struct AiAnalysisRow {
    pub id: i64,
    pub user_id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub result: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    target_type: &str,
    target_id: i64,
    result: &Value,
) -> AppResult<i64> {
    let now = Utc::now();
    let result_str = serde_json::to_string(result)
        .map_err(|e| crate::error::AppError::Internal(format!("serialize result: {e}")))?;
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO ai_analysis (user_id, target_type, target_id, result, created_at)
               VALUES ({}, '{}', {}, '{}', '{}')
               RETURNING id"#,
            user_id,
            target_type.replace('\'', "''"),
            target_id,
            result_str.replace('\'', "''"),
            now.to_rfc3339(),
        ),
    );
    let row = db.query_one(stmt).await?.ok_or_else(|| {
        crate::error::AppError::Internal("ai_analysis insert returned no row".into())
    })?;
    row.try_get_by::<i64, _>("id")
        .map_err(|e| crate::error::AppError::Internal(format!("ai_analysis id parse: {e}")))
}

pub async fn list_by_user(db: &DatabaseConnection, user_id: i64) -> AppResult<Vec<AiAnalysisRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT id, user_id, target_type, target_id, result, created_at \
             FROM ai_analysis WHERE user_id = {} ORDER BY created_at DESC",
            user_id,
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_analysis).collect())
}

pub async fn find_by_target(
    db: &DatabaseConnection,
    user_id: i64,
    target_type: &str,
    target_id: i64,
) -> AppResult<Option<AiAnalysisRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT id, user_id, target_type, target_id, result, created_at \
             FROM ai_analysis WHERE user_id = {} AND target_type = '{}' AND target_id = {} \
             ORDER BY created_at DESC LIMIT 1",
            user_id,
            target_type.replace('\'', "''"),
            target_id,
        ),
    );
    Ok(db.query_one(stmt).await?.and_then(row_to_analysis))
}

/// Batch query: find analyses for multiple targets at once.
pub async fn find_by_targets(
    db: &DatabaseConnection,
    user_id: i64,
    target_type: &str,
    target_ids: &[i64],
) -> AppResult<Vec<AiAnalysisRow>> {
    if target_ids.is_empty() {
        return Ok(vec![]);
    }
    let ids_str: Vec<String> = target_ids.iter().map(|id| id.to_string()).collect();
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT DISTINCT ON (target_id) id, user_id, target_type, target_id, result, created_at \
             FROM ai_analysis WHERE user_id = {} AND target_type = '{}' AND target_id IN ({}) \
             ORDER BY target_id, created_at DESC",
            user_id,
            target_type.replace('\'', "''"),
            ids_str.join(","),
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_analysis).collect())
}

fn row_to_analysis(row: sea_orm::QueryResult) -> Option<AiAnalysisRow> {
    let result_str: String = row.try_get_by("result").ok()?;
    let result: Value = serde_json::from_str(&result_str).ok()?;
    Some(AiAnalysisRow {
        id: row.try_get_by("id").ok()?,
        user_id: row.try_get_by("user_id").ok()?,
        target_type: row.try_get_by("target_type").ok()?,
        target_id: row.try_get_by("target_id").ok()?,
        result,
        created_at: row.try_get_by("created_at").ok()?,
    })
}
