use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeKind {
    Template,
    Technique,
    Note,
    Snippet,
}

impl KnowledgeKind {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            KnowledgeKind::Template => "template",
            KnowledgeKind::Technique => "technique",
            KnowledgeKind::Note => "note",
            KnowledgeKind::Snippet => "snippet",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeReq {
    pub problem_id: Option<i64>,
    pub kind: KnowledgeKind,
    pub title: String,
    pub content: String,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKnowledgeReq {
    pub problem_id: Option<i64>,
    pub kind: Option<KnowledgeKind>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeResp {
    pub id: i64,
    pub user_id: i64,
    pub problem_id: Option<i64>,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub tag_ids: Vec<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
