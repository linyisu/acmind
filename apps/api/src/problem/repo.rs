use crate::{
    entity::{problem, problem_tag, tag},
    error::{AppError, AppResult},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, ModelTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

/// Insert a problem and link tags. Returns the row.
#[allow(clippy::too_many_arguments)]
pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    source: &str,
    external_id: Option<&str>,
    title: &str,
    url: Option<&str>,
    difficulty: Option<i32>,
    statement: Option<&str>,
    tag_ids: &[i64],
) -> AppResult<problem::Model> {
    let now = Utc::now();
    let am = problem::ActiveModel {
        user_id: Set(user_id),
        source: Set(source.to_string()),
        external_id: Set(external_id.map(str::to_string)),
        title: Set(title.to_string()),
        url: Set(url.map(str::to_string)),
        difficulty: Set(difficulty),
        statement: Set(statement.map(str::to_string)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    let row = am.insert(db).await?;

    for tag_id in tag_ids {
        problem_tag::ActiveModel {
            problem_id: Set(row.id),
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
) -> AppResult<Option<problem::Model>> {
    Ok(problem::Entity::find_by_id(id)
        .filter(problem::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

/// List problems for a user, optionally filtered to a single tag.
/// Returns the problem + its tags (M2M via problem_tag).
pub async fn list_by_user(
    db: &DatabaseConnection,
    user_id: i64,
    tag_id: Option<i64>,
) -> AppResult<Vec<(problem::Model, Vec<tag::Model>)>> {
    let mut q = problem::Entity::find()
        .filter(problem::Column::UserId.eq(user_id))
        .order_by_desc(problem::Column::Id);

    if let Some(t) = tag_id {
        // M2M join through problem_tag -> tag.
        q = q
            .join(JoinType::InnerJoin, problem::Relation::ProblemTag.def())
            .join(JoinType::InnerJoin, problem_tag::Relation::Tag.def())
            .filter(problem_tag::Column::TagId.eq(t));
    }

    let problems = q.distinct().all(db).await?;

    let mut out = Vec::with_capacity(problems.len());
    for p in problems {
        let tags = p.find_related(tag::Entity).all(db).await?;
        out.push((p, tags));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
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
) -> AppResult<Option<problem::Model>> {
    let existing = match find_by_id(db, user_id, id).await? {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut am: problem::ActiveModel = existing.into();
    if let Some(s) = source {
        am.source = Set(s.to_string());
    }
    if let Some(e) = external_id {
        am.external_id = Set(Some(e.to_string()));
    }
    if let Some(t) = title {
        am.title = Set(t.to_string());
    }
    if let Some(u) = url {
        am.url = Set(Some(u.to_string()));
    }
    if let Some(d) = difficulty {
        am.difficulty = Set(Some(d));
    }
    if let Some(s) = statement {
        am.statement = Set(Some(s.to_string()));
    }
    am.updated_at = Set(Utc::now().into());

    Ok(Some(am.update(db).await?))
}

pub async fn replace_tags(
    db: &DatabaseConnection,
    problem_id: i64,
    tag_ids: &[i64],
) -> AppResult<()> {
    problem_tag::Entity::delete_many()
        .filter(problem_tag::Column::ProblemId.eq(problem_id))
        .exec(db)
        .await?;
    for tag_id in tag_ids {
        problem_tag::ActiveModel {
            problem_id: Set(problem_id),
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

pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let res = problem::Entity::delete_many()
        .filter(problem::Column::Id.eq(id))
        .filter(problem::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(res.rows_affected > 0)
}
