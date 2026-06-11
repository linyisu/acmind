use crate::ai::provider::LlmProvider;
use crate::{
    ai::{
        context::{ClassificationBrief, SubmissionSummary, TemplateBrief},
        model::ExtractedTemplate,
        parse::parse_llm_json,
        prompt::{build_template_prompt, SYSTEM_TEMPLATE},
    },
    error::AppResult,
};
use serde::Deserialize;

pub struct TemplateAgent;

#[derive(Deserialize)]
struct TemplateJson {
    templates: Vec<ExtractedTemplate>,
    #[serde(default)]
    matched: Vec<i64>,
}

pub struct TemplateAgentOutput {
    pub templates: Vec<ExtractedTemplate>,
    /// IDs of existing templates that were matched/reused.
    pub matched_template_ids: Vec<i64>,
}

impl TemplateAgent {
    pub async fn run(
        &self,
        llm: &dyn LlmProvider,
        brief: &ClassificationBrief,
        ac_codes: &[SubmissionSummary],
        existing_templates: &[TemplateBrief],
    ) -> AppResult<TemplateAgentOutput> {
        if ac_codes.is_empty() {
            return Ok(TemplateAgentOutput {
                templates: vec![],
                matched_template_ids: vec![],
            });
        }

        let prompt = build_template_prompt(brief, ac_codes, existing_templates);
        let response = llm.chat(SYSTEM_TEMPLATE, &prompt).await?;

        match parse_llm_json::<TemplateJson>(&response) {
            Ok(json) => Ok(TemplateAgentOutput {
                templates: json.templates,
                matched_template_ids: json.matched,
            }),
            Err(e) => {
                tracing::warn!("模板提取 JSON 解析失败: {e}");
                Ok(TemplateAgentOutput {
                    templates: vec![],
                    matched_template_ids: vec![],
                })
            }
        }
    }
}
