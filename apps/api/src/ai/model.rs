use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The structured result from LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub algorithm_type: String,
    pub sub_type: String,
    pub tags: Vec<String>,
    pub summary: String,
    #[serde(default)]
    pub template_snippet: Option<String>,
    #[serde(default)]
    pub error_analysis: Option<String>,
    #[serde(default)]
    pub suggested_difficulty: Option<i32>,
}

/// GET /api/v1/ai/analyses response item.
#[derive(Debug, Serialize)]
pub struct AnalysisResp {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub result: AnalysisResult,
    pub created_at: DateTime<Utc>,
}
