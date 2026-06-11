use crate::{
    entity::{problem, problem_tag, tag},
    error::AppResult,
    problem::{model::*, repo},
    state::AppState,
};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

pub struct ProblemService<'a> {
    pub state: &'a AppState,
}

impl<'a> ProblemService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(&self, user_id: i64, tag_id: Option<i64>) -> AppResult<Vec<ProblemResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id, tag_id).await?;
        Ok(rows
            .into_iter()
            .map(|(p, tags)| to_resp(p, tags.into_iter().map(|t| t.id).collect()))
            .collect())
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<ProblemResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(crate::error::AppError::NotFound)?;
        let tags = tags_for_problem(&self.state.db, row.id).await?;
        Ok(to_resp(row, tags))
    }

    pub async fn create(&self, user_id: i64, req: CreateProblemReq) -> AppResult<ProblemResp> {
        let row = repo::insert(
            &self.state.db,
            user_id,
            &req.source,
            req.external_id.as_deref(),
            &req.title,
            req.url.as_deref(),
            req.difficulty,
            req.statement.as_deref(),
            &req.tag_ids,
        )
        .await?;
        Ok(to_resp(row, req.tag_ids))
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        req: UpdateProblemReq,
    ) -> AppResult<ProblemResp> {
        let row = repo::update(
            &self.state.db,
            user_id,
            id,
            req.source.as_deref(),
            req.external_id.as_deref(),
            req.title.as_deref(),
            req.url.as_deref(),
            req.difficulty,
            req.statement.as_deref(),
        )
        .await?
        .ok_or(crate::error::AppError::NotFound)?;

        if let Some(tag_ids) = req.tag_ids {
            repo::replace_tags(&self.state.db, id, &tag_ids).await?;
            Ok(to_resp(row, tag_ids))
        } else {
            let tags = tags_for_problem(&self.state.db, id).await?;
            Ok(to_resp(row, tags))
        }
    }

    pub async fn delete(&self, user_id: i64, id: i64) -> AppResult<()> {
        if !repo::delete(&self.state.db, user_id, id).await? {
            return Err(crate::error::AppError::NotFound);
        }
        Ok(())
    }
}

async fn tags_for_problem(
    db: &sea_orm::DatabaseConnection,
    problem_id: i64,
) -> AppResult<Vec<i64>> {
    let tags = tag::Entity::find()
        .join(JoinType::InnerJoin, tag::Relation::ProblemTag.def())
        .join(JoinType::InnerJoin, problem_tag::Relation::Problem.def())
        .filter(problem_tag::Column::ProblemId.eq(problem_id))
        .all(db)
        .await?;
    Ok(tags.into_iter().map(|t| t.id).collect())
}

fn to_resp(row: problem::Model, tag_ids: Vec<i64>) -> ProblemResp {
    ProblemResp {
        id: row.id,
        user_id: row.user_id,
        source: row.source,
        external_id: row.external_id,
        title: row.title,
        url: row.url,
        difficulty: row.difficulty,
        statement: row.statement,
        tag_ids,
        created_at: row.created_at.naive_utc().and_utc(),
        updated_at: row.updated_at.naive_utc().and_utc(),
    }
}
