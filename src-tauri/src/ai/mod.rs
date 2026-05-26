use crate::db::models::Problem;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiPromptLocale {
    Zh,
    En,
}

impl AiPromptLocale {
    fn from_setting(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("en") | Some("en-US") | Some("en_US") => Self::En,
            _ => Self::Zh,
        }
    }

    fn system_prompt(self) -> &'static str {
        match self {
            Self::Zh => "你是一名专业的算法竞赛教练。请始终只返回有效 JSON，不要输出额外解释。",
            Self::En => "You are an expert algorithm coach. Always respond with valid JSON only, with no extra explanation.",
        }
    }

    fn no_note(self) -> &'static str {
        match self {
            Self::Zh => "无",
            Self::En => "none",
        }
    }
}

async fn read_problem_statement(problem: &Problem) -> Option<String> {
    let path = problem.statement_path.as_ref()?;
    match tokio::fs::read_to_string(path).await {
        Ok(content) => Some(content),
        Err(err) => {
            warn!(path = %path, error = %err, "Failed to read problem statement");
            None
        }
    }
}

pub async fn format_problem_statement(
    pool: &SqlitePool,
    raw_text: &str,
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
    let prompt_locale = AiPromptLocale::from_setting(repo::get_setting(pool, "app_locale").await?);

    if api_key.is_empty() {
        return Err(AppError::InvalidInput(
            "请先在设置 → AI 提供商中填写 API Key。".into(),
        ));
    }

    let prompt = build_statement_format_prompt(raw_text, prompt_locale);
    let response = match provider.as_str() {
        "anthropic" => call_anthropic(&api_key, &model, &prompt, prompt_locale).await?,
        "google" => call_gemini(&api_key, &model, &prompt).await?,
        _ => call_openai_compatible(&api_key, &model, &base_url, &prompt, prompt_locale).await?,
    };

    let formatted = normalize_markdown_spacing(&extract_formatted_markdown(&response));
    if formatted.is_empty() {
        return Err(AppError::AiError("AI 整理后题面为空，请重试。".into()));
    }

    Ok(formatted)
}

fn build_statement_format_prompt(raw_text: &str, locale: AiPromptLocale) -> String {
    match locale {
        AiPromptLocale::Zh => format!(
            "请把下面从 OJ 页面复制出来的混乱题面整理为可渲染的 Markdown。\n\n\
            要求：\n\
            1. 不要解题，不要补充原文没有的信息。\n\
            2. 保留题意、输入、输出、样例、约束和备注。\n\
            3. 修复粘连的数学公式，例如 n(2≤n≤12)n(2≤n≤12) 应整理为 $n$ ($2 \\le n \\le 12$)。\n\
            4. 使用 Markdown 标题：## 题目描述、## 输入格式、## 输出格式、## 样例。\n\
            5. 样例输入输出必须放入 ```text 代码块。\n\
            6. 数学表达式使用 $...$，不要使用 HTML。\n\
            7. 段落之间最多保留一个空行；不要在每行之间插入空行。\n\
            8. 只返回整理后的 Markdown 纯文本，不要返回 JSON，不要返回解释。\n\n\
            原始题面：\n{}",
            raw_text.trim()
        ),
        AiPromptLocale::En => format!(
            "Clean up the following messy competitive-programming statement into renderable Markdown.\n\n\
            Requirements:\n\
            1. Do not solve the problem or add information not present in the text.\n\
            2. Preserve statement, input, output, samples, constraints, and notes.\n\
            3. Repair duplicated/stuck formulas, e.g. n(2≤n≤12)n(2≤n≤12) should become $n$ ($2 \\le n \\le 12$).\n\
            4. Use Markdown headings: ## Statement, ## Input, ## Output, ## Samples.\n\
            5. Put sample input/output in ```text code fences.\n\
            6. Use $...$ for math expressions, not HTML.\n\
            7. Keep at most one blank line between paragraphs; do not insert blank lines between every line.\n\
            8. Return only the cleaned Markdown plain text. Do not return JSON or explanation.\n\n\
            Raw statement:\n{}",
            raw_text.trim()
        ),
    }
}

fn strip_markdown_fence(response: &str) -> &str {
    let trimmed = response.trim();
    if let Some(rest) = trimmed.strip_prefix("```markdown") {
        return rest.trim_end_matches("```");
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim_end_matches("```");
    }
    trimmed
}

fn extract_formatted_markdown(response: &str) -> String {
    let content = strip_markdown_fence(response).trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(markdown) = value.get("markdown").and_then(|item| item.as_str()) {
            return markdown.to_string();
        }
    }

    content.to_string()
}

fn normalize_markdown_spacing(markdown: &str) -> String {
    let mut result = String::new();
    let mut blank_lines = 0usize;
    let mut in_fence = false;

    for raw_line in markdown.trim().lines() {
        let line = raw_line.trim_end();
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }

        if !in_fence && line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines > 1 {
                continue;
            }
        } else {
            blank_lines = 0;
        }

        result.push_str(line);
        result.push('\n');
    }

    result.trim().to_string()
}

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

    let prompt_locale = AiPromptLocale::from_setting(repo::get_setting(pool, "app_locale").await?);

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

    let statement = read_problem_statement(&problem).await;

    info!(
        problem = %problem.title,
        submissions = submissions.len(),
        notes = notes.len(),
        has_statement = statement.is_some(),
        "Loaded problem data"
    );

    // Build the analysis prompt
    let prompt = build_analysis_prompt(
        &problem.title,
        statement.as_deref(),
        &submissions,
        &notes,
        prompt_locale,
    );

    // Call the AI API
    info!("Calling {} API with model {}", provider, model);
    let response = match provider.as_str() {
        "anthropic" => call_anthropic(&api_key, &model, &prompt, prompt_locale).await?,
        "google" => call_gemini(&api_key, &model, &prompt).await?,
        _ => call_openai_compatible(&api_key, &model, &base_url, &prompt, prompt_locale).await?,
    };

    info!("AI response received ({} bytes)", response.len());

    // Parse JSON from response
    parse_analysis_response(&response)
}

fn build_analysis_prompt(
    title: &str,
    statement: Option<&str>,
    submissions: &[crate::db::models::Submission],
    notes: &[crate::db::models::SolutionNote],
    locale: AiPromptLocale,
) -> String {
    let mut prompt = match locale {
        AiPromptLocale::Zh => format!(
            "你是一名 ACM/ICPC 训练教练。请分析下面的题目、提交记录和笔记。\n\n\
            题目：{}\n\n\
            提交记录：\n",
            title
        ),
        AiPromptLocale::En => format!(
            "You are an ACM/ICPC training coach. Analyze the following problem, submissions, and notes.\n\n\
            Problem: {}\n\n\
            Submissions:\n",
            title
        ),
    };

    if let Some(statement) = statement.filter(|content| !content.trim().is_empty()) {
        match locale {
            AiPromptLocale::Zh => prompt.push_str(&format!("题面：\n{}\n\n", statement.trim())),
            AiPromptLocale::En => prompt.push_str(&format!("Statement:\n{}\n\n", statement.trim())),
        }
    }

    for sub in submissions {
        match locale {
            AiPromptLocale::Zh => prompt.push_str(&format!(
                "- 状态：{}，语言：{}，备注：{}\n",
                sub.status,
                sub.language,
                sub.note.as_deref().unwrap_or(locale.no_note())
            )),
            AiPromptLocale::En => prompt.push_str(&format!(
                "- Status: {}, Language: {}, Note: {}\n",
                sub.status,
                sub.language,
                sub.note.as_deref().unwrap_or(locale.no_note())
            )),
        }
    }

    if !notes.is_empty() {
        match locale {
            AiPromptLocale::Zh => prompt.push_str("\n笔记：\n"),
            AiPromptLocale::En => prompt.push_str("\nNotes:\n"),
        }
        for note in notes {
            prompt.push_str(&format!("- [{}] {}\n", note.note_type, note.content));
        }
    }

    match locale {
        AiPromptLocale::Zh => prompt.push_str(
            "\n请基于以上信息，返回一个 JSON 分析对象，字段必须严格如下（字段名保持英文，不要翻译）：\n\
            {\n\
              \"knowledge_points\": [\"涉及的算法或数据结构标签，例如 dp、graph、greedy\"],\n\
              \"error_type\": \"只能是以下之一：logic、boundary、overflow、index、initialization、complexity、template、misread、other\",\n\
              \"root_cause\": \"用中文详细说明错误根因\",\n\
              \"fix_summary\": \"用中文简明说明修复方式\",\n\
              \"suggestions\": [\"3-5 条中文、具体、可执行的改进建议\"]\n\
            }\n\n\
            只返回有效 JSON，不要返回 Markdown、代码块或任何额外文本。",
        ),
        AiPromptLocale::En => prompt.push_str(
            "\nBased on the above, return a JSON analysis object with exactly these fields (keep field names in English):\n\
            {\n\
              \"knowledge_points\": [\"algorithm/data-structure tags involved, such as dp, graph, greedy\"],\n\
              \"error_type\": \"one of: logic, boundary, overflow, index, initialization, complexity, template, misread, other\",\n\
              \"root_cause\": \"detailed root cause of the failure in English\",\n\
              \"fix_summary\": \"concise fix explanation in English\",\n\
              \"suggestions\": [\"3-5 specific, actionable improvement suggestions in English\"]\n\
            }\n\n\
            Return valid JSON only. Do not return Markdown, code fences, or any extra text.",
        ),
    }

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
    prompt_locale: AiPromptLocale,
) -> Result<String, AppError> {
    let url = if base_url.is_empty() {
        "https://api.openai.com/v1/chat/completions".to_string()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": prompt_locale.system_prompt()},
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

async fn call_anthropic(
    api_key: &str,
    model: &str,
    prompt: &str,
    prompt_locale: AiPromptLocale,
) -> Result<String, AppError> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "system": prompt_locale.system_prompt()
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
    let prompt_locale = AiPromptLocale::from_setting(repo::get_setting(pool, "app_locale").await?);

    info!("Starting streaming AI analysis");

    if api_key.is_empty() {
        return Err(AppError::InvalidInput(
            "请先在设置中填写 AI API Key。".into(),
        ));
    }

    let problem = repo::get_problem(pool, problem_id).await?;
    let submissions = repo::list_submissions_by_problem(pool, problem_id).await?;
    let notes = repo::list_notes_by_problem(pool, problem_id).await?;

    let statement = read_problem_statement(&problem).await;
    let prompt = build_analysis_prompt(
        &problem.title,
        statement.as_deref(),
        &submissions,
        &notes,
        prompt_locale,
    );

    let response = match provider.as_str() {
        "anthropic" => {
            on_chunk("（Anthropic 暂不支持流式输出，请稍等...）\n");
            call_anthropic(&api_key, &model, &prompt, prompt_locale).await?
        }
        "google" => {
            on_chunk("（Gemini 暂不支持流式输出，请稍等...）\n");
            call_gemini(&api_key, &model, &prompt).await?
        }
        _ => {
            call_openai_compatible_streaming(
                &api_key,
                &model,
                &base_url,
                &prompt,
                prompt_locale,
                &on_chunk,
            )
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
    prompt_locale: AiPromptLocale,
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
            {"role": "system", "content": prompt_locale.system_prompt()},
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
    use super::{
        append_openai_stream_line, build_analysis_prompt, extract_formatted_markdown,
        normalize_markdown_spacing, parse_analysis_response, AiPromptLocale,
    };

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
    fn builds_chinese_analysis_prompt_by_default_locale() {
        let prompt = build_analysis_prompt("两数之和", None, &[], &[], AiPromptLocale::Zh);

        assert!(prompt.contains("题目：两数之和"));
        assert!(prompt.contains("用中文详细说明错误根因"));
        assert!(!prompt.contains("Problem:"));
    }

    #[test]
    fn builds_english_analysis_prompt_when_locale_is_en() {
        let prompt = build_analysis_prompt("Two Sum", None, &[], &[], AiPromptLocale::En);

        assert!(prompt.contains("Problem: Two Sum"));
        assert!(prompt.contains("root cause of the failure in English"));
        assert!(!prompt.contains("题目："));
    }

    #[test]
    fn extracts_markdown_from_json_wrapper() {
        let response = r###"{"markdown":"## 题目描述\n\n内容"}"###;

        assert_eq!(extract_formatted_markdown(response), "## 题目描述\n\n内容");
    }

    #[test]
    fn normalizes_excessive_markdown_blank_lines() {
        let markdown = "## 题目描述\n\n\n内容\n\n\n\n```text\n1\n\n2\n```\n\n\n## 输出";
        let normalized = normalize_markdown_spacing(markdown);

        assert_eq!(
            normalized,
            "## 题目描述\n\n内容\n\n```text\n1\n\n2\n```\n\n## 输出"
        );
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
