use crate::ai::provider::LlmProvider;
use crate::{
    ai::{
        context::{ClassificationBrief, SubmissionSummary},
        model::ExtractedError,
        parse::parse_llm_json,
        prompt::{build_error_prompt, SYSTEM_ERROR},
    },
    error::AppResult,
};
use serde::Deserialize;

pub struct ErrorAgent;

#[derive(Deserialize)]
struct ErrorsJson {
    errors: Vec<ExtractedError>,
}

impl ErrorAgent {
    pub async fn run(
        &self,
        llm: &dyn LlmProvider,
        submissions: &[SubmissionSummary],
        brief: &ClassificationBrief,
    ) -> AppResult<Vec<ExtractedError>> {
        let has_non_ac = submissions.iter().any(|s| s.verdict != "AC");
        if !has_non_ac {
            return Ok(vec![]);
        }

        let prompt = build_error_prompt(brief, submissions);
        let response = llm.chat(SYSTEM_ERROR, &prompt).await?;

        match parse_llm_json::<ErrorsJson>(&response) {
            Ok(json) => Ok(json.errors),
            Err(e) => {
                tracing::warn!("错误分析 JSON 解析失败: {e}");
                Ok(vec![])
            }
        }
    }
}
