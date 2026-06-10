use crate::{
    ai::{
        context::{AnalysisContext, ClassificationBrief},
        model::ExtractedKnowledge,
        parse::parse_llm_json,
        prompt::{build_knowledge_prompt, SYSTEM_KNOWLEDGE},
    },
    error::AppResult,
};
use crate::ai::provider::LlmProvider;
use serde::Deserialize;

pub struct KnowledgeAgent;

#[derive(Deserialize)]
struct KnowledgeJson {
    knowledge_points: Vec<ExtractedKnowledge>,
}

impl KnowledgeAgent {
    pub async fn run(
        &self,
        llm: &dyn LlmProvider,
        ctx: &AnalysisContext,
        brief: &ClassificationBrief,
    ) -> AppResult<Vec<ExtractedKnowledge>> {
        let prompt = build_knowledge_prompt(brief, ctx);
        let response = llm.chat(SYSTEM_KNOWLEDGE, &prompt).await?;

        match parse_llm_json::<KnowledgeJson>(&response) {
            Ok(json) => Ok(json.knowledge_points),
            Err(e) => {
                tracing::warn!("知识点提取 JSON 解析失败: {e}");
                Ok(vec![])
            }
        }
    }
}
