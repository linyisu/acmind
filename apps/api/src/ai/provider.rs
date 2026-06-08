use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Abstraction over LLM API calls.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a system prompt + user prompt to the LLM and return the response text.
    async fn chat(&self, system: &str, user: &str) -> AppResult<String>;
}

// ---- OpenAI-compatible provider ----

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
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user.into() },
            ],
            temperature: 0.3,
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("LLM request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "LLM API error {status}: {body}"
            )));
        }

        let chat: ChatResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("LLM response parse failed: {e}")))?;

        chat
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::Internal("LLM returned no choices".into()))
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

        Ok(format!(
            r#"{{
            "algorithm_type": "{algorithm}",
            "sub_type": "basic",
            "tags": ["{algorithm}", "template"],
            "summary": "这是一个{algorithm}类型的题目。代码使用了标准的解题思路。",
            "template_snippet": "",
            "error_analysis": null,
            "suggested_difficulty": 3
        }}"#
        ))
    }
}
