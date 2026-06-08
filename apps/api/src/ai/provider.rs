use crate::error::AppResult;
use async_trait::async_trait;

/// Abstraction over LLM API calls. Implement this trait to connect
/// to OpenAI, Claude, Ollama, or any other LLM provider.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a system prompt + user prompt to the LLM and return the response text.
    async fn chat(&self, system: &str, user: &str) -> AppResult<String>;
}

/// Mock LLM provider for development and testing.
/// Returns a hardcoded analysis result without calling any external API.
pub struct NoopLlmProvider;

#[async_trait]
impl LlmProvider for NoopLlmProvider {
    async fn chat(&self, _system: &str, user: &str) -> AppResult<String> {
        // Extract some info from the prompt to make the mock response more realistic
        let verdict = if user.contains("AC") { "AC" } else { "WA" };
        let algorithm = if user.contains("dp") || user.contains("DP") || user.contains("背包") {
            "dp"
        } else if user.contains("graph") || user.contains("图") || user.contains("Dijkstra") {
            "graph"
        } else if user.contains("sort") || user.contains("排序") {
            "sorting"
        } else {
            "general"
        };

        Ok(format!(
            r#"{{
            "algorithm_type": "{algorithm}",
            "sub_type": "basic",
            "tags": ["{algorithm}", "template"],
            "summary": "这是一个{algorithm}类型的题目。代码使用了标准的解题思路。",
            "template_snippet": "",
            "error_analysis": {},
            "suggested_difficulty": 3
        }}"#,
            if verdict != "AC" {
                r#""可能存在逻辑错误或边界条件处理不当""#
            } else {
                "null"
            }
        ))
    }
}
