use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateProblemReq {
    pub source: String,
    pub external_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub statement: Option<String>,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProblemReq {
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub statement: Option<String>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProblemResp {
    pub id: i64,
    pub user_id: i64,
    pub source: String,
    pub external_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub statement: Option<String>,
    pub tag_ids: Vec<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProblemRow {
    pub id: i64,
    pub user_id: i64,
    pub source: String,
    pub external_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub statement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
