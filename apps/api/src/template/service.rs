use crate::{
    error::{AppError, AppResult},
    state::AppState,
    template::{model::*, repo},
};

pub struct TemplateService<'a> {
    pub state: &'a AppState,
}

impl<'a> TemplateService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(
        &self,
        user_id: i64,
        query: &ListTemplatesQuery,
    ) -> AppResult<Vec<TemplateResp>> {
        let rows = repo::list(&self.state.db, user_id, query).await?;
        let mut out = Vec::with_capacity(rows.len());
        for t in rows {
            let tag_ids = repo::tag_ids(&self.state.db, t.id).await?;
            let problem_ids = repo::problem_ids(&self.state.db, t.id).await?;
            out.push(to_resp(t, tag_ids, problem_ids));
        }
        Ok(out)
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<TemplateResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(AppError::NotFound)?;
        let tag_ids = repo::tag_ids(&self.state.db, id).await?;
        let problem_ids = repo::problem_ids(&self.state.db, id).await?;
        Ok(to_resp(row, tag_ids, problem_ids))
    }

    pub async fn create(&self, user_id: i64, req: CreateTemplateReq) -> AppResult<TemplateResp> {
        let row = repo::insert(
            &self.state.db,
            user_id,
            &req.title,
            req.category.as_db_str(),
            &req.language,
            &req.code,
            &req.description,
            req.summary.as_deref().unwrap_or(""),
            req.time_complexity.as_deref(),
            req.space_complexity.as_deref(),
            "manual",
            None,
            req.difficulty,
            &req.tag_ids,
            &req.problem_ids,
        )
        .await?;
        Ok(to_resp(row, req.tag_ids, req.problem_ids))
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        req: UpdateTemplateReq,
    ) -> AppResult<TemplateResp> {
        let row = repo::update(
            &self.state.db,
            user_id,
            id,
            req.title.as_deref(),
            req.category.map(|c| c.as_db_str()),
            req.language.as_deref(),
            req.code.as_deref(),
            req.description.as_deref(),
            req.summary.as_deref(),
            req.time_complexity.as_deref(),
            req.space_complexity.as_deref(),
            req.difficulty,
        )
        .await?
        .ok_or(AppError::NotFound)?;

        if let Some(tag_ids) = req.tag_ids {
            repo::replace_tags(&self.state.db, id, &tag_ids).await?;
        }

        let tag_ids = repo::tag_ids(&self.state.db, id).await?;
        let problem_ids = repo::problem_ids(&self.state.db, id).await?;
        Ok(to_resp(row, tag_ids, problem_ids))
    }

    pub async fn delete(&self, user_id: i64, id: i64) -> AppResult<()> {
        if !repo::delete(&self.state.db, user_id, id).await? {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn link_problem(
        &self,
        user_id: i64,
        template_id: i64,
        problem_id: i64,
    ) -> AppResult<()> {
        // Verify template belongs to user
        repo::find_by_id(&self.state.db, user_id, template_id)
            .await?
            .ok_or(AppError::NotFound)?;
        repo::link_problem(&self.state.db, template_id, problem_id).await?;
        Ok(())
    }

    pub async fn unlink_problem(
        &self,
        user_id: i64,
        template_id: i64,
        problem_id: i64,
    ) -> AppResult<()> {
        repo::find_by_id(&self.state.db, user_id, template_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if !repo::unlink_problem(&self.state.db, template_id, problem_id).await? {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn stats(&self, user_id: i64) -> AppResult<TemplateStats> {
        let total = repo::total_count(&self.state.db, user_id).await?;
        let by_category = repo::stats_by_category(&self.state.db, user_id).await?;
        let by_language = repo::stats_by_language(&self.state.db, user_id).await?;
        Ok(TemplateStats {
            total,
            by_category,
            by_language,
        })
    }
}

fn to_resp(
    row: crate::entity::template::Model,
    tag_ids: Vec<i64>,
    problem_ids: Vec<i64>,
) -> TemplateResp {
    TemplateResp {
        id: row.id,
        user_id: row.user_id,
        title: row.title,
        category: row.category,
        language: row.language,
        code: row.code,
        description: row.description,
        summary: row.summary,
        time_complexity: row.time_complexity,
        space_complexity: row.space_complexity,
        source: row.source,
        source_problem_id: row.source_problem_id,
        difficulty: row.difficulty,
        usage_count: row.usage_count,
        tag_ids,
        problem_ids,
        created_at: row.created_at.naive_utc().and_utc(),
        updated_at: row.updated_at.naive_utc().and_utc(),
    }
}
