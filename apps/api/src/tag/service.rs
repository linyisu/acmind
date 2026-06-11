use crate::{
    error::{AppError, AppResult},
    state::AppState,
    tag::{model::*, repo},
};

pub struct TagService<'a> {
    pub state: &'a AppState,
}

impl<'a> TagService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(&self, user_id: i64) -> AppResult<Vec<TagResp>> {
        Ok(repo::list_by_user(&self.state.db, user_id)
            .await?
            .into_iter()
            .map(to_resp)
            .collect())
    }

    pub async fn create(&self, user_id: i64, name: &str) -> AppResult<TagResp> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("tag name cannot be empty".into()));
        }
        let row = repo::insert(&self.state.db, user_id, trimmed).await?;
        Ok(to_resp(row))
    }

    pub async fn delete(&self, user_id: i64, id: i64) -> AppResult<()> {
        if !repo::delete(&self.state.db, user_id, id).await? {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn validate_owned(&self, user_id: i64, tag_ids: &[i64]) -> AppResult<()> {
        for id in tag_ids {
            if !repo::exists_and_owned(&self.state.db, user_id, *id).await? {
                return Err(AppError::BadRequest(format!(
                    "tag {} not owned by user",
                    id
                )));
            }
        }
        Ok(())
    }
}

fn to_resp(row: TagRow) -> TagResp {
    TagResp {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
    }
}
