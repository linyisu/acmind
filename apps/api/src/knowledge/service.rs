use crate::{
    entity::{knowledge, knowledge_tag, tag},
    error::AppResult,
    knowledge::{model::*, repo},
    state::AppState,
};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

pub struct KnowledgeService<'a> {
    pub state: &'a AppState,
}

impl<'a> KnowledgeService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list(&self, user_id: i64) -> AppResult<Vec<KnowledgeResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id).await?;
        Ok(rows
            .into_iter()
            .map(|(k, tags)| to_resp(k, tags.into_iter().map(|t| t.id).collect()))
            .collect())
    }

    pub async fn get(&self, user_id: i64, id: i64) -> AppResult<KnowledgeResp> {
        let row = repo::find_by_id(&self.state.db, user_id, id)
            .await?
            .ok_or(crate::error::AppError::NotFound)?;
        let tags = tags_for_knowledge(&self.state.db, id).await?;
        Ok(to_resp(row, tags))
    }

    pub async fn create(&self, user_id: i64, req: CreateKnowledgeReq) -> AppResult<KnowledgeResp> {
        let row = repo::insert(
            &self.state.db,
            user_id,
            req.problem_id,
            req.kind.as_db_str(),
            &req.title,
            &req.content,
            &req.tag_ids,
        )
        .await?;
        Ok(to_resp(row, req.tag_ids))
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        req: UpdateKnowledgeReq,
    ) -> AppResult<KnowledgeResp> {
        let problem_arg = req.problem_id;
        let row = repo::update(
            &self.state.db,
            user_id,
            id,
            problem_arg,
            req.kind.map(|k| k.as_db_str()),
            req.title.as_deref(),
            req.content.as_deref(),
        )
        .await?
        .ok_or(crate::error::AppError::NotFound)?;

        if let Some(tag_ids) = req.tag_ids {
            repo::replace_tags(&self.state.db, id, &tag_ids).await?;
            Ok(to_resp(row, tag_ids))
        } else {
            let tags = tags_for_knowledge(&self.state.db, id).await?;
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

async fn tags_for_knowledge(
    db: &sea_orm::DatabaseConnection,
    knowledge_id: i64,
) -> AppResult<Vec<i64>> {
    let tags = tag::Entity::find()
        .join(JoinType::InnerJoin, tag::Relation::KnowledgeTag.def())
        .join(JoinType::InnerJoin, knowledge_tag::Relation::Knowledge.def())
        .filter(knowledge_tag::Column::KnowledgeId.eq(knowledge_id))
        .all(db)
        .await?;
    Ok(tags.into_iter().map(|t| t.id).collect())
}

fn to_resp(row: knowledge::Model, tag_ids: Vec<i64>) -> KnowledgeResp {
    KnowledgeResp {
        id: row.id,
        user_id: row.user_id,
        problem_id: row.problem_id,
        kind: row.kind,
        title: row.title,
        content: row.content,
        tag_ids,
        created_at: row.created_at.naive_utc().and_utc(),
        updated_at: row.updated_at.naive_utc().and_utc(),
    }
}
