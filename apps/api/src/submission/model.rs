use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    AC,
    WA,
    TLE,
    RE,
    CE,
    Pending,
}

impl Verdict {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Verdict::AC => "AC",
            Verdict::WA => "WA",
            Verdict::TLE => "TLE",
            Verdict::RE => "RE",
            Verdict::CE => "CE",
            Verdict::Pending => "PENDING",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionReq {
    pub problem_id: i64,
    pub language: String,
    pub code: String,
    pub verdict: Verdict,
    pub runtime_ms: Option<i32>,
    pub memory_kb: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResp {
    pub id: i64,
    pub user_id: i64,
    pub problem_id: i64,
    pub language: String,
    pub code: String,
    pub verdict: String,
    pub runtime_ms: Option<i32>,
    pub memory_kb: Option<i32>,
    pub notes: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SubmissionRow {
    pub id: i64,
    pub user_id: i64,
    pub problem_id: i64,
    pub language: String,
    pub code: String,
    pub verdict: String,
    pub runtime_ms: Option<i32>,
    pub memory_kb: Option<i32>,
    pub notes: Option<String>,
    pub submitted_at: DateTime<Utc>,
}
