use crate::db::repo;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub knowledge_points: Vec<String>,
    pub error_type: String,
    pub root_cause: String,
    pub fix_summary: String,
    pub suggestions: Vec<String>,
}

/// Analyze a problem's WA and AC codes against the statement and solutions.
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

    if api_key.is_empty() {
        return Err(AppError::InvalidInput(
            "Please configure AI API key in Settings first.".into(),
        ));
    }

    // Load problem and submissions
    let problem = repo::get_problem(pool, problem_id).await?;
    let submissions = repo::list_submissions_by_problem(pool, problem_id).await?;
    let notes = repo::list_notes_by_problem(pool, problem_id).await?;

    // Build the analysis prompt
    let prompt = build_analysis_prompt(&problem.title, &submissions, &notes);

    // Call the AI API
    let response = match provider.as_str() {
        "anthropic" => call_anthropic(&api_key, &model, &prompt).await?,
        "google" => call_gemini(&api_key, &model, &prompt).await?,
        _ => call_openai_compatible(&api_key, &model, &base_url, &prompt).await?,
    };

    // Parse JSON from response
    parse_analysis_response(&response)
}

fn build_analysis_prompt(
    title: &str,
    submissions: &[crate::db::models::Submission],
    notes: &[crate::db::models::SolutionNote],
) -> String {
    let mut prompt = format!(
        "You are an ACM/ICPC training coach. Analyze the following problem and submissions.\n\n\
        Problem: {}\n\n\
        Submissions:\n",
        title
    );

    for sub in submissions {
        prompt.push_str(&format!(
            "- Status: {}, Language: {}, Note: {}\n",
            sub.status,
            sub.language,
            sub.note.as_deref().unwrap_or("none")
        ));
    }

    if !notes.is_empty() {
        prompt.push_str("\nNotes:\n");
        for note in notes {
            prompt.push_str(&format!(
                "- [{}] {}\n",
                note.note_type, note.content
            ));
        }
    }

    prompt.push_str(
        "\nBased on the above, provide a JSON analysis with these fields:\n\
        {\n\
          \"knowledge_points\": [\"list of algorithm/data-structure tags involved\"],\n\
          \"error_type\": \"one of: logic, boundary, overflow, index, initialization, complexity, template, misread, other\",\n\
          \"root_cause\": \"detailed root cause of the failure\",\n\
          \"fix_summary\": \"concise fix explanation\",\n\
          \"suggestions\": [\"3-5 specific improvement suggestions\"]\n\
        }\n\n\
        Return ONLY valid JSON, no other text.",
    );

    prompt
}

fn parse_analysis_response(response: &str) -> Result<AnalysisResult, AppError> {
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

    let result: AnalysisResult = serde_json::from_str(json_str.trim())
        .map_err(|e| AppError::AiError(format!("Failed to parse AI response: {}", e)))?;

    Ok(result)
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
            {"role": "system", "content": "You are an expert algorithm coach. Always respond in valid JSON."},
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
        .map_err(|e| AppError::AiError(format!("API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!(
            "API error {}: {}",
            status, text
        )));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::AiError(format!("Failed to parse API response: {}", e))
    })?;

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
) -> Result<String, AppError> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "system": "You are an expert algorithm coach. Always respond in valid JSON."
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
        .map_err(|e| AppError::AiError(format!("Anthropic API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!(
            "Anthropic API error {}: {}",
            status, text
        )));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::AiError(format!("Failed to parse Anthropic response: {}", e))
    })?;

    let content = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

// -- Google Gemini API --

async fn call_gemini(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, AppError> {
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
        .map_err(|e| AppError::AiError(format!("Gemini API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::AiError(format!(
            "Gemini API error {}: {}",
            status, text
        )));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::AiError(format!("Failed to parse Gemini response: {}", e))
    })?;

    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}
