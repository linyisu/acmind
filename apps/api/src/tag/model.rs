use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTagReq {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TagResp {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TagRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
}
