use serde::{Deserialize, Serialize};

/// A single VJudge submission item.
#[derive(Debug, Deserialize)]
pub struct VjudgeSubmissionItem {
    pub oj: String,
    pub prob_num: String,
    pub status: String,
    pub language: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub submit_time: Option<String>,
}

/// POST /api/v1/import/vjudge/problem-full
/// Imports a problem and all its submissions in one request.
#[derive(Debug, Deserialize)]
pub struct ImportProblemFullReq {
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
    pub submissions: Vec<VjudgeSubmissionItem>,
}

/// Response for the full import.
#[derive(Debug, Serialize)]
pub struct ImportProblemFullResp {
    pub problem_id: i64,
    pub submissions_imported: usize,
    pub submissions_skipped: usize,
    pub errors: Vec<String>,
}
