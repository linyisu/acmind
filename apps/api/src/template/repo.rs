use crate::{
    entity::{tag, template, template_problem, template_tag},
    error::{AppError, AppResult},
    template::model::{CategoryCount, LanguageCount, ListTemplatesQuery},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

/// Insert a new template with tags and problem associations.
#[allow(clippy::too_many_arguments)]
pub async fn insert(
    db: &DatabaseConnection,
    user_id: i64,
    title: &str,
    category: &str,
    language: &str,
    code: &str,
    description: &str,
    summary: &str,
    time_complexity: Option<&str>,
    space_complexity: Option<&str>,
    source: &str,
    source_problem_id: Option<i64>,
    difficulty: Option<i32>,
    tag_ids: &[i64],
    problem_ids: &[i64],
) -> AppResult<template::Model> {
    let now = Utc::now();
    let am = template::ActiveModel {
        user_id: Set(user_id),
        title: Set(title.to_string()),
        category: Set(category.to_string()),
        language: Set(language.to_string()),
        code: Set(code.to_string()),
        description: Set(description.to_string()),
        summary: Set(summary.to_string()),
        time_complexity: Set(time_complexity.map(|s| s.to_string())),
        space_complexity: Set(space_complexity.map(|s| s.to_string())),
        source: Set(source.to_string()),
        source_problem_id: Set(source_problem_id),
        difficulty: Set(difficulty),
        usage_count: Set(problem_ids.len() as i32),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    let row = am.insert(db).await?;

    for tag_id in tag_ids {
        template_tag::ActiveModel {
            template_id: Set(row.id),
            tag_id: Set(*tag_id),
        }
        .insert(db)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotInserted => AppError::Internal("link tag".into()),
            other => AppError::from(other),
        })?;
    }

    for problem_id in problem_ids {
        template_problem::ActiveModel {
            template_id: Set(row.id),
            problem_id: Set(*problem_id),
            created_at: Set(now.into()),
        }
        .insert(db)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotInserted => AppError::Internal("link problem".into()),
            other => AppError::from(other),
        })?;
    }

    Ok(row)
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
) -> AppResult<Option<template::Model>> {
    Ok(template::Entity::find_by_id(id)
        .filter(template::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

/// List templates with optional filters.
pub async fn list(
    db: &DatabaseConnection,
    user_id: i64,
    query: &ListTemplatesQuery,
) -> AppResult<Vec<template::Model>> {
    let mut sel = template::Entity::find().filter(template::Column::UserId.eq(user_id));

    if let Some(ref cat) = query.category {
        sel = sel.filter(template::Column::Category.eq(cat.as_db_str()));
    }
    if let Some(ref lang) = query.language {
        sel = sel.filter(template::Column::Language.eq(lang.as_str()));
    }
    if let Some(ref q) = query.search {
        let pattern = format!("%{}%", q);
        sel = sel.filter(
            template::Column::Title
                .contains(&pattern)
                .or(template::Column::Description.contains(&pattern))
                .or(template::Column::Code.contains(&pattern)),
        );
    }
    // Filter by tag via join
    if let Some(tag_id) = query.tag_id {
        sel = sel
            .join(JoinType::InnerJoin, template::Relation::TemplateTag.def())
            .filter(template_tag::Column::TagId.eq(tag_id));
    }
    // Filter by problem via join
    if let Some(problem_id) = query.problem_id {
        sel = sel
            .join(
                JoinType::InnerJoin,
                template::Relation::TemplateProblem.def(),
            )
            .filter(template_problem::Column::ProblemId.eq(problem_id));
    }

    sel = match query.sort.as_deref() {
        Some("usage") => sel.order_by_desc(template::Column::UsageCount),
        Some("title") => sel.order_by_asc(template::Column::Title),
        _ => sel.order_by_desc(template::Column::Id),
    };

    Ok(sel.all(db).await?)
}

/// Get tag IDs for a template.
pub async fn tag_ids(db: &DatabaseConnection, template_id: i64) -> AppResult<Vec<i64>> {
    let tpl = template::Entity::find_by_id(template_id).one(db).await?;
    let Some(tpl) = tpl else { return Ok(vec![]) };
    let tags = tpl.find_related(tag::Entity).all(db).await?;
    Ok(tags.into_iter().map(|t| t.id).collect())
}

/// Get problem IDs for a template.
pub async fn problem_ids(db: &DatabaseConnection, template_id: i64) -> AppResult<Vec<i64>> {
    let rows = template_problem::Entity::find()
        .filter(template_problem::Column::TemplateId.eq(template_id))
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|r| r.problem_id).collect())
}

/// Update a template (partial fields).
#[allow(clippy::too_many_arguments)]
pub async fn update(
    db: &DatabaseConnection,
    user_id: i64,
    id: i64,
    title: Option<&str>,
    category: Option<&str>,
    language: Option<&str>,
    code: Option<&str>,
    description: Option<&str>,
    summary: Option<&str>,
    time_complexity: Option<&str>,
    space_complexity: Option<&str>,
    difficulty: Option<i32>,
) -> AppResult<Option<template::Model>> {
    let existing = match find_by_id(db, user_id, id).await? {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut am: template::ActiveModel = existing.into();
    if let Some(v) = title {
        am.title = Set(v.to_string());
    }
    if let Some(v) = category {
        am.category = Set(v.to_string());
    }
    if let Some(v) = language {
        am.language = Set(v.to_string());
    }
    if let Some(v) = code {
        am.code = Set(v.to_string());
    }
    if let Some(v) = description {
        am.description = Set(v.to_string());
    }
    if let Some(v) = summary {
        am.summary = Set(v.to_string());
    }
    if let Some(v) = time_complexity {
        am.time_complexity = Set(Some(v.to_string()));
    }
    if let Some(v) = space_complexity {
        am.space_complexity = Set(Some(v.to_string()));
    }
    if let Some(v) = difficulty {
        am.difficulty = Set(Some(v));
    }
    am.updated_at = Set(Utc::now().into());

    Ok(Some(am.update(db).await?))
}

/// Replace all tags for a template.
pub async fn replace_tags(
    db: &DatabaseConnection,
    template_id: i64,
    tag_ids: &[i64],
) -> AppResult<()> {
    template_tag::Entity::delete_many()
        .filter(template_tag::Column::TemplateId.eq(template_id))
        .exec(db)
        .await?;
    for tag_id in tag_ids {
        template_tag::ActiveModel {
            template_id: Set(template_id),
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

/// Link a problem to a template (idempotent).
pub async fn link_problem(
    db: &DatabaseConnection,
    template_id: i64,
    problem_id: i64,
) -> AppResult<bool> {
    // Check if already linked
    let existing = template_problem::Entity::find()
        .filter(template_problem::Column::TemplateId.eq(template_id))
        .filter(template_problem::Column::ProblemId.eq(problem_id))
        .one(db)
        .await?;
    if existing.is_some() {
        return Ok(false); // already linked
    }

    template_problem::ActiveModel {
        template_id: Set(template_id),
        problem_id: Set(problem_id),
        created_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await?;

    // Update usage_count
    if let Some(t) = template::Entity::find_by_id(template_id).one(db).await? {
        let count = template_problem::Entity::find()
            .filter(template_problem::Column::TemplateId.eq(template_id))
            .count(db)
            .await?;
        let mut am: template::ActiveModel = t.into();
        am.usage_count = Set(count as i32);
        am.updated_at = Set(Utc::now().into());
        am.update(db).await?;
    }

    Ok(true)
}

/// Unlink a problem from a template.
pub async fn unlink_problem(
    db: &DatabaseConnection,
    template_id: i64,
    problem_id: i64,
) -> AppResult<bool> {
    let res = template_problem::Entity::delete_many()
        .filter(template_problem::Column::TemplateId.eq(template_id))
        .filter(template_problem::Column::ProblemId.eq(problem_id))
        .exec(db)
        .await?;

    if res.rows_affected > 0 {
        // Update usage_count
        if let Some(t) = template::Entity::find_by_id(template_id).one(db).await? {
            let count = template_problem::Entity::find()
                .filter(template_problem::Column::TemplateId.eq(template_id))
                .count(db)
                .await?;
            let mut am: template::ActiveModel = t.into();
            am.usage_count = Set(count as i32);
            am.updated_at = Set(Utc::now().into());
            am.update(db).await?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Delete a template (and cascade deletes tags/problems via FK).
pub async fn delete(db: &DatabaseConnection, user_id: i64, id: i64) -> AppResult<bool> {
    let res = template::Entity::delete_many()
        .filter(template::Column::Id.eq(id))
        .filter(template::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(res.rows_affected > 0)
}

/// Count templates by category for a user.
pub async fn stats_by_category(
    db: &DatabaseConnection,
    user_id: i64,
) -> AppResult<Vec<CategoryCount>> {
    let rows = template::Entity::find()
        .filter(template::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for r in rows {
        *map.entry(r.category).or_insert(0) += 1;
    }
    let mut result: Vec<CategoryCount> = map
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.count));
    Ok(result)
}

/// Count templates by language for a user.
pub async fn stats_by_language(
    db: &DatabaseConnection,
    user_id: i64,
) -> AppResult<Vec<LanguageCount>> {
    let rows = template::Entity::find()
        .filter(template::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for r in rows {
        *map.entry(r.language).or_insert(0) += 1;
    }
    let mut result: Vec<LanguageCount> = map
        .into_iter()
        .map(|(language, count)| LanguageCount { language, count })
        .collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.count));
    Ok(result)
}

/// Total template count for a user.
pub async fn total_count(db: &DatabaseConnection, user_id: i64) -> AppResult<i64> {
    let count = template::Entity::find()
        .filter(template::Column::UserId.eq(user_id))
        .count(db)
        .await?;
    Ok(count as i64)
}

/// Check if a template exists by (user_id, category, language, title) — used for dedup.
pub async fn exists_by_identity(
    db: &DatabaseConnection,
    user_id: i64,
    category: &str,
    language: &str,
    title: &str,
) -> AppResult<bool> {
    let count = template::Entity::find()
        .filter(template::Column::UserId.eq(user_id))
        .filter(template::Column::Category.eq(category))
        .filter(template::Column::Language.eq(language))
        .filter(template::Column::Title.eq(title))
        .count(db)
        .await?;
    Ok(count > 0)
}
