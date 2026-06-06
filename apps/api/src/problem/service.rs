use crate::{
    error::AppResult,
    problem::{model::*, repo},
    state::AppState,
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

pub struct ProblemService<'a> {
    pub state: &'a AppState,
}

impl<'a> ProblemService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(
        &self,
        user_id: i64,
        tag_id: Option<i64>,
    ) -> AppResult<Vec<ProblemResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id, tag_id).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let tag_ids = fetch_problem_tag_ids(&self.state.db, r.id).await?;
            out.push(to_resp(r, tag_ids));
        }
        Ok(out)
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<ProblemResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(crate::error::AppError::NotFound)?;
        let tag_ids = fetch_problem_tag_ids(&self.state.db, id).await?;
        Ok(to_resp(row, tag_ids))
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
        )
        .await?;
        for tag_id in &req.tag_ids {
            link_problem_tag(&self.state.db, row.id, *tag_id).await?;
        }
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
            replace_problem_tags(&self.state.db, id, &tag_ids).await?;
            let resp = to_resp(row, tag_ids);
            Ok(resp)
        } else {
            let tag_ids = fetch_problem_tag_ids(&self.state.db, id).await?;
            Ok(to_resp(row, tag_ids))
        }
    }

    pub async fn delete(&self, user_id: i64, id: i64) -> AppResult<()> {
        if !repo::delete(&self.state.db, user_id, id).await? {
            return Err(crate::error::AppError::NotFound);
        }
        Ok(())
    }
}

fn to_resp(row: ProblemRow, tag_ids: Vec<i64>) -> ProblemResp {
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
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

async fn fetch_problem_tag_ids(
    db: &sea_orm::DatabaseConnection,
    problem_id: i64,
) -> AppResult<Vec<i64>> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT tag_id FROM problem_tag WHERE problem_id = {} ORDER BY tag_id",
            problem_id
        ),
    );
    let rows = db.query_all(stmt).await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| r.try_get_by::<i64, _>("tag_id").ok())
        .collect())
}

pub async fn link_problem_tag(
    db: &sea_orm::DatabaseConnection,
    problem_id: i64,
    tag_id: i64,
) -> AppResult<()> {
    let stmt = Statement::from_string(
        DbBackend::Postgres,
        format!(
            "INSERT INTO problem_tag (problem_id, tag_id) VALUES ({}, {}) ON CONFLICT DO NOTHING",
            problem_id, tag_id
        ),
    );
    db.execute(stmt).await?;
    Ok(())
}

pub async fn replace_problem_tags(
    db: &sea_orm::DatabaseConnection,
    problem_id: i64,
    tag_ids: &[i64],
) -> AppResult<()> {
    let del = Statement::from_string(
        DbBackend::Postgres,
        format!("DELETE FROM problem_tag WHERE problem_id = {}", problem_id),
    );
    db.execute(del).await?;
    for tag_id in tag_ids {
        link_problem_tag(db, problem_id, *tag_id).await?;
    }
    Ok(())
}
