use crate::{
    entity::{knowledge, knowledge_tag, tag},
    error::{AppError, AppResult},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    QueryOrder, Set,
};

pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: Option<i64>,
    kind: &str,
    title: &str,
    content: &str,
    tag_ids: &[i64],
) -> AppResult<knowledge::Model> {
    let now = Utc::now();
    let am = knowledge::ActiveModel {
        user_id: Set(user_id),
        problem_id: Set(problem_id),
        kind: Set(kind.to_string()),
        title: Set(title.to_string()),
        content: Set(content.to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    let row = am.insert(db).await?;

    for tag_id in tag_ids {
        knowledge_tag::ActiveModel {
            knowledge_id: Set(row.id),
            tag_id: Set(*tag_id),
        }
        .insert(db)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotInserted => AppError::Internal("link tag".into()),
            other => AppError::from(other),
        })?;
    }
    Ok(row)
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<knowledge::Model>> {
    Ok(knowledge::Entity::find_by_id(id)
        .filter(knowledge::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
) -> AppResult<Vec<(knowledge::Model, Vec<tag::Model>)>> {
    let rows = knowledge::Entity::find()
        .filter(knowledge::Column::UserId.eq(user_id))
        .order_by_desc(knowledge::Column::Id)
        .all(db)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for k in rows {
        let tags = k.find_related(tag::Entity).all(db).await?;
        out.push((k, tags));
    }
    Ok(out)
}

pub async fn update(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
    problem_id: Option<i64>,
    kind: Option<&str>,
    title: Option<&str>,
    content: Option<&str>,
) -> AppResult<Option<knowledge::Model>> {
    let existing = match find_by_id(db, user_id, id).await? {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut am: knowledge::ActiveModel = existing.into();
    if let Some(p) = problem_id { am.problem_id = Set(Some(p)); }
    if let Some(k) = kind { am.kind = Set(k.to_string()); }
    if let Some(t) = title { am.title = Set(t.to_string()); }
    if let Some(c) = content { am.content = Set(c.to_string()); }
    am.updated_at = Set(Utc::now().into());

    Ok(Some(am.update(db).await?))
}

pub async fn replace_tags(
    db: &DatabaseConnection,
    knowledge_id: i64,
    tag_ids: &[i64],
) -> AppResult<()> {
    knowledge_tag::Entity::delete_many()
        .filter(knowledge_tag::Column::KnowledgeId.eq(knowledge_id))
        .exec(db)
        .await?;
    for tag_id in tag_ids {
        knowledge_tag::ActiveModel {
            knowledge_id: Set(knowledge_id),
            tag_id: Set(*tag_id),
        }
        .insert(db)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotInserted => AppError::Internal("link tag".into()),
            other => AppError::from(other),
        })?;
    }
    Ok(())
}

pub async fn list_by_problem_id(
    db: &DatabaseConnection,
    user_id: i64,
    problem_id: i64,
) -> AppResult<Vec<knowledge::Model>> {
    Ok(knowledge::Entity::find()
        .filter(knowledge::Column::UserId.eq(user_id))
        .filter(knowledge::Column::ProblemId.eq(problem_id))
        .order_by_desc(knowledge::Column::UpdatedAt)
        .all(db)
        .await?)
}

pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let res = knowledge::Entity::delete_many()
        .filter(knowledge::Column::Id.eq(id))
        .filter(knowledge::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(res.rows_affected > 0)
}
