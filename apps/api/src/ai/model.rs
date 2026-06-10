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

/// POST /api/v1/ai/analyze-problem/{problem_id} response.
#[derive(Debug, Serialize)]
pub struct ProblemAnalysisResp {
    pub analysis: AnalysisResp,
    pub extracted_templates: usize,
    pub extracted_errors: usize,
    pub extracted_knowledge: usize,
    pub submissions_analyzed: usize,
    pub knowledge_merged: usize,
}

/// LLM extraction response types (used by AI agents).
#[derive(Debug, Deserialize)]
pub struct ExtractedTemplate {
    pub title: String,
    pub code: String,
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub time_complexity: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtractedError {
    pub title: String,
    pub description: String,
    pub fix_suggestion: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtractedKnowledge {
    pub title: String,
    pub content: String,
}
