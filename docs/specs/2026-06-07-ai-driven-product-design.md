# ACMind 产品设计文档

**日期**：2026-06-07
**目标**：重新定义 ACMind 的产品方向，从手动 CRUD 表单转向 AI 驱动的低用户摩擦训练知识库

---

## 1. 问题与现状

### 当前实现的问题

ACMind 目前是一个**手动 CRUD 应用**：用户手动创建 Problem、手动填写 Submission（包括 verdict、code）、手动撰写 Knowledge。这和用 Excel 记录没有本质区别，用户摩擦极高。

### 核心洞察

竞品编程训练的场景有一个关键特点：**数据格式高度结构化**。
- 题目有固定的字段：来源、题号、标题、难度、标签
- 提交有固定的格式：语言、代码、判题结果、耗时/内存
- 知识点可以被分类：算法模板、解题技巧、复盘笔记、代码片段

这意味着 AI 可以在这个垂类场景中做到**极高质量的自动化**。

---

## 2. 产品愿景

**ACMind = AI 驱动的个人竞品编程训练知识库**

用户只需要做一件事：**提交代码**（甚至这一步也可以自动化）。其余所有工作由 AI 完成：

| 用户行为 | AI 自动完成 |
|---------|------------|
| 粘贴一段 AC 代码 | 自动识别算法类型、提取解题模板、打标签 |
| 导入 OJ 提交记录 | 自动建 Problem、解析判题结果、关联题目 |
| 持续积累提交 | 自动蒸馏出知识点熟练度、能力画像 |
| 查看分析报告 | 自动生成薄弱环节识别、训练建议 |

### 用户画像

- **主要用户**：ACM/ICPC 集训队员（如深圳技术大学 ACM 集训队）
- **使用场景**：每天训练后，提交代码到 ACMind，AI 自动整理出知识沉淀
- **核心价值**：不改变用户的训练习惯（还是在 OJ 上做题），但增加了知识沉淀层

---

## 3. AI 自动化流水线

### 3.1 代码分析 → 知识提取

用户提交一段代码（AC 或 WA），AI 自动完成：

```
输入：代码 + 题目信息 + 判题结果
  ↓
AI 分析：
  - 识别算法类型（DP / 图论 / 数据结构 / 数论 / ...）
  - 识别解题模式（背包 / 最短路 / 线段树 / ...）
  - 提取代码中的模板片段（可复用的算法骨架）
  - 如果是 WA：分析可能的错误原因
  ↓
输出：
  - 自动打标签（algorithm/dp/knapsack）
  - 自动生成 Knowledge 条目（kind=template 或 technique）
  - 关联到 Problem
```

### 3.2 知识蒸馏 → 能力画像

积累足够数据后，AI 自动蒸馏：

```
输入：用户所有 Problem + Submission + Knowledge
  ↓
AI 蒸馏：
  - 每个知识点的熟练度（AC 率、平均用时、是否有模板）
  - 薄弱环节识别（某类题 WA 率高、某知识点无模板）
  - 训练趋势（最近在练什么、忽略了什么）
  - 与目标的差距（对标 ICPC 区域赛水平，还差什么）
  ↓
输出：
  - 用户能力画像（雷达图 / 熟练度矩阵）
  - 个性化训练建议
  - 知识点掌握度评分
```

### 3.3 OJ 导入 → 零手动录入

```
输入：OJ 用户名 / 提交页面 URL / 批量 JSON
  ↓
AI + 解析器：
  - 自动抓取提交记录（Codeforces API / AtCoder API / VJudge 导出）
  - 自动建 Problem（标题、难度、标签）
  - 自动建 Submission（代码、判题结果）
  - 触发 3.1 的知识提取流水线
  ↓
输出：完整的训练数据 + 自动生成的知识库
```

---

## 4. 数据模型扩展

### 现有模型（保持不变）

- `user` — 用户
- `problem` — 题目
- `submission` — 提交记录
- `knowledge` — 知识条目
- `tag` — 标签
- `problem_tag` / `knowledge_tag` — 多对多关联

### 需要新增的模型

#### `skill_profile` — 用户能力画像

```sql
CREATE TABLE skill_profile (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    category    TEXT NOT NULL,          -- 'dp', 'graph', 'data_structure', 'math', ...
    proficiency REAL NOT NULL DEFAULT 0, -- 0.0 ~ 1.0 熟练度评分
    ac_count    INT NOT NULL DEFAULT 0,  -- 该分类 AC 题数
    total_count INT NOT NULL DEFAULT 0,  -- 该分类总题数
    last_trained_at TIMESTAMPTZ,         -- 最近一次训练该分类的时间
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, category)
);
```

#### `ai_analysis` — AI 分析结果

```sql
CREATE TABLE ai_analysis (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL,  -- 'submission', 'problem', 'profile'
    target_id   BIGINT,        -- 关联的 submission/problem id（profile 时为 NULL）
    analysis    JSONB NOT NULL, -- AI 分析结果（结构化 JSON）
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### `import_job` — 导入任务

```sql
CREATE TABLE import_job (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    source      TEXT NOT NULL,       -- 'codeforces', 'atcoder', 'vjudge', 'manual'
    status      TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'done', 'failed'
    payload     JSONB,               -- 导入参数（用户名、URL 等）
    result      JSONB,               -- 导入结果摘要
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
```

### 对 Submission 模型的扩展

```sql
ALTER TABLE submission ADD COLUMN ai_analyzed BOOLEAN NOT NULL DEFAULT FALSE;
-- 标记该提交是否已被 AI 分析过
```

---

## 5. API 设计（新增端点）

### AI 分析

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/ai/analyze-submission/:id` | 触发 AI 分析某个提交 |
| POST | `/api/v1/ai/analyze-batch` | 批量分析未处理的提交 |
| GET  | `/api/v1/ai/skill-profile` | 获取用户能力画像 |
| GET  | `/api/v1/ai/recommendations` | 获取 AI 训练建议 |

### 导入

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/import/codeforces` | 导入 Codeforces 提交记录 |
| POST | `/api/v1/import/atcoder` | 导入 AtCoder 提交记录 |
| POST | `/api/v1/import/json` | 导入 JSON 格式的通用数据 |
| GET  | `/api/v1/import/jobs` | 查看导入任务列表 |
| GET  | `/api/v1/import/jobs/:id` | 查看导入任务状态 |

### 能力画像（增强版 Analysis）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/analysis/skill-radar` | 能力雷达图数据 |
| GET | `/api/v1/analysis/proficiency-matrix` | 知识点熟练度矩阵 |
| GET | `/api/v1/analysis/weak-spots` | 薄弱环节识别 |
| GET | `/api/v1/analysis/training-suggestions` | 个性化训练建议 |

---

## 6. 前端页面设计

### Dashboard（重设计）

当前 Dashboard 只显示 4 个统计卡片。重新设计为：

```
┌─────────────────────────────────────────────────┐
│  Dashboard                                      │
├────────────┬────────────┬────────────┬──────────┤
│  总题数     │  AC 率      │  连续训练   │  知识条目 │
├────────────┴────────────┴────────────┴──────────┤
│  能力雷达图（Recharts RadarChart）               │
│       DP                                         │
│      / \                                         │
│   图论   数据结构                                 │
│      \ /                                         │
│    数学                                          │
├─────────────────────────────────────────────────┤
│  AI 训练建议（最近生成的 3 条）                   │
│  • "你的 DP 熟练度 0.7，建议加强区间 DP 练习"     │
│  • "图论最短路 AC 率 45%，推荐 5 道同类题"        │
│  • "最近 7 天没有练习数论，建议保持平衡"           │
├─────────────────────────────────────────────────┤
│  最近提交（时间线）                              │
└─────────────────────────────────────────────────┘
```

### Knowledge 页面（重设计）

当前是手动 CRUD 列表。重设计为 AI 生成 + 人工编辑：

```
┌─────────────────────────────────────────────────┐
│  Knowledge Base                    [导入] [AI整理]│
├─────────────────────────────────────────────────┤
│  筛选：[全部] [模板] [技巧] [笔记] [代码片段]     │
│  标签：[DP] [图论] [数据结构] [数论] ...          │
├─────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐        │
│  │ 📝 背包 DP 模板          [auto]     │        │
│  │ 关联：3 道题 | 标签：DP, 背包        │        │
│  │ "0-1 背包的状态转移方程..."          │        │
│  │ [编辑] [删除]                        │        │
│  └─────────────────────────────────────┘        │
│  ┌─────────────────────────────────────┐        │
│  │ 🔧 Dijkstra 最短路模板   [auto]     │        │
│  │ ...                                  │        │
│  └─────────────────────────────────────┘        │
└─────────────────────────────────────────────────┘
```

---

## 7. AI 集成技术方案

### 调用方式

通过 HTTP 调用 LLM API（如 OpenAI / Claude / 本地模型），不把模型嵌入 Rust 后端。

```
Rust 后端 → HTTP → LLM API → 解析 JSON 响应 → 存入数据库
```

### Prompt 设计示例

**代码分析 Prompt**：
```
你是一个竞品编程分析助手。分析以下提交代码：

题目：{problem.title}（难度：{problem.difficulty}）
判题结果：{submission.verdict}
语言：{submission.language}
代码：
{submission.code}

请返回 JSON：
{
  "algorithm_type": "dp",
  "sub_type": "knapsack",
  "tags": ["dp", "knapsack", "0-1"],
  "template_snippet": "核心代码片段",
  "technique_summary": "解题思路摘要",
  "error_analysis": "如果是WA，分析可能的错误原因",
  "difficulty_assessment": "对用户来说的实际难度评估"
}
```

**能力蒸馏 Prompt**：
```
基于用户的训练数据，生成能力画像：

最近 30 天提交统计：
{summary_json}

知识点分布：
{tag_distribution}

请返回 JSON：
{
  "skills": [
    {"category": "dp", "proficiency": 0.75, "trend": "improving"},
    {"category": "graph", "proficiency": 0.45, "trend": "stable"},
    ...
  ],
  "weak_spots": [
    {"category": "math", "reason": "AC率低且练习量少", "suggested_problems": 5},
    ...
  ],
  "recommendations": [
    "建议本周重点练习区间DP和树形DP",
    "图论最短路部分建议回顾Dijkstra模板",
    ...
  ]
}
```

### 异步处理

AI 分析是耗时操作，应该异步处理：

```
用户触发分析 → 写入任务队列 → 后台 worker 消费 → 调用 LLM → 写入结果
                                         ↓
                                    WebSocket 推送通知前端
```

技术选型：
- 任务队列：用 PostgreSQL 的 `LISTEN/NOTIFY` + 轮询（简单方案），或引入 Redis（复杂方案）
- 初期建议：直接 tokio::spawn 异步任务，不引入额外中间件

---

## 8. 实现路线图

### Phase 1：AI 代码分析（核心价值）

1. 新增 `ai_analysis` 表和 migration
2. 新增 `ai` 模块（service + route）
3. 实现 `POST /ai/analyze-submission/:id`：
   - 读取 submission + problem 数据
   - 构造 prompt，调用 LLM API
   - 解析响应，自动生成 Knowledge 条目 + Tag
   - 标记 submission 为已分析
4. 前端：Submission 列表加"AI 分析"按钮
5. 前端：Knowledge 列表区分"AI 生成"和"手动创建"

### Phase 2：能力画像

1. 新增 `skill_profile` 表和 migration
2. 实现 `POST /ai/analyze-batch`：批量分析 + 蒸馏能力画像
3. 实现 `GET /ai/skill-profile` 和 `GET /ai/recommendations`
4. 前端：Dashboard 加雷达图 + 训练建议卡片
5. 前端：新增 SkillProfile 页面

### Phase 3：OJ 导入

1. 新增 `import_job` 表和 migration
2. 实现 Codeforces API 对接（公开 API，无需 OAuth）
3. 实现通用 JSON 导入
4. 前端：导入页面 + 任务进度展示
5. 导入完成后自动触发 AI 分析流水线

### Phase 4：智能检索与推荐

1. 知识库全文搜索（PostgreSQL `tsvector`）
2. 基于能力画像的题目推荐
3. 训练计划自动生成

---

## 9. 与现有代码的关系

### 保留的

- 所有现有 Entity、Migration、CRUD 模块 — 它们是 AI 的数据基础
- Auth、限流、Docker Compose、CI — 工程基础设施
- Analysis 模块 — 基础统计，AI 分析是上层

### 需要新增的

- `ai` 模块（分析 service + LLM 调用）
- `import` 模块（OJ 数据导入）
- `skill_profile` 相关的 migration + entity + route
- 前端能力画像页面、AI 分析交互

### 需要重构的

- Submission 模型加 `ai_analyzed` 字段
- Knowledge 模型加 `source` 字段（`ai_generated` vs `manual`）
- Dashboard 页面重设计
- Knowledge 页面重设计

---

## 10. 面试价值

这套设计在面试中可以聊的点：

| 话题 | 怎么聊 |
|------|--------|
| AI 工程化 | "不是调 API 就完了——prompt 设计、结构化输出解析、异步任务队列、失败重试" |
| 数据建模 | "能力画像怎么量化？proficiency 0~1 怎么算？tags 粒度怎么定？" |
| 异步架构 | "AI 分析是耗时操作，用 tokio::spawn + WebSocket 推送，不阻塞主请求" |
| 垂类 AI | "竞品编程格式高度结构化，AI 可以做到比通用场景更高的准确率" |
| 产品思维 | "核心价值是降低用户摩擦——用户只需要提交代码，AI 做剩下的事" |
