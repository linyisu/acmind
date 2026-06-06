use crate::{error::AppResult, knowledge::model::KnowledgeRow};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: Option<i64>,
    kind: &str,
    title: &str,
    content: &str,
) -> AppResult<KnowledgeRow> {
    let now = Utc::now();
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"INSERT INTO knowledge (user_id, problem_id, kind, title, content, created_at, updated_at)
               VALUES ({}, {}, '{}', '{}', '{}', '{}', '{}')
               RETURNING id, user_id, problem_id, kind, title, content, created_at, updated_at"#,
            user_id,
            opt_i64(problem_id),
            esc(kind),
            esc(title),
            esc(content),
            now.to_rfc3339(),
            now.to_rfc3339(),
        ),
    );
    let row = db
        .query_one(stmt)
        .await?
        .ok_or_else(|| crate::error::AppError::Internal("knowledge insert returned no row".into()))?;
    row_to_knowledge(row).ok_or_else(|| crate::error::AppError::Internal("knowledge row parse".into()))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<KnowledgeRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, problem_id, kind, title, content, created_at, updated_at
               FROM knowledge WHERE id = {} AND user_id = {}"#,
            id, user_id
        ),
    );
    let row = db.query_one(stmt).await?;
    Ok(row.and_then(row_to_knowledge))
}

pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
) -> AppResult<Vec<KnowledgeRow>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"SELECT id, user_id, problem_id, kind, title, content, created_at, updated_at
               FROM knowledge WHERE user_id = {} ORDER BY id DESC"#,
            user_id
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(row_to_knowledge).collect())
}

pub async fn update(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
    problem_id: Option<i64>,
    kind: Option<&str>,
    title: Option<&str>,
    content: Option<&str>,
) -> AppResult<Option<KnowledgeRow>> {
    let mut sets: Vec<String> = vec![];
    if let Some(p) = problem_id { sets.push(format!("problem_id = {}", p)); }
    if let Some(k) = kind { sets.push(format!("kind = '{}'", esc(k))); }
    if let Some(t) = title { sets.push(format!("title = '{}'", esc(t))); }
    if let Some(c) = content { sets.push(format!("content = '{}'", esc(c))); }
    if sets.is_empty() { return find_by_id(db, user_id, id).await; }
    let now = Utc::now();
    sets.push(format!("updated_at = '{}'", now.to_rfc3339()));
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            r#"UPDATE knowledge SET {} WHERE id = {} AND user_id = {}
               RETURNING id, user_id, problem_id, kind, title, content, created_at, updated_at"#,
            sets.join(", "), id, user_id
        ),
    );
    let row = db.query_one(stmt).await?;
    Ok(row.and_then(row_to_knowledge))
}

pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!("DELETE FROM knowledge WHERE id = {} AND user_id = {}", id, user_id),
    );
    let res = db.execute(stmt).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn link_knowledge_tag(
    db: &DatabaseConnection,
    knowledge_id: i64,
    tag_id: i64,
) -> AppResult<()> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "INSERT INTO knowledge_tag (knowledge_id, tag_id) VALUES ({}, {}) ON CONFLICT DO NOTHING",
            knowledge_id, tag_id
        ),
    );
    db.execute(stmt).await?;
    Ok(())
}

pub async fn replace_knowledge_tags(
    db: &DatabaseConnection,
    knowledge_id: i64,
    tag_ids: &[i64],
) -> AppResult<()> {
    let del = Statement::from_string(
        DbBackend::Postgres,
        format!("DELETE FROM knowledge_tag WHERE knowledge_id = {}", knowledge_id),
    );
    db.execute(del).await?;
    for tag_id in tag_ids {
        link_knowledge_tag(db, knowledge_id, *tag_id).await?;
    }
    Ok(())
}

pub async fn fetch_knowledge_tag_ids(
    db: &DatabaseConnection,
    knowledge_id: i64,
) -> AppResult<Vec<i64>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT tag_id FROM knowledge_tag WHERE knowledge_id = {} ORDER BY tag_id",
            knowledge_id
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get_by::<i64, _>("tag_id").ok()).collect())
}

pub fn row_to_knowledge(row: sea_orm::QueryResult) -> Option<KnowledgeRow> {
    Some(KnowledgeRow {
        id: row.try_get_by::<i64, _>("id").ok()?,
        user_id: row.try_get_by::<i64, _>("user_id").ok()?,
        problem_id: row.try_get_by::<Option<i64>, _>("problem_id").ok()?,
        kind: row.try_get_by::<String, _>("kind").ok()?,
        title: row.try_get_by::<String, _>("title").ok()?,
        content: row.try_get_by::<String, _>("content").ok()?,
        created_at: row.try_get_by::<chrono::DateTime<chrono::Utc>, _>("created_at").ok()?,
        updated_at: row.try_get_by::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok()?,
    })
}

fn esc(s: &str) -> String { s.replace('\'', "''") }
fn opt_i64(v: Option<i64>) -> String {
    match v { Some(n) => n.to_string(), None => "NULL".to_string() }
}
