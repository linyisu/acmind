# AI 分析系统优化方案

> 日期：2026-06-10 | 状态：设计中

## 核心改动：Diff 提交表示

### 设计思路

**现状：** 把每条提交的完整代码塞进 prompt（20 条 × 2000 字符 = 40K）

**优化：** 按时间排序，第一条传完整代码，后续传与前一条的 diff

```
提交时间线：
  #1 WA  ── 完整代码（200行）
  #2 WA  ── diff #1→#2（改了15行）
  #3 WA  ── diff #2→#3（改了3行）
  #4 AC  ── diff #3→#4（改了8行）

总计：~230 行 vs 当前 ~800 行
```

**收益：**
- token 体积减少 60-70%
- LLM 能直接看到「从 WA 到 AC 改了什么」，分析更精准
- 错误模式在 diff 中更明显（反复出现的修改就是典型 bug）

### Diff 生成方案

使用 `similar` crate（Rust 生态成熟的 diff 库），生成 unified diff 格式：

```toml
# Cargo.toml
similar = "2"
```

```rust
use similar::{ChangeTag, TextDiff};

fn build_submission_diffs(submissions: &[SubmissionSummary]) -> Vec<SubmissionBlock> {
    // 按 submitted_at 排序（最早的在前）
    let sorted = submissions; // 已排序

    sorted.iter().enumerate().map(|(i, sub)| {
        if i == 0 {
            // 第一条：完整代码
            SubmissionBlock {
                id: sub.id,
                verdict: &sub.verdict,
                language: &sub.language,
                runtime_ms: sub.runtime_ms,
                content: format!("```{}\n{}\n```", sub.language, sub.code),
            }
        } else {
            // 后续：diff against previous
            let prev = &sorted[i - 1];
            let diff = TextDiff::from_lines(&prev.code, &sub.code);
            let unified: String = diff
                .unified_diff()
                .context_radius(3)
                .header(&format!("提交 #{}", prev.id), &format!("提交 #{}", sub.id))
                .to_string();

            SubmissionBlock {
                id: sub.id,
                verdict: &sub.verdict,
                language: &sub.language,
                runtime_ms: sub.runtime_ms,
                content: format!("```diff\n{}\n```", unified),
            }
        }
    }).collect()
}
```

Prompt 中的呈现方式：

```
## 提交 #1 (WA, cpp, 120ms)
```cpp
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n; cin >> n;
    // ... 完整代码 ...
}
```

## 提交 #2 (WA → 提交 #1, cpp, 95ms)
```diff
--- 提交 #1
+++ 提交 #2
@@ -15,7 +15,7 @@
     for (int i = 0; i <= n; i++) {  // 边界错误
-        dp[i] = dp[i-1] + dp[i-2];
+        dp[i] = dp[max(0,i-1)] + dp[max(0,i-2)];
     }
```

## 提交 #3 (AC → 提交 #2, cpp, 48ms)
```diff
--- 提交 #2
+++ 提交 #3
@@ -12,4 +12,4 @@
-    for (int i = 0; i <= n; i++) {
+    for (int i = 2; i <= n; i++) {
```
```

### 模板提取的特殊处理

模板提取只需要最终 AC 代码（完整），不需要 diff：

```
Stage 2A（模板提取）：只传最近 1 条 AC 的完整代码
Stage 2B（错误分析）：传全部提交的 diff 链（WA→WA→AC 的变化过程）
Stage 2C（知识点）  ：传题目信息 + Stage1 的 algorithm_type
```

---

## Prompt 优化：强化输出 + LaTeX

### 要求 LaTeX 的理由

前端已有 `react-markdown` + `remark-math` + `rehype-katex`，但 LLM 当前不输出 LaTeX。
复杂度、数学公式等应该用 LaTeX 表示。

### System prompt 改写

**Stage 1 — 综合分析：**

```rust
const SYSTEM_ANALYSIS: &str = "\
你是一个算法训练分析专家。分析一个题目的提交历史，给出综合评估。

要求：
1. summary 使用 Markdown 格式，数学公式用 LaTeX（行内 $O(n)$，独立公式用 $$...$$）
2. algorithm_type 从以下选择：dp, graph, string, data_structure, math, geometry, greedy, search, sort, binary_search, other
3. suggested_difficulty 1-5 整数
4. 只返回 JSON，不要任何其他文字

输出格式：
{
  \"algorithm_type\": \"...\",
  \"sub_type\": \"...\",
  \"tags\": [\"...\"],
  \"summary\": \"Markdown 格式分析，包含 LaTeX 公式\",
  \"difficulty_analysis\": \"难度评估（Markdown）\",
  \"progress_notes\": \"学习进步分析（Markdown）\",
  \"suggested_difficulty\": 3
}";
```

**Stage 2A — 模板提取：**

```rust
const SYSTEM_TEMPLATE: &str = "\
你是一个算法模板提取专家。从 AC 代码中提取可复用的代码模板。

要求：
1. 去除题目特有的输入输出部分（读入/输出），只保留核心算法
2. category 必须是以下之一：data_structure, graph, dp, string, math, geometry, greedy, search, sort, binary_search, other
3. time_complexity 使用 LaTeX 格式，如 \"$O(n \\log n)$\"
4. description 使用 Markdown，包含适用场景和注意事项
5. 如果代码中没有值得提取的通用模板，返回空数组
6. 只返回 JSON

输出格式：
{\"templates\": [{\"title\": \"...\", \"code\": \"...\", \"description\": \"...\", \"category\": \"...\", \"time_complexity\": \"$O(n)$\"}]}";
```

**Stage 2B — 错误分析：**

```rust
const SYSTEM_ERROR: &str = "\
你是一个编程错误分析专家。分析提交历史中的 diff，找出反复出现的错误模式。

要求：
1. 重点关注 diff 中反复修改的部分——那是典型的 bug 热点
2. description 说明错误原因和触发条件
3. fix_suggestion 给出具体修复方法
4. 如果没有明显的错误模式，返回空数组
5. 只返回 JSON

输出格式：
{\"errors\": [{\"title\": \"...\", \"description\": \"...\", \"fix_suggestion\": \"...\"}]}";
```

**Stage 2C — 知识点提取：**

```rust
const SYSTEM_KNOWLEDGE: &str = "\
你是一个算法知识提取专家。从题目信息中提取涉及的算法和数据结构知识点。

要求：
1. content 使用 Markdown 格式，数学公式用 LaTeX（如 $O(n)$, $$\\sum_{i=1}^{n}$$）
2. 每个知识点包含：核心思路、时间/空间复杂度、常见变体
3. 只返回 JSON

输出格式：
{\"knowledge_points\": [{\"title\": \"...\", \"content\": \"Markdown 格式，含 LaTeX\"}]}";
```

---

## 其他优化项

### LLM 调用层：重试 + timeout

在 `provider.rs` 的 `OpenAiProvider::chat` 中增加：

```rust
const MAX_RETRIES: u32 = 2;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

for attempt in 0..=MAX_RETRIES {
    let resp = self.client
        .post(format!("{}/chat/completions", self.base_url))
        .header("Authorization", format!("Bearer {}", self.api_key))
        .timeout(REQUEST_TIMEOUT)
        .json(&req)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => { /* parse and return */ }
        Ok(r) if r.status().as_u16() == 429 || r.status().as_u16() >= 500 => {
            if attempt < MAX_RETRIES {
                let delay = Duration::from_millis(1000 * 2u64.pow(attempt));
                tokio::time::sleep(delay).await;
                continue;
            }
        }
        // 其他错误直接返回
    }
}
```

### 修复 NoopProvider

更新所有 mock 响应：
- `ExtractedTemplate` 增加 `category` + `time_complexity`
- `ProblemAnalysisJson` 增加 `difficulty_analysis` + `progress_notes`
- 所有复杂度字段用 LaTeX 格式（`$O(n^2)$`）

### Stage 2 prompt 精简

不再传完整 `ProblemAnalysisJson`，只提取摘要字符串：

```rust
fn analysis_brief(analysis: &ProblemAnalysisJson) -> String {
    format!(
        "算法类型: {} / {}\n标签: {}\n摘要: {}",
        analysis.algorithm_type,
        analysis.sub_type,
        analysis.tags.join(", "),
        // summary 截断到前 300 字符
        analysis.summary.chars().take(300).collect::<String>(),
    )
}
```

### past_analyses 批量查询

新增 `ai_repo::find_by_targets(db, user_id, "submission", &[...])` 一次性查询，消除 N+1。

### 代码截断改进

```rust
fn truncate_code_smart(code: &str, max_chars: usize) -> String {
    if code.len() <= max_chars { return code.to_string(); }
    let lines: Vec<&str> = code.lines().collect();
    let head: Vec<&str> = lines.iter().take(30).copied().collect();
    let tail: Vec<&str> = lines.iter().rev().take(5).rev().copied().collect();
    let mut result = head;
    if lines.len() > 35 {
        result.push("// ... ({} lines omitted) ...");
    }
    result.extend(tail);
    result.join("\n")
}
```

---

## 实施顺序

| # | 改动 | 涉及文件 | 工作量 |
|---|------|---------|--------|
| 1 | 新增 `similar` 依赖 + diff 生成函数 | `Cargo.toml`, `problem_analyzer.rs` | 20min |
| 2 | 重写 prompt（强化 + LaTeX + diff 格式） | `problem_analyzer.rs` | 20min |
| 3 | 改造 Stage 1 用户 prompt：diff 链代替全量代码 | `problem_analyzer.rs` | 15min |
| 4 | 改造 Stage 2：精简传参 + 模板只用 AC 全量 | `problem_analyzer.rs` | 15min |
| 5 | LLM 重试 + timeout | `provider.rs` | 15min |
| 6 | 修复 NoopProvider | `provider.rs` | 10min |
| 7 | past_analyses 批量查询 | `ai/repo.rs`, `problem_analyzer.rs` | 10min |
| 8 | 代码截断改进 | `problem_analyzer.rs` | 10min |
