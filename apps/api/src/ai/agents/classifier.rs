use crate::{
    ai::{
        context::{AnalysisContext, ClassificationBrief},
        parse::parse_llm_json,
        prompt::{build_classifier_prompt, SYSTEM_CLASSIFIER},
    },
    error::AppResult,
};
use crate::ai::provider::LlmProvider;
use serde::Deserialize;

pub struct ClassifierAgent;

#[derive(Deserialize)]
struct ClassifierJson {
    algorithm_type: String,
    sub_type: String,
    tags: Vec<String>,
    summary: String,
    #[serde(default)]
    difficulty_analysis: Option<String>,
    #[serde(default)]
    progress_notes: Option<String>,
    #[serde(default)]
    suggested_difficulty: Option<i32>,
}

impl ClassifierAgent {
    pub async fn run(
        &self,
        llm: &dyn LlmProvider,
        ctx: &AnalysisContext,
    ) -> AppResult<(ClassificationBrief, String)> {
        let prompt = build_classifier_prompt(ctx);
        let response = llm.chat(SYSTEM_CLASSIFIER, &prompt).await?;
        let json: ClassifierJson = parse_llm_json(&response)?;

        let brief = ClassificationBrief {
            algorithm_type: json.algorithm_type,
            sub_type: json.sub_type,
            tags: json.tags,
            summary: json.summary,
            difficulty_analysis: json.difficulty_analysis,
            progress_notes: json.progress_notes,
            suggested_difficulty: json.suggested_difficulty,
        };

        Ok((brief, response))
    }
}
