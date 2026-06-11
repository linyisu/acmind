use crate::ai::context::{
    build_diff_chain, build_error_code_blocks, classification_brief, AnalysisContext,
    ClassificationBrief, SubmissionSummary, TemplateBrief,
};

// ─── System prompts ─────────────────────────────────────────────────

pub const SYSTEM_CLASSIFIER: &str = "\
你是一个算法训练分析专家。分析一个题目的提交历史（以 diff 形式呈现），判断算法类型。

要求：
1. summary 使用 Markdown，数学公式用 LaTeX（行内 $O(n)$，独立公式用 $$...$$）
2. algorithm_type 从以下选择：dp, graph, string, data_structure, math, geometry, greedy, search, sort, binary_search, other
3. 重点分析提交的变化趋势和解题思路
4. suggested_difficulty 1-5 整数
5. 只返回 JSON，不要任何其他文字

输出格式：
{
  \"algorithm_type\": \"算法类型\",
  \"sub_type\": \"子类型\",
  \"tags\": [\"标签1\", \"标签2\"],
  \"summary\": \"Markdown 综合分析，含 LaTeX 公式\",
  \"difficulty_analysis\": \"难度评估（Markdown）\",
  \"progress_notes\": \"学习进步分析（Markdown）\",
  \"suggested_difficulty\": 3
}";

pub const SYSTEM_TEMPLATE: &str = "\
你是一个算法模板提取专家。从 AC 代码中提取可复用的代码模板。

## 现有模板库
以下是用户已有的相关模板。如果代码可以直接复用某个现有模板，请在 matched 字段中返回其 ID。
如果现有模板不够用，需要提取新模板。

要求：
1. 去除题目特有的输入输出部分（读入/输出），只保留核心算法逻辑
2. category 必须是以下之一：data_structure, graph, dp, string, math, geometry, greedy, search, sort, binary_search, other
3. time_complexity 使用 LaTeX 格式，如 \"$O(n \\\\log n)$\"
4. description 使用 Markdown，包含适用场景和注意事项
5. 如果代码中没有值得提取的通用模板，返回空数组
6. 只返回 JSON，不要任何其他文字

输出格式：
{
  \"templates\": [
    {\"title\": \"模板名称\", \"code\": \"核心代码\", \"description\": \"说明（Markdown + LaTeX）\", \"category\": \"算法分类\", \"time_complexity\": \"$O(n)$\"}
  ],
  \"matched\": [1, 3]
}";

pub const SYSTEM_ERROR: &str = "\
你是一个编程错误分析专家。分析提交历史中的代码，找出反复出现的错误模式。

要求：
1. 重点关注反复修改的部分——那是典型的 bug 热点
2. description 说明错误原因和触发条件
3. fix_suggestion 给出具体修复方法
4. 如果没有明显的错误模式，返回空数组
5. 只返回 JSON，不要任何其他文字

输出格式：
{
  \"errors\": [
    {\"title\": \"错误模式名称\", \"description\": \"错误描述和常见原因\", \"fix_suggestion\": \"修复建议\"}
  ]
}";

pub const SYSTEM_KNOWLEDGE: &str = "\
你是一个算法知识提取专家。从题目信息中提取涉及的算法和数据结构知识点。

要求：
1. content 使用 Markdown 格式，数学公式用 LaTeX（如 $O(n)$, $$\\\\sum_{i=1}^{n}$$）
2. 每个知识点包含：核心思路、时间/空间复杂度、常见变体
3. 只返回 JSON，不要任何其他文字

输出格式：
{
  \"knowledge_points\": [
    {\"title\": \"知识点名称\", \"content\": \"Markdown 格式，含 LaTeX 公式\"}
  ]
}";

// ─── User prompt builders ───────────────────────────────────────────

/// Build the main analysis prompt for the classifier agent.
pub fn build_classifier_prompt(ctx: &AnalysisContext) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "## 题目信息\n- 标题：{}\n- 来源：{}\n- 难度：{}\n",
        ctx.problem_title,
        ctx.problem_source,
        ctx.problem_difficulty
            .map_or("未知".to_string(), |d| d.to_string()),
    ));

    if let Some(stmt) = &ctx.problem_statement {
        let truncated: String = stmt.chars().take(2000).collect();
        prompt.push_str(&format!("- 题面：\n{}\n", truncated));
    }

    let total = ctx.submissions.len();
    let ac_count = ctx.submissions.iter().filter(|s| s.verdict == "AC").count();
    prompt.push_str(&format!(
        "\n## 提交统计\n- 总提交数：{}\n- AC 数：{}\n- AC 率：{:.1}%\n",
        total,
        ac_count,
        if total > 0 {
            ac_count as f64 / total as f64 * 100.0
        } else {
            0.0
        },
    ));

    prompt.push_str("\n## 提交记录（第一条为完整代码，后续为与前一条的 diff）\n\n");
    prompt.push_str(&build_diff_chain(&ctx.submissions));

    if !ctx.past_analyses.is_empty() {
        prompt.push_str("\n## 过往 AI 分析结果\n");
        for (i, a) in ctx.past_analyses.iter().enumerate() {
            prompt.push_str(&format!("分析 {}：{}\n", i + 1, a));
        }
    }

    if !ctx.existing_knowledge.is_empty() {
        prompt.push_str(&format!(
            "\n## 已有知识点\n{}\n",
            ctx.existing_knowledge.join("、"),
        ));
    }

    prompt
}

/// Build the template extraction prompt.
pub fn build_template_prompt(
    brief: &ClassificationBrief,
    ac_codes: &[SubmissionSummary],
    existing_templates: &[TemplateBrief],
) -> String {
    let brief_str = classification_brief(brief);
    let mut prompt = format!("## 算法信息\n{}\n\n", brief_str);

    // List existing templates as "skill summaries"
    if !existing_templates.is_empty() {
        prompt.push_str("## 现有模板库\n");
        for t in existing_templates {
            prompt.push_str(&format!("- [ID:{}] {} — {}\n", t.id, t.title, t.summary));
        }
        prompt.push('\n');
    }

    // Show AC code
    prompt.push_str("## AC 提交代码\n");
    for s in ac_codes {
        prompt.push_str(&format!(
            "\n### 提交 #{} ({})\n```{}\n{}\n```\n",
            s.id, s.language, s.language, s.code,
        ));
    }

    prompt
}

/// Build the error analysis prompt.
pub fn build_error_prompt(
    brief: &ClassificationBrief,
    submissions: &[SubmissionSummary],
) -> String {
    let non_ac: Vec<&SubmissionSummary> =
        submissions.iter().filter(|s| s.verdict != "AC").collect();
    if non_ac.is_empty() {
        return "没有非 AC 提交，无需分析错误模式。返回 {\"errors\": []}".to_string();
    }
    let brief_str = classification_brief(brief);
    let mut prompt = format!("## 算法信息\n{}\n\n## 非 AC 提交代码\n", brief_str);
    prompt.push_str(&build_error_code_blocks(submissions));
    prompt
}

/// Build the knowledge extraction prompt.
pub fn build_knowledge_prompt(brief: &ClassificationBrief, ctx: &AnalysisContext) -> String {
    let brief_str = classification_brief(brief);
    let mut prompt = format!(
        "## 算法信息\n{}\n\n## 题目标题：{}\n",
        brief_str, ctx.problem_title,
    );
    if let Some(stmt) = &ctx.problem_statement {
        let truncated: String = stmt.chars().take(1500).collect();
        prompt.push_str(&format!("## 题面\n{}\n", truncated));
    }
    prompt
}
