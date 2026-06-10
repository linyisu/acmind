# AI 分析系统设计文档

**日期**：2026-06-09
**目标**：设计两层 AI 分析架构，在上下文窗口限制内实现高效的提交分析和知识沉淀

---

## 1. 背景与约束

### 核心问题

用户一次训练可能提交 10~50 次代码，单次提交平均 200~800 行。直接把所有代码丢给 AI 会超过上下文窗口（128K tokens），且成本高、速度慢。

### 设计目标

- **低延迟**：导入后秒级触发 Layer 1 分析
- **深度分析**：Layer 2 能分析同一题所有提交 + 题解，找出模式
- **跨题目复用**：同类算法（如 DP 的背包类）的分析结果缓存，不重复分析
- **可扩展**：未来接入用户反馈、题解对比、能力画像

---

## 2. 两层架构

### Layer 1：轻量级逐提交分析

**触发时机**：每次导入或提交时自动触发
**输入**：单个 submission + problem 信息
**输出**：结构化 JSON（存入 `ai_analysis` 表，`analysis_type = 'layer1'`）

```json
{
  "algorithm_type": "dp",
  "algorithm_sub_type": "0-1_knapsack",
  "code_summary": "使用状态转移 dp[j] = max(dp[j], dp[j-w]+v)，空间优化为一维",
  "quality_signal": "ac_clean",
  "error_hint": null,
  "verdict_confidence": "high",
  "tags": ["dp", "knapsack", "greedy"]
}
```

quality_signal 取值：ac_clean | ac_messy | wa_pattern | tle_pattern | re_pattern
error_hint：仅 wa/tle/re 时有值

**Token 预算**：~3-7K tokens（系统提示 + 代码 + 题目信息）

**关键设计**：
- `code_summary` 将 200~800 行代码压缩为 100~200 token 的摘要
- `quality_signal` 提供结构化的质量信号，便于后续聚合
- `algorithm_sub_type` 用于缓存键和跨题目复用

### Layer 2：深度对比分析

**触发时机**：
- 同一题的 Layer 1 分析 >= 3 个提交后自动触发
- 用户提供题解（editorial）后触发
- 用户手动请求深度分析时触发

**输入**：压缩后的 Layer 1 摘要 + 缓存的算法模板 + 用户反馈（可选）
**输出**：结构化 JSON（存入 `ai_analysis` 表，`analysis_type = 'layer2'`）

```json
{
  "submission_pattern_analysis": {
    "common_mistakes": ["边界条件处理不当", "状态转移方向错误"],
    "improvement_trajectory": "从暴力递归 -> 记忆化搜索 -> DP table -> 空间优化",
    "bad_habits": ["习惯性忽略初始化", "调试时加过多 print 不删"]
  },
  "algorithm_insight": {
    "key_technique": "背包问题的核心是状态定义和转移方向",
    "template_snippet": "for i in 0..n: for j in W..w[i]: dp[j] = max(dp[j], dp[j-w[i]]+v[i])",
    "common_pitfalls": ["一维数组必须逆序遍历", "完全背包正序遍历"]
  },
  "editorial_comparison": null,
  "suggested_next_steps": ["练习完全背包", "练习分组背包"],
  "difficulty_assessment": 3
}
```

**Token 预算**：~5-10K tokens（压缩后的 Layer 1 摘要 + 缓存 + 提示词）

**压缩策略**：
- Layer 1 的 `code_summary` 替代原始代码（200 行 -> 100 token）
- 同一题 10 个提交的摘要约 1000 tokens，远低于原始的 20K tokens
- 缓存的算法模板/错误模式约 500 tokens

---

## 3. 缓存体系

### 3.1 算法类型缓存（AlgorithmTypeCache）

**表**：`algorithm_type_cache`

```sql
CREATE TABLE algorithm_type_cache (
    id              BIGSERIAL PRIMARY KEY,
    algorithm_type  TEXT NOT NULL,
    sub_type        TEXT NOT NULL,
    template        TEXT,
    error_patterns  JSONB,
    key_insights    JSONB,
    problem_count   INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(algorithm_type, sub_type)
);
```

**更新时机**：Layer 2 完成后合并新发现的模板/错误模式；每 10 个同类型题目触发增量更新。

### 3.2 题目分析缓存（ProblemAnalysisCache）

**表**：`problem_analysis_cache`

```sql
CREATE TABLE problem_analysis_cache (
    id                   BIGSERIAL PRIMARY KEY,
    user_id              BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    problem_id           BIGINT NOT NULL REFERENCES problem(id) ON DELETE CASCADE,
    submission_summaries JSONB,
    layer2_result        JSONB,
    editorial_digest     TEXT,
    user_feedback        TEXT,
    submission_count     INT NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, problem_id)
);
```

### 3.3 知识条目（KnowledgeEntry）

扩展现有 `knowledge` 表：

```sql
ALTER TABLE knowledge ADD COLUMN algorithm_type TEXT;
ALTER TABLE knowledge ADD COLUMN sub_type TEXT;
ALTER TABLE knowledge ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
```

---

## 4. 分析流水线

### 4.1 Layer 1 流程

```
用户导入/提交代码
    |
    +-- 检查 submission 是否已分析（跳过已分析的）
    |
    +-- 读取 problem 信息（标题、难度、描述）
    |
    +-- 构造 Layer 1 prompt
    |   - system: 分析角色定义 + 输出 JSON schema
    |   - user: problem 信息 + submission 代码
    |
    +-- 调用 LLM -> 解析 JSON
    |
    +-- 存入 ai_analysis (type=layer1)
    |
    +-- 检查该题 Layer 1 数量
    |   +-- >= 3 -> 触发 Layer 2（异步）
    |
    +-- 返回 Layer 1 结果给前端
```

### 4.2 Layer 2 流程

```
触发条件（>=3 提交 / 有题解 / 手动请求）
    |
    +-- 读取该题所有 Layer 1 分析
    |
    +-- 压缩为 submission_summaries
    |   - 合并相似提交
    |   - 保留 AC/WA 模式差异
    |   - 提取时间序列（从 WA 到 AC 的改进路径）
    |
    +-- 查询 AlgorithmTypeCache
    |   - 获取该算法类型的模板和错误模式
    |   - 如无缓存，标记为首次分析
    |
    +-- 读取用户反馈和题解（如果有）
    |
    +-- 构造 Layer 2 prompt
    |   - system: 对比分析角色定义 + 输出 JSON schema
    |   - user: 压缩后的提交摘要 + 缓存 + 反馈
    |
    +-- 调用 LLM -> 解析 JSON
    |
    +-- 存入 ai_analysis (type=layer2)
    |
    +-- 更新 ProblemAnalysisCache
    |
    +-- 增量更新 AlgorithmTypeCache
    |
    +-- 更新/创建 KnowledgeEntry
```

---

## 5. Prompt 工程

### 5.1 Layer 1 Prompt

**System**：
```
你是一个竞品编程分析助手。分析单个提交代码，输出结构化 JSON。

要求：
1. algorithm_type: dp / graph / data_structure / math / string / geometry / greedy / brute_force / other
2. algorithm_sub_type: 细分到具体算法类型（如 dp 有：0-1_knapsack, complete_knapsack, LIS, LCS, interval_dp, tree_dp, digit_dp, bitmask_dp 等）
3. code_summary: 100~200 字，描述算法思路和关键实现
4. quality_signal: ac_clean | ac_messy | wa_pattern | tle_pattern | re_pattern
5. error_hint: 仅 wa/tle/re 时分析可能错误原因
6. tags: 2~5 个标签

输出严格 JSON，不要有其他内容。
```

**User**：
```
题目：{title}（难度：{difficulty}）
题目描述：{description 截取前 500 字}

提交：
- 语言：{language}
- 判题结果：{verdict}
- 代码：
{code}
```

### 5.2 Layer 2 Prompt

**System**：
```
你是一个竞品编程深度分析助手。基于用户对该题的多次提交，进行对比分析。

已知信息：
- 该算法类型的模板和常见错误：{cache_data}
- 用户提供的题解：{editorial}

分析维度：
1. 提交模式分析：用户的改进路径、反复犯的错误、代码习惯
2. 算法洞察：该算法的核心技巧、常见陷阱
3. 与题解对比（如有）：用户思路与标准解法的差异
4. 建议：下一步应该练什么

输出严格 JSON，不要有其他内容。
```

**User**：
```
题目：{title}（难度：{difficulty}）

用户对该题的提交摘要：
{submission_summaries}

用户反馈（如有）：
{user_feedback}
```

---

## 6. 错误处理与降级

- **LLM 调用失败**：Layer 1 不影响导入流程，Layer 2 下次有新提交时重试。指数退避最多 3 次。
- **JSON 解析失败**：记录原始响应到 `ai_analysis.raw_response`，尝试正则提取 JSON 块。
- **缓存一致性**：`algorithm_type_cache` 过期（> 7 天）时重新分析。

---

## 7. 数据库变更

### 新增表

1. `algorithm_type_cache`
2. `problem_analysis_cache`

### 修改表

**ai_analysis** 添加字段：
- `analysis_type TEXT NOT NULL DEFAULT 'layer1'`
- `raw_response TEXT`
- `tokens_used INT`

**knowledge** 添加字段：
- `algorithm_type TEXT`
- `sub_type TEXT`
- `source TEXT NOT NULL DEFAULT 'manual'`

---

## 8. API 设计

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/ai/analyze/{submission_id}` | Layer 1（现有，扩展逻辑） |
| POST | `/api/v1/ai/analyze-problem/{problem_id}` | Layer 2（手动触发） |
| GET  | `/api/v1/ai/analyses` | 用户所有分析结果（支持 type 过滤） |
| GET  | `/api/v1/ai/analyses/{problem_id}` | 某题所有分析结果 |
| POST | `/api/v1/ai/feedback/{problem_id}` | 提交用户反馈 |
| POST | `/api/v1/ai/editorial/{problem_id}` | 提交题解 |
| GET  | `/api/v1/ai/test` | AI 连接测试（现有） |

---

## 9. 实现优先级

### Phase 1：Layer 1

1. 扩展 `ai_analysis` 表（migration 添加字段）
2. 重构 `analyze_submission` 实现 Layer 1 prompt 和解析
3. 前端展示 Layer 1 结果

### Phase 2：Layer 2 + 缓存

1. 新增 `algorithm_type_cache` 和 `problem_analysis_cache` 表
2. 实现 Layer 2 触发逻辑（>=3 提交自动触发）
3. 实现缓存读写
4. 前端"深度分析"按钮

### Phase 3：用户反馈

1. feedback 和 editorial API
2. Layer 2 prompt 集成用户反馈
3. 前端反馈表单

### Phase 4：知识生成

1. Layer 2 完成后自动创建/更新 Knowledge
2. 前端 Knowledge 页面展示 AI 生成内容

---

## 10. 成本与性能预估

- **Layer 1**：~5K tokens/提交 x 20 提交/题 = 100K tokens/题
- **Layer 2**：~8K tokens/次
- **GPT-4o-mini 单价**：输入 $0.15/1M，输出 $0.60/1M
- **一次完整分析（20 提交 + 1 Layer 2）**：约 $0.02
- **Layer 1 延迟**：~2-5 秒/提交
- **Layer 2 延迟**：~5-10 秒

---

## 11. 未来扩展

- **用户反馈闭环**：Layer 2 后展示结果，用户确认/纠正，纳入下次 prompt
- **跨题目知识聚合**：同一 `algorithm_sub_type` 的 Layer 2 结果聚合为知识模板
- **能力画像**：基于 Layer 1 的 `quality_signal` 和 `verdict` 聚合，生成熟练度评分
