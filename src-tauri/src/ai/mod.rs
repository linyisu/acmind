use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub knowledge_points: Vec<String>,
    pub error_type: String,
    pub root_cause: String,
    pub fix_summary: String,
    pub suggestions: Vec<String>,
}

/// Analyze a problem's WA and AC codes against the statement and solutions.
/// This is a placeholder that returns a mock analysis.
/// In production, this will call an LLM API (OpenAI / Claude / Gemini).
pub async fn analyze_problem(
    _statement: &str,
    _wa_code: &str,
    _ac_code: &str,
    _solution: &str,
) -> Result<AnalysisResult, AppError> {
    // TODO: Integrate with LLM API
    // For now, return a placeholder analysis
    Ok(AnalysisResult {
        knowledge_points: vec!["待分析".to_string()],
        error_type: "logic".to_string(),
        root_cause: "请配置 AI API 后使用自动分析功能".to_string(),
        fix_summary: "请配置 AI API 后使用自动分析功能".to_string(),
        suggestions: vec![
            "请在 Settings 页面配置 AI API Key".to_string(),
        ],
    })
}

/// Generate a training report using AI.
/// Placeholder that returns a markdown report template.
pub async fn generate_ai_report(
    _dashboard_stats: &str,
    _error_stats: &str,
    _period: &str,
) -> Result<String, AppError> {
    // TODO: Integrate with LLM API
    Ok("# AI 训练报告\n\n请配置 AI API 后使用自动报告生成功能。".to_string())
}
