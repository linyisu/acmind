use crate::db::repo;
use crate::error::AppError;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{error, info, instrument, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub knowledge_points: Vec<String>,
    pub error_type: String,
    pub root_cause: String,
    pub fix_summary: String,
    pub suggestions: Vec<String>,
}

const AI_SYSTEM_PROMPT: &str =
    "你是一名专业的算法竞赛教练。请始终只返回有效 JSON，不要输出额外解释。";

/// Analyze a problem's WA and AC codes against the statement and solutions.
#[instrument(skip(pool), fields(problem_id = %problem_id))]
pub async fn analyze_problem(
    pool: &SqlitePool,
    problem_id: &str,
) -> Result<AnalysisResult, AppError> {
    // Get settings
    let provider = repo::get_setting(pool, "ai_provider")
        .await?
        .unwrap_or_else(|| "openai".into());
    let api_key = repo::get_setting(pool, "ai_api_key")
        .await?
        .unwrap_or_default();
    let model = repo::get_setting(pool, "ai_model")
        .await?
        .unwrap_or_else(|| "gpt-4o".into());
    let base_url = repo::get_setting(pool, "ai_base_url")
        .await?
        .unwrap_or_default();

    info!(
        provider = %provider,
        model = %model,
        has_key = !api_key.is_empty(),
        "Starting AI analysis"
    );

    if api_key.is_empty() {
        warn!("AI analysis blocked: no API key configured");
        return Err(AppError::InvalidInput(
            "请先在设置 → AI 提供商中填写 API Key。".into(),
        ));
    }

    // Load problem and submissions
    let problem = repo::get_problem(pool, problem_id).await?;
    let submissions = repo::list_submissions_by_problem(pool, problem_id).await?;
    let notes = repo::list_notes_by_problem(pool, problem_id).await?;

    info!(
        problem = %problem.title,
        submissions = submissions.len(),
        notes = notes.len(),
        "Loaded problem data"
    );

    // Build the analysis prompt
    let prompt = build_analysis_prompt(&problem.title, &submissions, &notes);

    // Call the AI API
    info!("Calling {} API with model {}", provider, model);
    let response = match provider.as_str() {
        "anthropic" => call_anthropic(&api_key, &model, &prompt).await?,
        "google" => call_gemini(&api_key, &model, &prompt).await?,
        _ => call_openai_compatible(&api_key, &model, &base_url, &prompt).await?,
    };

    info!("AI response received ({} bytes)", response.len());

    // Parse JSON from response
    parse_analysis_response(&response)
}

fn build_analysis_prompt(
    title: &str,
    submissions: &[crate::db::models::Submission],
    notes: &[crate::db::models::SolutionNote],
) -> String {
    let mut prompt = format!(
        "你是一名 ACM/ICPC 训练教练。请分析下面的题目、提交记录和笔记。\n\n\
        题目：{}\n\n\
        提交记录：\n",
        title
    );

    for sub in submissions {
        prompt.push_str(&format!(
            "- 状态：{}，语言：{}，备注：{}\n",
            sub.status,
            sub.language,
            sub.note.as_deref().unwrap_or("无")
        ));
    }

    if !notes.is_empty() {
        prompt.push_str("\n笔记：\n");
        for note in notes {
            prompt.push_str(&format!("- [{}] {}\n", note.note_type, note.content));
        }
    }

    prompt.push_str(
        "\n请基于以上信息，返回一个 JSON 分析对象，字段必须严格如下（字段名保持英文，不要翻译）：\n\
        {\n\
          \"knowledge_points\": [\"涉及的算法或数据结构标签，例如 dp、graph、greedy\"],\n\
          \"error_type\": \"只能是以下之一：logic、boundary、overflow、index、initialization、complexity、template、misread、other\",\n\
          \"root_cause\": \"用中文详细说明错误根因\",\n\
          \"fix_summary\": \"用中文简明说明修复方式\",\n\
          \"suggestions\": [\"3-5 条中文、具体、可执行的改进建议\"]\n\
        }\n\n\
        只返回有效 JSON，不要返回 Markdown、代码块或任何额外文本。",
    );

    prompt
}

pub fn parse_analysis_response(response: &str) -> Result<AnalysisResult, AppError> {
    // Extract JSON from the response (handle markdown code blocks)
    let json_str = if let Some(start) = response.find("```json") {
        let inner = &response[start + 7..];
        if let Some(end) = inner.find("```") {
            &inner[..end]
        } else {
            inner
        }
    } else if let Some(start) = response.find('{') {
        let end = response.rfind('}').unwrap_or(response.len() - 1);
        &response[start..=end]
    } else {
        response
    };

    if let Ok(result) = serde_json::from_str(json_str.trim()) {
        info!("AI analysis completed successfully");
        return Ok(result);
    }

    let Some(repaired) = repair_truncated_json(json_str.trim()) else {
        return serde_json::from_str(json_str.trim()).map_err(|e| {
            error!("Failed to parse AI JSON response: {}", e);
            error!(
                "Raw response (first 500 chars): {}",
                &response[..response.len().min(500)]
            );
            AppError::AiError(format!("AI 返回内容不是有效 JSON：{}", e))
        });
    };

    let result: AnalysisResult = serde_json::from_str(&repaired).map_err(|e| {
        error!("Failed to parse repaired AI JSON response: {}", e);
        error!("Repaired response: {}", repaired);
        AppError::AiError(format!("AI 返回内容不完整，且自动修复失败：{}", e))
    })?;

    warn!("AI JSON response was truncated and repaired");
    Ok(result)
}

fn repair_truncated_json(input: &str) -> Option<String> {
    if !input.starts_with('{') {
        return None;
    }

    let mut output = input.trim().to_string();
    let quote_count = output.chars().filter(|c| *c == '"').count();
    if quote_count % 2 == 1 {
        output.push('"');
    }

    let open_arrays = output.chars().filter(|c| *c == '[').count();
    let close_arrays = output.chars().filter(|c| *c == ']').count();
    for _ in close_arrays..open_arrays {
        output.push(']');
    }

    let open_objects = output.chars().filter(|c| *c == '{').count();
    let close_objects = output.chars().filter(|c| *c == '}').count();
    for _ in close_objects..open_objects {
        output.push('}');
    }

    Some(output)
}

// -- OpenAI-compatible API (OpenAI, DeepSeek, custom endpoints) --

async fn call_openai_compatible(
    api_key: &str,
    model: &str,
    base_url: &str,
    prompt: &str,
) -> Result<String, AppError> {
    let url = if base_url.is_empty() {
        "https://api.openai.com/v1/chat/completions".to_string()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": AI_SYSTEM_PROMPT},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 1024
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::AiError(format!("API 请求失败：{}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!("API 错误 {}：{}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::AiError(format!("解析 API 响应失败：{}", e)))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

// -- Anthropic (Claude) API --

async fn call_anthropic(api_key: &str, model: &str, prompt: &str) -> Result<String, AppError> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "system": AI_SYSTEM_PROMPT
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::AiError(format!("Anthropic API 请求失败：{}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!(
            "Anthropic API 错误 {}：{}",
            status, text
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::AiError(format!("解析 Anthropic 响应失败：{}", e)))?;

    let content = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

// -- Google Gemini API --

async fn call_gemini(api_key: &str, model: &str, prompt: &str) -> Result<String, AppError> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }],
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 1024
        }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::AiError(format!("Gemini API 请求失败：{}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!(
            "Gemini API 错误 {}：{}",
            status, text
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::AiError(format!("解析 Gemini 响应失败：{}", e)))?;

    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

// -- Streaming API calls (OpenAI-compatible only) --

/// Stream AI response chunks via callback.
pub async fn analyze_problem_streaming(
    pool: &SqlitePool,
    problem_id: &str,
    on_chunk: impl Fn(&str),
) -> Result<String, AppError> {
    let provider = repo::get_setting(pool, "ai_provider")
        .await?
        .unwrap_or_else(|| "openai".into());
    let api_key = repo::get_setting(pool, "ai_api_key")
        .await?
        .unwrap_or_default();
    let model = repo::get_setting(pool, "ai_model")
        .await?
        .unwrap_or_else(|| "gpt-4o".into());
    let base_url = repo::get_setting(pool, "ai_base_url")
        .await?
        .unwrap_or_default();

    info!("Starting streaming AI analysis");

    if api_key.is_empty() {
        return Err(AppError::InvalidInput(
            "请先在设置中填写 AI API Key。".into(),
        ));
    }

    let problem = repo::get_problem(pool, problem_id).await?;
    let submissions = repo::list_submissions_by_problem(pool, problem_id).await?;
    let notes = repo::list_notes_by_problem(pool, problem_id).await?;

    let prompt = build_analysis_prompt(&problem.title, &submissions, &notes);

    let response = match provider.as_str() {
        "anthropic" => {
            on_chunk("（Anthropic 暂不支持流式输出，请稍等...）\n");
            call_anthropic(&api_key, &model, &prompt).await?
        }
        "google" => {
            on_chunk("（Gemini 暂不支持流式输出，请稍等...）\n");
            call_gemini(&api_key, &model, &prompt).await?
        }
        _ => {
            call_openai_compatible_streaming(&api_key, &model, &base_url, &prompt, &on_chunk)
                .await?
        }
    };

    info!("Streaming AI analysis complete ({} bytes)", response.len());
    Ok(response)
}

/// Call OpenAI-compatible API with SSE streaming.
async fn call_openai_compatible_streaming(
    api_key: &str,
    model: &str,
    base_url: &str,
    prompt: &str,
    on_chunk: &impl Fn(&str),
) -> Result<String, AppError> {
    let url = if base_url.is_empty() {
        "https://api.openai.com/v1/chat/completions".to_string()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": AI_SYSTEM_PROMPT},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 1024,
        "stream": true
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::AiError(format!("流式 API 请求失败：{}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!("API 错误 {}：{}", status, text)));
    }

    let mut full_text = String::new();
    let mut pending = String::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk =
            chunk_result.map_err(|e| AppError::AiError(format!("读取流式响应失败：{}", e)))?;
        pending.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_index) = pending.find('\n') {
            let line = pending[..newline_index].trim().to_string();
            pending.drain(..=newline_index);
            append_openai_stream_line(&line, &mut full_text, on_chunk);
        }
    }

    if !pending.trim().is_empty() {
        append_openai_stream_line(pending.trim(), &mut full_text, on_chunk);
    }

    if full_text.is_empty() {
        return Err(AppError::AiError("AI 返回了空响应".into()));
    }

    Ok(full_text)
}

fn append_openai_stream_line(line: &str, full_text: &mut String, on_chunk: &impl Fn(&str)) {
    if line.is_empty() || line == "data: [DONE]" {
        return;
    }

    let Some(data) = line.strip_prefix("data: ") else {
        return;
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
        return;
    };

    if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
        full_text.push_str(delta);
        on_chunk(delta);
    }
}

#[cfg(test)]
mod tests {
    use super::{append_openai_stream_line, parse_analysis_response};

    #[test]
    fn appends_openai_stream_content_delta() {
        let mut full_text = String::new();
        let chunks = std::cell::RefCell::new(Vec::new());

        append_openai_stream_line(
            r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#,
            &mut full_text,
            &|chunk| chunks.borrow_mut().push(chunk.to_string()),
        );

        assert_eq!(full_text, "hello");
        assert_eq!(chunks.into_inner(), vec!["hello"]);
    }

    #[test]
    fn ignores_done_and_non_content_lines() {
        let mut full_text = String::new();
        let chunks: Vec<String> = Vec::new();

        append_openai_stream_line("data: [DONE]", &mut full_text, &|_| {});
        append_openai_stream_line("event: ping", &mut full_text, &|_| {});

        assert!(full_text.is_empty());
        assert!(chunks.is_empty());
    }

    #[test]
    fn repairs_truncated_analysis_json() {
        let result = parse_analysis_response(
            r#"{"knowledge_points":["dp"],"error_type":"logic","root_cause":"状态转移漏了边界","fix_summary":"补上边界","suggestions":["补充边界用例""#,
        )
        .unwrap();

        assert_eq!(result.error_type, "logic");
        assert_eq!(result.suggestions, vec!["补充边界用例"]);
    }
}
