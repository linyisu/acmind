use crate::{
    ai::repo as ai_repo,
    error::AppResult,
    knowledge::repo as knowledge_repo,
    problem::repo as prob_repo,
    submission::repo as sub_repo,
};
use similar::TextDiff;

// ─── Shared types ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SubmissionSummary {
    pub id: i64,
    pub language: String,
    pub code: String,
    pub verdict: String,
    pub runtime_ms: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct AnalysisContext {
    pub problem_title: String,
    pub problem_source: String,
    pub problem_difficulty: Option<i32>,
    pub problem_statement: Option<String>,
    pub submissions: Vec<SubmissionSummary>,
    pub past_analyses: Vec<serde_json::Value>,
    pub existing_knowledge: Vec<String>,
}

/// Brief classification result, used as input to all downstream agents.
#[derive(Clone, Debug)]
pub struct ClassificationBrief {
    pub algorithm_type: String,
    pub sub_type: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub difficulty_analysis: Option<String>,
    pub progress_notes: Option<String>,
    pub suggested_difficulty: Option<i32>,
}

impl ClassificationBrief {
    /// Convert algorithm_type string to a TemplateCategory if possible.
    pub fn algorithm_type_as_category(&self) -> Option<crate::template::model::TemplateCategory> {
        use crate::template::model::TemplateCategory;
        match self.algorithm_type.as_str() {
            "dp" => Some(TemplateCategory::Dp),
            "graph" => Some(TemplateCategory::Graph),
            "string" => Some(TemplateCategory::String),
            "data_structure" => Some(TemplateCategory::DataStructure),
            "math" => Some(TemplateCategory::Math),
            "geometry" => Some(TemplateCategory::Geometry),
            "greedy" => Some(TemplateCategory::Greedy),
            "search" => Some(TemplateCategory::Search),
            "sort" => Some(TemplateCategory::Sort),
            "binary_search" => Some(TemplateCategory::BinarySearch),
            _ => None,
        }
    }
}

/// Brief info about an existing template (for matching).
#[derive(Clone, Debug)]
pub struct TemplateBrief {
    pub id: i64,
    pub title: String,
    pub summary: String,
}

// ─── Context limits ─────────────────────────────────────────────────

const MAX_CODE_LEN: usize = 4000;

// ─── Context collection ─────────────────────────────────────────────

pub async fn collect_context(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    problem_id: i64,
) -> AppResult<AnalysisContext> {
    let problem = prob_repo::find_by_id(db, user_id, problem_id)
        .await?
        .ok_or(crate::error::AppError::NotFound)?;

    let all_submissions = sub_repo::list_by_user(db, user_id, Some(problem_id)).await?;

    // Keep all submissions, sorted by time
    let mut submissions = all_submissions;
    submissions.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));

    // Gather past analyses (batch)
    let submission_ids: Vec<i64> = submissions.iter().map(|s| s.id).collect();
    let past_analyses = ai_repo::find_by_targets(db, user_id, "submission", &submission_ids).await?;
    let past_analyses: Vec<serde_json::Value> = past_analyses.into_iter().map(|r| r.result).collect();

    // Gather existing knowledge for this problem
    let existing = knowledge_repo::list_by_problem_id(db, user_id, problem_id).await?;
    let existing_knowledge: Vec<String> = existing.iter().map(|k| k.title.clone()).collect();

    Ok(AnalysisContext {
        problem_title: problem.title,
        problem_source: problem.source,
        problem_difficulty: problem.difficulty,
        problem_statement: problem.statement,
        submissions: submissions
            .into_iter()
            .map(|s| SubmissionSummary {
                id: s.id,
                language: s.language,
                code: truncate_code(&s.code, MAX_CODE_LEN),
                verdict: s.verdict,
                runtime_ms: s.runtime_ms,
            })
            .collect(),
        past_analyses,
        existing_knowledge,
    })
}

// ─── Diff generation ────────────────────────────────────────────────

/// Build diff representation of a submission chain.
/// First submission: full code. Subsequent: unified diff against previous.
pub fn build_diff_chain(submissions: &[SubmissionSummary]) -> String {
    let mut output = String::new();
    for (i, sub) in submissions.iter().enumerate() {
        let meta = format!(
            "### 提交 #{} ({}, {}, {}ms)",
            sub.id,
            sub.verdict,
            sub.language,
            sub.runtime_ms.map_or("—".to_string(), |v| v.to_string()),
        );

        if i == 0 {
            output.push_str(&format!(
                "{}\n```{}\n{}\n```\n\n",
                meta, sub.language, sub.code,
            ));
        } else {
            let prev = &submissions[i - 1];
            let diff = TextDiff::from_lines(&prev.code, &sub.code);
            let unified: String = diff
                .unified_diff()
                .context_radius(3)
                .header(&format!("提交 #{}", prev.id), &format!("提交 #{}", sub.id))
                .to_string();

            if unified.is_empty() || unified.lines().count() <= 3 {
                output.push_str(&format!("{} （与上一条相同，无代码变化）\n\n", meta));
            } else {
                output.push_str(&format!(
                    "{} (vs 提交 #{})\n```diff\n{}\n```\n\n",
                    meta, prev.id, unified,
                ));
            }
        }
    }
    output
}

/// Build non-AC submissions as full code blocks (for error analysis).
pub fn build_error_code_blocks(submissions: &[SubmissionSummary]) -> String {
    let non_ac: Vec<&SubmissionSummary> = submissions.iter().filter(|s| s.verdict != "AC").collect();
    if non_ac.is_empty() {
        return String::new();
    }
    let mut output = String::new();
    for sub in &non_ac {
        output.push_str(&format!(
            "### 提交 #{} ({}, {}, {}ms)\n```{}\n{}\n```\n\n",
            sub.id, sub.verdict, sub.language,
            sub.runtime_ms.map_or("—".to_string(), |v| v.to_string()),
            sub.language, sub.code,
        ));
    }
    output
}

// ─── Code truncation ────────────────────────────────────────────────

/// Smart code truncation: preserve function signatures.
pub fn truncate_code(code: &str, max_chars: usize) -> String {
    if code.len() <= max_chars {
        return code.to_string();
    }
    let lines: Vec<&str> = code.lines().collect();
    let head: Vec<&str> = lines.iter().take(30).copied().collect();
    let tail: Vec<&str> = lines.iter().rev().take(5).rev().copied().collect();
    let mut result = head;
    if lines.len() > 35 {
        result.push("// ... (代码已截断) ...");
    }
    result.extend(tail);
    result.join("\n")
}

// ─── Prompt helpers ─────────────────────────────────────────────────

/// Build a brief summary string from ClassificationBrief (for downstream agents).
pub fn classification_brief(b: &ClassificationBrief) -> String {
    let summary_truncated: String = b.summary.chars().take(300).collect();
    format!(
        "算法类型: {} / {}\n标签: {}\n摘要: {}",
        b.algorithm_type,
        b.sub_type,
        b.tags.join(", "),
        summary_truncated,
    )
}
