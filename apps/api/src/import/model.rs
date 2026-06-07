use serde::{Deserialize, Serialize};

/// A single VJudge submission item from the browser extension.
#[derive(Debug, Deserialize)]
pub struct VjudgeSubmissionItem {
    pub oj: String,
    pub prob_num: String,
    pub status: String,
    pub language: String,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub submit_time: Option<String>,
}

/// POST /api/v1/import/vjudge/submissions
#[derive(Debug, Deserialize)]
pub struct ImportSubmissionsReq {
    pub username: String,
    pub items: Vec<VjudgeSubmissionItem>,
}

/// POST /api/v1/import/vjudge/problem
#[derive(Debug, Deserialize)]
pub struct ImportProblemReq {
    pub source_problem_id: String,
    pub oj: String,
    pub prob_num: String,
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub statement: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// POST /api/v1/import/vjudge/submission
#[derive(Debug, Deserialize)]
pub struct ImportSingleSubmissionReq {
    pub run_id: Option<String>,
    pub oj: String,
    pub prob_num: String,
    pub status: String,
    pub language: String,
    pub code: String,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub submit_time: Option<String>,
}

/// Response for import operations.
#[derive(Debug, Serialize)]
pub struct ImportResp {
    pub created: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// A single imported submission result.
#[derive(Debug, Serialize)]
pub struct ImportSubmissionResp {
    pub problem_id: i64,
    pub submission_id: i64,
}
