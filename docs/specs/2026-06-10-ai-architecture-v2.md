# AI 架构 V2 + 任务看板 + 全中文化

> 日期：2026-06-10 | 状态：设计中

---

## 一、整体架构变更

### 现状

```
apps/api/src/ai/
├── mod.rs
├── model.rs          ← 所有 DTO 混在一起
├── provider.rs       ← LLM 调用抽象
├── problem_analyzer.rs  ← 950 行巨文件，4 个 stage 全在一个函数里
├── repo.rs
├── route.rs
└── service.rs
```

问题：`problem_analyzer.rs` 承担了太多职责（上下文收集、4 种 prompt 构建、3 种提取逻辑、结果保存），难以扩展。

### 目标

```
apps/api/src/ai/
├── mod.rs
├── provider.rs          ← LLM 调用抽象（不变）
├── prompt.rs            ← 所有 system prompt 常量 + prompt 构建工具函数
├── parse.rs             ← parse_llm_json 等通用解析工具
├── context.rs           ← AnalysisContext 收集、diff 生成、代码截断
├── repo.rs              ← ai_analysis 表 CRUD（不变）
├── route.rs             ← 路由（不变）
├── service.rs           ← 单次提交分析（不变）
├── orchestrator.rs      ← 全量分析编排器（原 problem_analyzer 的 run_task）
├── agents/
│   ├── mod.rs
│   ├── classifier.rs    ← 分类 Agent：判断题目算法类型
│   ├── template.rs      ← 模板 Agent：提取模板（含现有模板匹配）
│   ├── error.rs         ← 错误 Agent：分析错误模式
│   └── knowledge.rs     ← 知识 Agent：提取知识点
└── model.rs             ← 所有 DTO（不变）
```

### 核心设计原则

**每个 Agent 是一个独立的 struct + async fn**，拥有：
- 自己的 system prompt（中文）
- 自己的 LLM 调用逻辑
- 自己的结果类型
- 自己的进度上报回调

```rust
// agents/template.rs 示例
pub struct TemplateAgent;

pub struct TemplateAgentInput<'a> {
    pub algorithm_type: &'a str,
    pub ac_submissions: &'a [SubmissionSummary],
    pub existing_templates: &'a [TemplateBrief],  // 新增：现有模板摘要
}

pub struct TemplateAgentOutput {
    pub templates: Vec<ExtractedTemplate>,
    pub matched_templates: Vec<i64>,  // 复用了哪些现有模板的 ID
}

impl TemplateAgent {
    pub async fn run(
        &self,
        llm: &dyn LlmProvider,
        input: &TemplateAgentInput<'_>,
        on_progress: impl Fn(&str),  // 进度回调
    ) -> AppResult<TemplateAgentOutput> {
        on_progress("正在匹配现有模板...");
        // ... 检查现有模板 ...
        on_progress("正在从 AC 代码中提取模板...");
        // ... 调用 LLM ...
        Ok(...)
    }
}
```

---

## 二、Skill 机制 — 模板智能匹配

### 设计思路

现有模板应该像「技能库」一样工作：

1. 每个模板有一个 **概要（summary）** — 简短描述这个模板是什么、适用于什么场景
2. 分析新题目时，先判断算法类型 → 用类型匹配现有模板 → 把相关模板喂给 AI
3. AI 看到现有模板后，可以：
   - 直接复用（建立题目-模板关联）
   - 在现有模板基础上改进
   - 发现需要新模板时提取新的

### 模板表扩展

`template` 表新增一个字段：

| 列名 | 类型 | 说明 |
|------|------|------|
| `summary` | `VARCHAR(500)` | 模板概要，1-2 句话描述适用场景 |

Migration：`m20260610_100000_add_template_summary.rs`

### 模板匹配流程

```
                    题目提交
                       ↓
              ┌─ Classifier Agent ─┐
              │  "这是一道图论题"    │
              │  algorithm_type=graph │
              └─────────┬───────────┘
                        ↓
              ┌─ 查现有模板 ──────────────────────────┐
              │  SELECT id, title, summary, code       │
              │  FROM template                         │
              │  WHERE user_id = ? AND category = 'graph' │
              │  ORDER BY usage_count DESC LIMIT 5     │
              └─────────┬─────────────────────────────┘
                        ↓
              ┌─ Template Agent ─────────────────────────────┐
              │  输入：                                       │
              │    - 算法类型: graph                           │
              │    - AC 代码                                  │
              │    - 现有模板摘要:                             │
              │        [1] "Dijkstra 最短路模板 — 适用于..."   │
              │        [2] "BFS 层序遍历模板 — 适用于..."      │
              │        [3] "拓扑排序模板 — 适用于..."          │
              │                                               │
              │  AI 决策：                                    │
              │    - #1 匹配 → 建立关联，不重复提取            │
              │    - #2 不匹配 → 提取新模板                    │
              │    - #3 不匹配但发现了新的 → 提取新模板        │
              └─────────┬────────────────────────────────────┘
                        ↓
              输出：
                - 新提取的模板（存入 template 表）
                - 复用的现有模板 ID（建立 template_problem 关联）
```

### System Prompt 变化

```rust
const SYSTEM_TEMPLATE: &str = "\
你是一个算法模板提取专家。从 AC 代码中提取可复用的代码模板。

## 现有模板库
以下是用户已有的相关模板。如果代码可以直接复用某个现有模板，请在 matched 字段中返回其 ID。
如果现有模板不够用，需要提取新模板。

要求：
1. 去除题目特有的输入输出部分，只保留核心算法逻辑
2. category 必须是以下之一：data_structure, graph, dp, string, math, geometry, greedy, search, sort, binary_search, other
3. time_complexity 使用 LaTeX 格式，如 \"$O(n \\\\log n)$\"
4. description 使用 Markdown，包含适用场景
5. 只返回 JSON

输出格式：
{
  \"templates\": [
    {\"title\": \"...\", \"code\": \"...\", \"description\": \"...\", \"category\": \"...\", \"time_complexity\": \"...\"}
  ],
  \"matched\": [1, 3]
}";
```

---

## 三、Classifier Agent — 算法分类

### 职责

独立的分类步骤，最先运行，结果供后续所有 Agent 使用。

```rust
pub struct ClassifierAgent;

pub struct ClassifierOutput {
    pub algorithm_type: String,    // dp, graph, string, ...
    pub sub_type: String,          // knapsack, bfs, kmp, ...
    pub tags: Vec<String>,         // ["最短路", "Dijkstra", "带权图"]
    pub summary: String,           // 综合分析摘要（Markdown + LaTeX）
    pub difficulty_analysis: String,
    pub progress_notes: String,
    pub suggested_difficulty: i32,
}
```

### System Prompt

```rust
const SYSTEM_CLASSIFIER: &str = "\
你是一个算法训练分析专家。分析一个题目的提交历史（以 diff 形式呈现），判断算法类型。

要求：
1. summary 使用 Markdown，数学公式用 LaTeX（$O(n)$，$$...$$）
2. algorithm_type 从以下选择：dp, graph, string, data_structure, math, geometry, greedy, search, sort, binary_search, other
3. 重点分析提交的变化趋势和解题思路
4. suggested_difficulty 1-5 整数
5. 只返回 JSON

输出格式：
{
  \"algorithm_type\": \"...\",
  \"sub_type\": \"...\",
  \"tags\": [\"...\"],
  \"summary\": \"Markdown 综合分析\",
  \"difficulty_analysis\": \"难度评估\",
  \"progress_notes\": \"学习进步分析\",
  \"suggested_difficulty\": 3
}";
```

---

## 四、Orchestrator — 编排器

原 `problem_analyzer.rs` 的 `run_task` 重构为编排器，职责变为「协调各 Agent + 管理进度」：

```rust
// orchestrator.rs
pub async fn run_task(state: &AppState, task_id: i64, user_id: i64, problem_id: i64) -> AppResult<()> {
    let ctx = collect_context(&state.db, user_id, problem_id).await?;

    // Stage 1: 分类
    report_agent(task_id, "题目分析官", "running", "正在分析题目类型...").await;
    let classifier = ClassifierAgent;
    let classification = classifier.run(&*state.llm, &ctx, |msg| {
        report_agent(task_id, "题目分析官", "running", msg).await;
    }).await?;
    report_agent(task_id, "题目分析官", "completed", &format!("识别为 {} 算法", classification.algorithm_type)).await;

    // Stage 2: 查现有模板
    let existing_templates = template_repo::list(&state.db, user_id, &ListTemplatesQuery {
        category: Some(classification.algorithm_type_as_category()),
        ..Default::default()
    }).await?;
    let template_briefs: Vec<TemplateBrief> = existing_templates.iter().map(|t| TemplateBrief {
        id: t.id,
        title: t.title.clone(),
        summary: t.summary.clone().unwrap_or_default(),
    }).collect();

    // Stage 3: 并行提取（模板、错误、知识）
    let (template_res, error_res, knowledge_res) = tokio::join!(
        async {
            report_agent(task_id, "模板提取官", "running", "正在提取代码模板...").await;
            TemplateAgent.run(&*state.llm, &TemplateAgentInput {
                algorithm_type: &classification.algorithm_type,
                ac_submissions: &ac_codes,
                existing_templates: &template_briefs,
            }, |msg| { /* update progress */ }).await
        },
        async {
            report_agent(task_id, "错误诊断官", "running", "正在分析错误模式...").await;
            ErrorAgent.run(&*state.llm, &ctx.submissions, &classification).await
        },
        async {
            report_agent(task_id, "知识梳理官", "running", "正在提取知识点...").await;
            KnowledgeAgent.run(&*state.llm, &ctx, &classification).await
        },
    );

    // Stage 4: 保存
    report_agent(task_id, "数据存档官", "running", "正在保存结果...").await;
    // ... 保存逻辑 ...

    Ok(())
}
```

---

## 五、任务系统升级

### Progress 格式扩展

当前的 `progress` 是一个扁平的 step 数组。升级为 **Agent 级别** 的进度追踪：

```json
{
  "agents": [
    {
      "id": "classifier",
      "name": "题目分析官",
      "icon": "🔍",
      "status": "completed",
      "message": "识别为 graph 算法",
      "steps": [
        {"label": "收集提交记录", "status": "completed", "detail": "23 条提交"},
        {"label": "分析算法类型", "status": "completed", "detail": "graph / 最短路"}
      ]
    },
    {
      "id": "template",
      "name": "模板提取官",
      "icon": "🔧",
      "status": "running",
      "message": "正在匹配现有模板...",
      "steps": [
        {"label": "匹配现有模板", "status": "completed", "detail": "找到 2 个相关模板"},
        {"label": "分析 AC 代码", "status": "running", "detail": ""},
        {"label": "提取新模板", "status": "pending", "detail": ""}
      ]
    },
    {
      "id": "error",
      "name": "错误诊断官",
      "icon": "🔎",
      "status": "running",
      "message": "正在分析错误模式...",
      "steps": [
        {"label": "分析非 AC 提交", "status": "running", "detail": ""}
      ]
    },
    {
      "id": "knowledge",
      "name": "知识梳理官",
      "icon": "📚",
      "status": "pending",
      "message": "",
      "steps": [
        {"label": "提取知识点", "status": "pending", "detail": ""}
      ]
    },
    {
      "id": "saver",
      "name": "数据存档官",
      "icon": "💾",
      "status": "pending",
      "message": "",
      "steps": [
        {"label": "保存结果", "status": "pending", "detail": ""}
      ]
    }
  ]
}
```

### DB 变更

`task` 表的 `progress` 字段（JSONB）格式从 `Vec<ProgressStep>` 变为新的 `TaskProgress` 结构。由于是 JSONB，无需 migration，只需变更解析逻辑。

### Agent 角色表

| Agent ID | 名称 | 图标 | 职责 |
|----------|------|------|------|
| `classifier` | 题目分析官 | 🔍 | 分析题目算法类型、综合评估 |
| `template` | 模板提取官 | 🔧 | 提取代码模板、匹配现有模板 |
| `error` | 错误诊断官 | 🔎 | 分析错误模式 |
| `knowledge` | 知识梳理官 | 📚 | 提取知识点 |
| `saver` | 数据存档官 | 💾 | 保存所有结果到数据库 |

---

## 六、任务看板页面

### 路由

`/tasks` — 新增页面，侧边栏导航

### UI 设计（类似 GitHub Actions）

```
┌──────────────────────────────────────────────────────────────┐
│ 任务中心                                                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ✅ #42  题目「POJ 3264」全量分析      3 分钟前    用时 45s  │
│     🔍 题目分析官: 识别为 data_structure 算法                │
│     🔧 模板提取官: 提取了 2 个模板（1 个复用现有）           │
│     🔎 错误诊断官: 发现 3 个错误模式                         │
│     📚 知识梳理官: 提取了 2 个知识点                         │
│     💾 数据存档官: 保存了 4 条知识条目                       │
│                                                              │
│  🔄 #43  题目「CF 1023D」全量分析     刚刚        用时 12s   │
│     ✅ 🔍 题目分析官: 识别为 dp 算法                         │
│     🔄 🔧 模板提取官: 正在匹配现有模板...                    │
│        ├─ ✅ 匹配现有模板 (找到 1 个)                        │
│        ├─ 🔄 分析 AC 代码                                    │
│        └─ ⏳ 提取新模板                                      │
│     ⏳ 🔎 错误诊断官: 等待中                                 │
│     ⏳ 📚 知识梳理官: 等待中                                 │
│     ⏳ 💾 数据存档官: 等待中                                 │
│                                                              │
│  ❌ #41  题目「HDU 1754」全量分析     1 小时前    用时 8s    │
│     🔍 题目分析官: 失败 — 没有提交记录                       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 功能细节

- 点击任务行 → 展开/折叠详细步骤
- 运行中的任务 → 自动轮询（2 秒间隔，现有逻辑）
- 点击题目名 → 跳转到 ProblemDetailPage
- 完成的任务 → 点击可查看完整分析结果（复用现有结果对话框）
- 顶部统计：今日完成 X 个、失败 X 个、进行中 X 个

### 路由和导航

```tsx
// router.tsx
<Route path="/tasks" element={<TasksPage />} />

// Sidebar.tsx
{ to: "/tasks", label: "任务中心", icon: Activity }
```

---

## 七、全中文化

### 改动范围

所有前端页面的 UI 文案统一为中文：

| 页面 | 改动项 |
|------|--------|
| LoginPage | "登录 ACMind"、"用户名"、"密码"、"没有账号？注册" |
| RegisterPage | "注册 ACMind"、"用户名"、"邮箱"、"密码" |
| DashboardPage | "仪表盘"、"题目数"、"提交数"、"知识条目"、"AC 率" |
| ProblemsListPage | "题目列表"、"新建题目"、"搜索题目..."、"全部标签" |
| ProblemDetailPage | "题面"、"提交记录"、"编辑"、"难度"、"时间" |
| ProblemFormPage | "新建题目" / "编辑题目"、所有表单标签 |
| KnowledgeListPage | "知识库"、"新建"、"搜索..." |
| KnowledgeFormPage | "新建知识" / "编辑知识" |
| AnalysisPage | "数据分析"、"提交总数"、"AC 率"、"提交时间线" |
| SettingsPage | "设置"、"账户"、"标签管理" |
| NotFoundPage | "页面不存在"、"返回首页" |
| Sidebar | "仪表盘"、"题目"、"模板库"、"知识库"、"数据分析"、"任务中心"、"设置" |

### 不需要改的

- 后端 LLM prompt（已经是中文）
- 模板页面（已经是中文）
- 任务/AI 相关 UI（已经是中文）
- API 端点路径（保持英文，RESTful 惯例）

---

## 八、实施计划

### Phase 1：AI 模块拆分（后端重构）

| # | 改动 | 文件 | 工作量 |
|---|------|------|--------|
| 1 | 提取 `parse.rs` — 通用 JSON 解析 | 新建 `ai/parse.rs` | 10min |
| 2 | 提取 `prompt.rs` — 所有 system prompt + prompt 构建 | 新建 `ai/prompt.rs` | 15min |
| 3 | 提取 `context.rs` — 上下文收集 + diff 生成 + 截断 | 新建 `ai/context.rs` | 15min |
| 4 | 实现 `agents/classifier.rs` | 新建 | 20min |
| 5 | 实现 `agents/template.rs`（含模板匹配） | 新建 | 25min |
| 6 | 实现 `agents/error.rs` | 新建 | 15min |
| 7 | 实现 `agents/knowledge.rs` | 新建 | 15min |
| 8 | 实现 `orchestrator.rs`（编排器） | 新建，替代 `problem_analyzer.rs` | 30min |
| 9 | 更新 `mod.rs` | 修改 | 5min |
| 10 | 删除 `problem_analyzer.rs` | 删除 | 2min |

### Phase 2：任务系统升级

| # | 改动 | 文件 | 工作量 |
|---|------|------|--------|
| 1 | 进度格式升级（ProgressStep → AgentProgress） | `task/model.rs` | 15min |
| 2 | 新增 `report_agent` 辅助函数 | `task/repo.rs` 或 `orchestrator.rs` | 10min |
| 3 | 前端：任务看板页面 `TasksPage.tsx` | 新建 | 40min |
| 4 | 前端：更新 `task-indicator.tsx` 适配新格式 | 修改 | 15min |
| 5 | 前端：更新 `task.ts` store | 修改 | 10min |
| 6 | 前端：侧边栏添加"任务中心" | `Sidebar.tsx` | 5min |
| 7 | 前端：路由添加 `/tasks` | `router.tsx` | 5min |

### Phase 3：模板 Skill 机制

| # | 改动 | 文件 | 工作量 |
|---|------|------|--------|
| 1 | Migration：`template` 表新增 `summary` 列 | 新建 migration | 10min |
| 2 | 更新 entity（自动生成） | `sea-orm-cli generate entity` | 5min |
| 3 | 更新 `template/model.rs`（DTO 补 summary） | 修改 | 5min |
| 4 | 更新 `template/repo.rs`（按 category 查询摘要） | 修改 | 10min |
| 5 | `agents/template.rs` 接入模板匹配 | 修改 | 已含在 Phase 1 |
| 6 | 前端：模板表单/详情增加 summary 字段 | 修改 | 10min |

### Phase 4：全中文化

| # | 改动 | 文件 | 工作量 |
|---|------|------|--------|
| 1 | LoginPage + RegisterPage | 2 个文件 | 10min |
| 2 | DashboardPage | 1 个文件 | 10min |
| 3 | ProblemsListPage + ProblemFormPage + ProblemDetailPage | 3 个文件 | 20min |
| 4 | KnowledgeListPage + KnowledgeFormPage + KnowledgeDetailPage | 3 个文件 | 15min |
| 5 | AnalysisPage + SettingsPage + NotFoundPage | 3 个文件 | 10min |
| 6 | Sidebar | 1 个文件 | 5min |
| 7 | TopBar | 1 个文件 | 5min |

---

## 九、后续可扩展方向

基于这套架构，未来可以轻松添加：

- **新 Agent** — 只需新建 `agents/xxx.rs`，在 orchestrator 中加入编排
- **Agent 自定义** — 用户可以给 Agent 改名、选择启用/禁用
- **多轮对话** — Agent 可以与 LLM 多轮交互（当前是单轮）
- **模板改进 Agent** — 发现现有模板有缺陷时，自动提出改进建议
- **学习计划 Agent** — 基于错误模式和知识点，生成个性化训练计划
