use crate::error::{AppError, AppResult};

/// Strip reasoning/thinking tags like `<think>...</think>` that some
/// reasoning models prepend to their output.
fn strip_think_tags(response: &str) -> String {
    let mut result = String::with_capacity(response.len());
    let lower = response.to_lowercase();
    let mut cursor = 0;
    while let Some(rel_start) = lower[cursor..].find("<think") {
        let abs_start = cursor + rel_start;
        result.push_str(&response[cursor..abs_start]);
        // Find the closing tag
        let close = lower[abs_start..].find("</think>").or_else(|| lower[abs_start..].find("</think"));
        if let Some(close_off) = close {
            cursor = abs_start + close_off + "</think>".len();
        } else {
            // No closing tag — assume rest is think content, skip it
            cursor = response.len();
            break;
        }
    }
    result.push_str(&response[cursor..]);
    result
}

/// Robust JSON parser with 5 fallback strategies:
/// 1. Strip <think>...</think> blocks
/// 2. Direct parse
/// 3. Extract from ```json ... ```
/// 4. Extract from ``` ... ```
/// 5. Find first { ... last }
pub fn parse_llm_json<T: serde::de::DeserializeOwned>(response: &str) -> AppResult<T> {
    let stripped = strip_think_tags(response);
    let cleaned = stripped.trim();

    // Strategy 1: Try direct parse
    if let Ok(v) = serde_json::from_str::<T>(cleaned) {
        return Ok(v);
    }

    // Strategy 2: Extract JSON from ```json ... ``` code block
    if let Some(start) = cleaned.find("```json") {
        let after_fence = &cleaned[start + 7..];
        if let Some(end) = after_fence.find("```") {
            let json_body = after_fence[..end].trim();
            if let Ok(v) = serde_json::from_str::<T>(json_body) {
                return Ok(v);
            }
        }
    }

    // Strategy 3: Extract from ``` ... ``` (no json tag)
    if let Some(start) = cleaned.find("```") {
        let after_fence = &cleaned[start + 3..];
        let body_start = after_fence.find('\n').unwrap_or(0);
        let after_tag = &after_fence[body_start..];
        if let Some(end) = after_tag.find("```") {
            let json_body = after_tag[..end].trim();
            if let Ok(v) = serde_json::from_str::<T>(json_body) {
                return Ok(v);
            }
        }
    }

    // Strategy 4: Find first { ... last } and try to parse
    if let Some(start) = cleaned.find('{') {
        if let Some(end) = cleaned.rfind('}') {
            if end > start {
                let json_body = &cleaned[start..=end];
                if let Ok(v) = serde_json::from_str::<T>(json_body) {
                    return Ok(v);
                }
            }
        }
    }

    Err(AppError::Internal(format!(
        "无法将 LLM 响应解析为 JSON\n响应: {}",
        response.chars().take(500).collect::<String>()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug)]
    struct TestObj {
        name: String,
        value: i32,
    }

    #[test]
    fn direct_parse() {
        let r: TestObj = parse_llm_json(r#"{"name":"x","value":1}"#).unwrap();
        assert_eq!(r.name, "x");
    }

    #[test]
    fn parse_from_json_codeblock() {
        let input = "```json\n{\"name\":\"y\",\"value\":2}\n```";
        let r: TestObj = parse_llm_json(input).unwrap();
        assert_eq!(r.value, 2);
    }

    #[test]
    fn parse_from_braces() {
        let input = "Here is the result:\n{\"name\":\"z\",\"value\":3}\nDone.";
        let r: TestObj = parse_llm_json(input).unwrap();
        assert_eq!(r.name, "z");
    }

    #[test]
    fn parse_invalid_returns_error() {
        let r: Result<TestObj, _> = parse_llm_json("not json");
        assert!(r.is_err());
    }

    #[test]
    fn parse_with_think_tag_prefix() {
        let input = "<think>Let me analyze this carefully.\nThe answer is x and y.</think>\n```json\n{\"name\":\"after-think\",\"value\":99}\n```";
        let r: TestObj = parse_llm_json(input).unwrap();
        assert_eq!(r.name, "after-think");
        assert_eq!(r.value, 99);
    }

    #[test]
    fn parse_with_think_tag_no_close() {
        // When <think> is not closed, strip rest of response — should fail to parse
        let input = "<think>this never ends and is all thinking\nmore thinking\n{\"name\":\"x\",\"value\":1}";
        let r: Result<TestObj, _> = parse_llm_json(input);
        // After stripping think content, nothing left to parse — should error
        assert!(r.is_err());
    }
}
