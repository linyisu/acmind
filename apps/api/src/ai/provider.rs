use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Abstraction over LLM API calls.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a system prompt + user prompt to the LLM and return the response text.
    async fn chat(&self, system: &str, user: &str) -> AppResult<String>;
}

// ---- OpenAI-compatible provider ----

const MAX_RETRIES: u32 = 2;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResp,
}

#[derive(Deserialize)]
struct ChatMessageResp {
    content: String,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, system: &str, user: &str) -> AppResult<String> {
        let req = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: system.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user.into(),
                },
            ],
            temperature: 0.3,
        };

        for attempt in 0..=MAX_RETRIES {
            let resp = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .timeout(REQUEST_TIMEOUT)
                .json(&req)
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let chat: ChatResponse = r.json().await.map_err(|e| {
                        AppError::Internal(format!("LLM response parse failed: {e}"))
                    })?;
                    return chat
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .ok_or_else(|| AppError::Internal("LLM returned no choices".into()));
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    // Retry on rate limit (429) or server errors (5xx)
                    if (status == 429 || status >= 500) && attempt < MAX_RETRIES {
                        let delay = Duration::from_millis(1000 * 2u64.pow(attempt));
                        tracing::warn!(
                            "LLM API error {} (attempt {}/{}), retrying in {:?}",
                            status,
                            attempt + 1,
                            MAX_RETRIES + 1,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(AppError::Internal(format!(
                        "LLM API error {status}: {body}"
                    )));
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        let delay = Duration::from_millis(1000 * 2u64.pow(attempt));
                        tracing::warn!(
                            "LLM request failed (attempt {}/{}): {}, retrying in {:?}",
                            attempt + 1,
                            MAX_RETRIES + 1,
                            e,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(AppError::Internal(format!("LLM request failed: {e}")));
                }
            }
        }

        Err(AppError::Internal(
            "LLM request failed after all retries".into(),
        ))
    }
}

// ---- Noop provider (mock for dev/testing) ----

pub struct NoopLlmProvider;

#[async_trait]
impl LlmProvider for NoopLlmProvider {
    async fn chat(&self, _system: &str, user: &str) -> AppResult<String> {
        let algorithm = if user.contains("dp") || user.contains("DP") || user.contains("背包") {
            "dp"
        } else if user.contains("graph") || user.contains("图") || user.contains("Dijkstra") {
            "graph"
        } else if user.contains("sort") || user.contains("排序") {
            "sorting"
        } else {
            "general"
        };

        // Detect prompt type from system prompt and return matching JSON format
        if _system.contains("模板提取") {
            Ok(format!(
                "{{\"templates\":[{{\"title\":\"{algorithm} 基础模板\",\"code\":\"void solve() {{\\n    // {algorithm} template\\n}}\",\"description\":\"适用于{algorithm}类型题目的基础代码模板\",\"category\":\"{cat}\",\"time_complexity\":\"$O(n)$\"}},{{\"title\":\"{algorithm} 优化模板\",\"code\":\"void solve_opt() {{\\n    // optimized {algorithm}\\n}}\",\"description\":\"{algorithm}的优化版本模板\",\"category\":\"{cat}\",\"time_complexity\":\"$O(n \\\\log n)$\"}}]}}",
                cat = match algorithm {
                    "dp" => "dp",
                    "graph" => "graph",
                    "sorting" => "sort",
                    _ => "other",
                }
            ))
        } else if _system.contains("错误分析") {
            Ok(format!(
                "{{\"errors\":[{{\"title\":\"边界条件错误\",\"description\":\"在{algorithm}题目中常见的边界处理错误\",\"fix_suggestion\":\"仔细检查循环边界和数组索引\"}},{{\"title\":\"初始化错误\",\"description\":\"变量或数组未正确初始化\",\"fix_suggestion\":\"确保所有变量在使用前已正确初始化\"}}]}}"
            ))
        } else if _system.contains("知识点") {
            Ok(format!(
                "{{\"knowledge_points\":[{{\"title\":\"{algorithm}算法基础\",\"content\":\"## {algorithm}算法\\n\\n{algorithm}是一种常用的算法思想。\\n\\n### 核心思路\\n- 分析问题结构\\n- 选择合适的策略\\n\\n### 时间复杂度\\n通常为 $O(n)$ 或 $O(n \\\\log n)$\"}},{{\"title\":\"{algorithm}常见优化技巧\",\"content\":\"## 优化技巧\\n\\n1. **预处理**：提前计算常用值\\n2. **剪枝**：减少不必要的计算\\n3. **空间换时间**：使用额外空间加速\"}}]}}"
            ))
        } else {
            // Default: main analysis format
            Ok(format!(
                "{{\"algorithm_type\":\"{algorithm}\",\"sub_type\":\"basic\",\"tags\":[\"{algorithm}\",\"template\"],\"summary\":\"## 综合分析\\n\\n这是一个 **{algorithm}** 类型的题目。\\n\\n### 解题思路\\n代码使用了标准的{algorithm}解题思路，整体结构清晰。\\n\\n### 时间复杂度\\n主要操作的时间复杂度为 $O(n \\\\log n)$。\\n\\n### 提交趋势\\n从提交记录来看，逐步改进了算法实现。\",\"difficulty_analysis\":\"该题难度适中，主要考察{algorithm}的基本应用。\",\"progress_notes\":\"学习进步明显，从最初的错误提交到最后的 AC，体现了对{algorithm}算法的逐步理解。\",\"suggested_difficulty\":3}}"
            ))
        }
    }
}
