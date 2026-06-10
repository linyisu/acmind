use crate::{
    entity::task,
    error::AppResult,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde_json::Value;

/// Create a new task in pending state.
pub async fn create(
    db: &DatabaseConnection,
    user_id: i64,
    kind: &str,
    target_type: &str,
    target_id: i64,
    initial_progress: Value,
) -> AppResult<task::Model> {
    let am = task::ActiveModel {
        user_id: Set(user_id),
        kind: Set(kind.to_string()),
        status: Set("pending".to_string()),
        target_type: Set(target_type.to_string()),
        target_id: Set(target_id),
        progress: Set(initial_progress),
        ..Default::default()
    };
    Ok(am.insert(db).await?)
}

/// List tasks for a user, newest first. Limited to `limit` rows.
pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
    limit: u64,
) -> AppResult<Vec<task::Model>> {
    Ok(task::Entity::find()
        .filter(task::Column::UserId.eq(user_id))
        .order_by_desc(task::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await?)
}

/// Get a single task by ID, scoped to user.
pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<task::Model>> {
    Ok(task::Entity::find_by_id(id)
        .filter(task::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

/// Get a single task by ID (no user filter, for internal use by background tasks).
pub async fn find_by_id_system(
    db: &DatabaseConnection,
    id: i64,
) -> AppResult<Option<task::Model>> {
    Ok(task::Entity::find_by_id(id).one(db).await?)
}

/// Update task status to "running" and set started_at.
pub async fn mark_running(db: &DatabaseConnection, id: i64) -> AppResult<()> {
    let m = task::Entity::find_by_id(id)
        .one(db)
        .await?
        .expect("task must exist");
    let mut am: task::ActiveModel = m.into();
    am.status = Set("running".to_string());
    am.started_at = Set(Some(Utc::now().into()));
    am.update(db).await?;
    Ok(())
}

/// Update the progress array of a task.
pub async fn update_progress(db: &DatabaseConnection, id: i64, progress: &Value) -> AppResult<()> {
    let m = task::Entity::find_by_id(id)
        .one(db)
        .await?
        .expect("task must exist");
    let mut am: task::ActiveModel = m.into();
    am.progress = Set(progress.clone());
    am.update(db).await?;
    Ok(())
}

/// Mark task as completed with result.
pub async fn mark_completed(db: &DatabaseConnection, id: i64, result: &Value) -> AppResult<()> {
    let m = task::Entity::find_by_id(id)
        .one(db)
        .await?
        .expect("task must exist");
    let mut am: task::ActiveModel = m.into();
    am.status = Set("completed".to_string());
    am.result = Set(Some(result.clone()));
    am.completed_at = Set(Some(Utc::now().into()));
    am.update(db).await?;
    Ok(())
}

/// Mark task as failed with error message.
pub async fn mark_failed(db: &DatabaseConnection, id: i64, error: &str) -> AppResult<()> {
    let m = task::Entity::find_by_id(id)
        .one(db)
        .await?
        .expect("task must exist");
    let mut am: task::ActiveModel = m.into();
    am.status = Set("failed".to_string());
    am.error = Set(Some(error.to_string()));
    am.completed_at = Set(Some(Utc::now().into()));
    am.update(db).await?;
    Ok(())
}

/// Check if a task has been marked for cancellation.
pub async fn is_cancelled(db: &DatabaseConnection, id: i64) -> bool {
    matches!(find_by_id_system(db, id).await, Ok(Some(t)) if t.status == "cancelled")
}

/// Mark a task as cancelled. Safe to call multiple times.
pub async fn mark_cancelled(db: &DatabaseConnection, id: i64) -> AppResult<bool> {
    let m = match task::Entity::find_by_id(id).one(db).await? {
        Some(m) => m,
        None => return Ok(false),
    };
    // Only cancel running/pending tasks; do not touch completed/failed.
    if m.status == "completed" || m.status == "failed" {
        return Ok(false);
    }
    let mut am: task::ActiveModel = m.into();
    am.status = Set("cancelled".to_string());
    am.completed_at = Set(Some(Utc::now().into()));
    am.update(db).await?;
    Ok(true)
}
