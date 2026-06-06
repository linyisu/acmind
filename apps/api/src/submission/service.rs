use crate::{
    error::{AppError, AppResult},
    state::AppState,
    submission::{model::*, repo},
};

pub struct SubmissionService<'a> {
    pub state: &'a AppState,
}

impl<'a> SubmissionService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(
        &self,
        user_id: i64,
        problem_id: Option<i64>,
    ) -> AppResult<Vec<SubmissionResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id, problem_id).await?;
        Ok(rows.into_iter().map(to_resp).collect())
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<SubmissionResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(to_resp(row))
    }

    pub async fn create(
        &self,
        user_id: i64,
        req: CreateSubmissionReq,
    ) -> AppResult<SubmissionResp> {
        if !repo::problem_belongs_to_user(&self.state.db, user_id, req.problem_id).await? {
            return Err(AppError::BadRequest(format!(
                "problem {} not found for user",
                req.problem_id
            )));
        }
        let row = repo::insert(
            &self.state.db,
            user_id,
            req.problem_id,
            &req.language,
            &req.code,
            req.verdict.as_db_str(),
            req.runtime_ms,
            req.memory_kb,
            req.notes.as_deref(),
        )
        .await?;
        Ok(to_resp(row))
    }
}

fn to_resp(row: SubmissionRow) -> SubmissionResp {
    SubmissionResp {
        id: row.id,
        user_id: row.user_id,
        problem_id: row.problem_id,
        language: row.language,
        code: row.code,
        verdict: row.verdict,
        runtime_ms: row.runtime_ms,
        memory_kb: row.memory_kb,
        notes: row.notes,
        submitted_at: row.submitted_at,
    }
}
