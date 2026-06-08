use crate::{
    ai::{
        model::{AnalysisResult, AnalysisResp},
        repo,
    },
    error::{AppError, AppResult},
    state::AppState,
    submission::repo as sub_repo,
    problem::repo as prob_repo,
};
use serde_json::Value;

const SYSTEM_PROMPT: &str = "你是一个竞品编程分析助手。分析提交代码，返回 JSON 格式结果。只返回 JSON，不要其他内容。";

pub struct AiService<'a> {
    pub state: &'a AppState,
}

impl<'a> AiService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Analyze a submission using the LLM provider.
    pub async fn analyze_submission(
        &self,
        user_id: i64,
        submission_id: i64,
    ) -> AppResult<AnalysisResp> {
        // Check if already analyzed
        if let Some(existing) = repo::find_by_target(&self.state.db, user_id, "submission", submission_id).await? {
            return to_resp(existing);
        }

        // Fetch submission
        let submission = sub_repo::find_by_id(&self.state.db, user_id, submission_id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Fetch problem
        let problem = prob_repo::find_by_id(&self.state.db, user_id, submission.problem_id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Build user prompt
        let user_prompt = format!(
            "题目：{}（来源：{}，难度：{}）\n语言：{}\n判题结果：{}\n代码：\n{}\n\n请返回 JSON：\n{{\n  \"algorithm_type\": \"算法类型\",\n  \"sub_type\": \"子类型\",\n  \"tags\": [\"标签1\", \"标签2\"],\n  \"summary\": \"解题思路摘要\",\n  \"template_snippet\": \"核心代码片段\",\n  \"error_analysis\": \"非AC时的错误分析\",\n  \"suggested_difficulty\": 3\n}}",
            problem.title,
            problem.source,
            problem.difficulty.map_or("未知".to_string(), |d| d.to_string()),
            submission.language,
            submission.verdict,
            submission.code,
        );

        // Call LLM
        let response = self.state.llm.chat(SYSTEM_PROMPT, &user_prompt).await?;

        // Parse JSON response
        let result: AnalysisResult = serde_json::from_str(response.trim())
            .map_err(|e| AppError::Internal(format!("Failed to parse LLM response as JSON: {e}\nResponse: {response}")))?;

        // Save to DB
        let result_json: Value = serde_json::to_value(&result)
            .map_err(|e| AppError::Internal(format!("serialize analysis result: {e}")))?;
        let id = repo::insert(&self.state.db, user_id, "submission", submission_id, &result_json).await?;

        Ok(AnalysisResp {
            id,
            target_type: "submission".to_string(),
            target_id: submission_id,
            result,
            created_at: chrono::Utc::now(),
        })
    }

    /// List all analysis results for a user.
    pub async fn list(&self, user_id: i64) -> AppResult<Vec<AnalysisResp>> {
        let rows = repo::list_by_user(&self.state.db, user_id).await?;
        rows.into_iter().map(to_resp).collect()
    }
}

fn to_resp(row: repo::AiAnalysisRow) -> AppResult<AnalysisResp> {
    let result: AnalysisResult = serde_json::from_value(row.result)
        .map_err(|e| AppError::Internal(format!("parse analysis result: {e}")))?;
    Ok(AnalysisResp {
        id: row.id,
        target_type: row.target_type,
        target_id: row.target_id,
        result,
        created_at: row.created_at,
    })
}
