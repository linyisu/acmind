use crate::{
    error::AppResult,
    knowledge::{model::*, repo},
    state::AppState,
};

pub struct KnowledgeService<'a> {
    pub state: &'a AppState,
}

impl<'a> KnowledgeService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(&self, user_id: i64) -> AppResult<Vec<KnowledgeResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let tag_ids = repo::fetch_knowledge_tag_ids(&self.state.db, r.id).await?;
            out.push(to_resp(r, tag_ids));
        }
        Ok(out)
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<KnowledgeResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(crate::error::AppError::NotFound)?;
        let tag_ids = repo::fetch_knowledge_tag_ids(&self.state.db, id).await?;
        Ok(to_resp(row, tag_ids))
    }

    pub async fn create(&self, user_id: i64, req: CreateKnowledgeReq) -> AppResult<KnowledgeResp> {
        let row = repo::insert(
            &self.state.db,
            user_id,
            req.problem_id,
            req.kind.as_db_str(),
            &req.title,
            &req.content,
        )
        .await?;
        for tag_id in &req.tag_ids {
            repo::link_knowledge_tag(&self.state.db, row.id, *tag_id).await?;
        }
        Ok(to_resp(row, req.tag_ids))
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        req: UpdateKnowledgeReq,
    ) -> AppResult<KnowledgeResp> {
        let row = repo::update(
            &self.state.db,
            user_id,
            id,
            req.problem_id,
            req.kind.map(|k| k.as_db_str()),
            req.title.as_deref(),
            req.content.as_deref(),
        )
        .await?
        .ok_or(crate::error::AppError::NotFound)?;
        if let Some(tag_ids) = req.tag_ids {
            repo::replace_knowledge_tags(&self.state.db, id, &tag_ids).await?;
            Ok(to_resp(row, tag_ids))
        } else {
            let tag_ids = repo::fetch_knowledge_tag_ids(&self.state.db, id).await?;
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

fn to_resp(row: KnowledgeRow, tag_ids: Vec<i64>) -> KnowledgeResp {
    KnowledgeResp {
        id: row.id,
        user_id: row.user_id,
        problem_id: row.problem_id,
        kind: row.kind,
        title: row.title,
        content: row.content,
        tag_ids,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
