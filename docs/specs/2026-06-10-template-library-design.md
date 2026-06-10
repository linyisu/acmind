# 智能模板库设计方案

> 日期：2026-06-10 | 状态：设计中

## 1. 背景与动机

### 当前状态
- 模板作为 `knowledge` 表的一种 `kind` 存储，字段为 `title` + `content`（Markdown 混排代码）
- AI 分析提取模板时，将代码和描述拼接为 Markdown 存入 `content`
- 没有语法高亮、语言分类、复杂度标注、使用追踪等结构化能力
- 前端只在 Knowledge 列表里用 kind=template 筛选，没有专属视图

### 目标
建立独立的**模板库模块**，支持：
1. **结构化模板存储** — 代码与元数据分离（语言、分类、复杂度）
2. **AI 自动提取** — 从提交分析中提取模板，自动填入结构化字段
3. **模板-题目关联** — 一个模板可用于多个题目，一个题目可关联多个模板
4. **使用推荐** — 基于算法类型/标签匹配推荐相关模板
5. **模板去重** — AI 提取时检测已有相似模板，避免重复

---

## 2. 数据库设计

### 2.1 新增 `template` 表

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGSERIAL` | PK | |
| `user_id` | `BIGINT` | FK → user, ON DELETE CASCADE | |
| `title` | `VARCHAR(255)` | NOT NULL | 模板名称，如 "Segment Tree"、"KMP" |
| `category` | `VARCHAR(64)` | NOT NULL, DEFAULT 'other' | 算法分类 |
| `language` | `VARCHAR(32)` | NOT NULL, DEFAULT 'cpp' | 编程语言 |
| `code` | `TEXT` | NOT NULL | 模板代码 |
| `description` | `TEXT` | NOT NULL DEFAULT '' | 使用说明（Markdown） |
| `time_complexity` | `VARCHAR(64)` | NULL | 时间复杂度，如 "O(n log n)" |
| `space_complexity` | `VARCHAR(64)` | NULL | 空间复杂度 |
| `source` | `VARCHAR(32)` | NOT NULL DEFAULT 'manual' | 来源：manual / ai_extracted |
| `source_problem_id` | `BIGINT` | FK → problem, ON DELETE SET NULL, NULL | 提取自哪个题目 |
| `difficulty` | `INTEGER` | NULL, CHECK 1-5 | 模板难度 |
| `usage_count` | `INTEGER` | NOT NULL DEFAULT 0 | 关联的题目数（冗余计数） |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, DEFAULT NOW() | |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, DEFAULT NOW() | |

索引：
- `idx_template_user` on `(user_id)`
- `idx_template_user_category` on `(user_id, category)`
- `idx_template_user_language` on `(user_id, language)`

### 2.2 新增 `template_tag` 关联表

| 列名 | 类型 | 约束 |
|------|------|------|
| `template_id` | `BIGINT` | FK → template, ON DELETE CASCADE |
| `tag_id` | `BIGINT` | FK → tag, ON DELETE CASCADE |

- 复合主键 `(template_id, tag_id)`
- 复用现有 `tag` 表

### 2.3 新增 `template_problem` 关联表

| 列名 | 类型 | 约束 |
|------|------|------|
| `template_id` | `BIGINT` | FK → template, ON DELETE CASCADE |
| `problem_id` | `BIGINT` | FK → problem, ON DELETE CASCADE |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, DEFAULT NOW() |

- 复合主键 `(template_id, problem_id)`
- 记录「哪个模板用于哪个题目」

### 2.4 枚举值定义

**category（算法分类）：**

| 值 | 中文标签 |
|----|---------|
| `data_structure` | 数据结构 |
| `graph` | 图论 |
| `dp` | 动态规划 |
| `string` | 字符串 |
| `math` | 数学 |
| `geometry` | 计算几何 |
| `greedy` | 贪心 |
| `search` | 搜索 |
| `sort` | 排序 |
| `binary_search` | 二分 |
| `other` | 其他 |

**language：** `cpp`, `python`, `java`, `rust`, `go`, `other`

---

## 3. 后端 API 设计

### 3.1 模块结构

```
apps/api/src/template/
├── mod.rs          # 模块声明
├── model.rs        # 请求/响应 DTO + TemplateCategory enum
├── repo.rs         # 数据库查询
├── route.rs        # Axum handlers
└── service.rs      # 业务逻辑
```

### 3.2 API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/templates` | 列表（支持筛选） |
| POST | `/api/v1/templates` | 创建模板 |
| GET | `/api/v1/templates/{id}` | 获取单个模板（含标签、关联题目） |
| PATCH | `/api/v1/templates/{id}` | 更新模板 |
| DELETE | `/api/v1/templates/{id}` | 删除模板 |
| POST | `/api/v1/templates/{id}/problems` | 关联题目 |
| DELETE | `/api/v1/templates/{id}/problems/{problem_id}` | 取消关联 |
| GET | `/api/v1/templates/stats` | 统计（各分类数量、总数） |

### 3.3 请求/响应 DTO

```rust
// ── model.rs ──

#[derive(Debug, Deserialize)]
pub struct CreateTemplateReq {
    pub title: String,
    pub category: TemplateCategory,   // enum, ser/de as lowercase string
    pub language: String,             // "cpp", "python", etc.
    pub code: String,
    pub description: String,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub difficulty: Option<i32>,      // 1-5
    pub tag_ids: Vec<i64>,
    pub problem_ids: Vec<i64>,        // 关联的题目
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateReq {
    pub title: Option<String>,
    pub category: Option<TemplateCategory>,
    pub language: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub difficulty: Option<i32>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub category: Option<TemplateCategory>,
    pub language: Option<String>,
    pub tag_id: Option<i64>,
    pub problem_id: Option<i64>,
    pub search: Option<String>,       // 搜索 title + description + code
    pub sort: Option<String>,         // "usage", "created", "title"
}

#[derive(Debug, Serialize)]
pub struct TemplateResp {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub category: String,
    pub language: String,
    pub code: String,
    pub description: String,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub source: String,
    pub source_problem_id: Option<i64>,
    pub difficulty: Option<i32>,
    pub usage_count: i32,
    pub tag_ids: Vec<i64>,
    pub problem_ids: Vec<i64>,        // 关联的题目 ID 列表
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TemplateStats {
    pub total: i64,
    pub by_category: Vec<CategoryCount>,
    pub by_language: Vec<LanguageCount>,
}

#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}
```

---

## 4. AI 集成改动

### 4.1 改动 `problem_analyzer.rs`

当前 Stage 3（模板提取）返回的 `ExtractedTemplate` 只有 `title, code, description`。
Stage 4 保存时拼接为 Markdown 存入 knowledge 表。

**改动点：**

1. **扩展 LLM 提取 prompt** — 要求 LLM 同时返回 `category`、`language`、`time_complexity`：

   ```rust
   #[derive(Deserialize)]
   struct ExtractedTemplate {
       title: String,
       code: String,
       description: String,
       category: String,          // 新增
       time_complexity: String,   // 新增
   }
   ```

2. **改保存逻辑** — 模板存入 `template` 表而非 `knowledge` 表（kind=template）

   ```rust
   // 原：knowledge_repo::insert(..., "template", ...)
   // 新：template_repo::insert_structured(...)
   ```

3. **去重逻辑** — 按 `(user_id, category, language, title)` 检查是否已存在相似模板
   - 已存在 → 跳过，或追加关联题目

4. **自动关联** — 提取出的模板自动关联到当前分析的 problem（通过 `template_problem` 表）

### 4.2 响应扩展

`ProblemAnalysisResp` 中的 `extracted_templates` 保持不变（计数），但新增返回模板 ID 列表供前端跳转。

---

## 5. 前端设计

### 5.1 路由与导航

| 路由 | 页面 | 说明 |
|------|------|------|
| `/templates` | `TemplatesListPage` | 模板库主页面 |
| `/templates/new` | `TemplateFormPage` | 创建模板 |
| `/templates/:id` | `TemplateDetailPage` | 模板详情 |
| `/templates/:id/edit` | `TemplateFormPage` | 编辑模板 |

Sidebar 新增 `Templates` 导航项，图标用 `Code2`。

### 5.2 页面设计

#### TemplatesListPage（模板列表）

```
┌─────────────────────────────────────────────────────────┐
│ 模板库                                    [+ 新建模板]   │
├─────────────────────────────────────────────────────────┤
│ [🔍 搜索模板...]  [分类 ▾]  [语言 ▾]  [标签 ▾]  [排序 ▾] │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│ │ 数据结构 (5) │ │ 图论 (3)    │ │ 动态规划 (4) │  ← 分类 │
│ └─────────────┘ └─────────────┘ └─────────────┘        │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Segment Tree        cpp  O(n log n)     ★★★  使用5次│ │
│ │ 数据结构 · 最后更新 2026-06-08                       │ │
│ │ 线段树模板，支持区间查询和单点修改...                  │ │
│ └─────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ KMP 算法             cpp  O(n+m)        ★★   使用3次│ │
│ │ 字符串 · 最后更新 2026-06-07                         │ │
│ │ 字符串匹配模板...                                    │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

- 分类快捷筛选标签（显示每类数量）
- 卡片列表：标题 + 语言 badge + 复杂度 + 难度星 + 使用次数
- 支持网格/列表视图切换

#### TemplateDetailPage（模板详情）

```
┌─────────────────────────────────────────────────────────┐
│ ← 返回列表            [编辑]  [删除]  [关联题目]        │
├─────────────────────────────────────────────────────────┤
│ Segment Tree                              cpp  ★★★     │
│ 数据结构 · O(n log n) · O(n) · 来源: AI 提取           │
│ 标签: [线段树] [区间查询] [数据结构]                     │
├─────────────────────────────────────────────────────────┤
│ 📝 使用说明                                             │
│ 支持区间求和查询和单点修改。建树 O(n)，查询/修改 O(log n)│
│ 适用于需要频繁区间操作的场景...                           │
├─────────────────────────────────────────────────────────┤
│ 💻 代码                                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ struct SegTree {                    [复制代码]       │ │
│ │     n: usize,                                       │ │
│ │     tree: Vec<i64>,                                 │ │
│ │ }                                                   │ │
│ │ ...                                                 │ │
│ └─────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│ 🔗 关联题目 (3)                                         │
│ • POJ 3264 - Balanced Lineup                            │
│ • CF 1023D - Array Restoration                          │
│ • HDU 1754 - I Hate It                                  │
└─────────────────────────────────────────────────────────┘
```

- 代码区支持语法高亮（使用 Prism.js 或 highlight.js）
- 关联题目可跳转到题目详情

#### TemplateFormPage（创建/编辑表单）

表单字段：
- 标题（必填）
- 分类（下拉选择）
- 语言（下拉选择）
- 代码（Monaco Editor 或 textarea）
- 使用说明（Markdown textarea）
- 时间复杂度（文本输入）
- 空间复杂度（文本输入）
- 难度（1-5 选择器）
- 标签（多选）
- 关联题目（搜索选择，仅创建时）

---

## 6. 共享类型扩展

`packages/shared/src/types/template.ts` 新增：

```typescript
export type TemplateCategory =
  | "data_structure" | "graph" | "dp" | "string"
  | "math" | "geometry" | "greedy" | "search"
  | "sort" | "binary_search" | "other";

export interface Template {
  id: number;
  user_id: number;
  title: string;
  category: TemplateCategory;
  language: string;
  code: string;
  description: string;
  time_complexity: string | null;
  space_complexity: string | null;
  source: string;
  source_problem_id: number | null;
  difficulty: number | null;
  usage_count: number;
  tag_ids: number[];
  problem_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface CreateTemplateRequest {
  title: string;
  category: TemplateCategory;
  language: string;
  code: string;
  description: string;
  time_complexity?: string;
  space_complexity?: string;
  difficulty?: number;
  tag_ids: number[];
  problem_ids: number[];
}

export interface UpdateTemplateRequest {
  title?: string;
  category?: TemplateCategory;
  language?: string;
  code?: string;
  description?: string;
  time_complexity?: string;
  space_complexity?: string;
  difficulty?: number;
  tag_ids?: number[];
}

export interface TemplateStats {
  total: number;
  by_category: { category: TemplateCategory; count: number }[];
  by_language: { language: string; count: number }[];
}
```

---

## 7. 与现有 Knowledge 模块的关系

| 维度 | Knowledge（保持不变） | Template（新建） |
|------|---------------------|-----------------|
| 定位 | 通用知识条目（笔记、技巧、片段） | 结构化代码模板 |
| 内容 | Markdown 文本 | 代码 + 描述分离 |
| 元数据 | kind + title + content | category + language + complexity + difficulty |
| 关联 | problem_id（单个） | template_problem（多对多） |
| AI 来源 | 错误分析、知识点提取 | 从 AC 代码提取 |

- `knowledge` 表中已有的 `kind=template` 条目**暂不迁移**，后续可写脚本导出
- AI 分析流程改用 `template` 表保存新提取的模板
- Knowledge 模块的 `KnowledgeKind` 枚举移除 `Template` 变体（或保留用于兼容，前端隐藏）

---

## 8. 实现计划（分步）

### Phase 1: 数据层（~1.5h）
1. 新建 migration：`template` 表 + `template_tag` + `template_problem`
2. `docker compose down -v` 重建 DB
3. `sea-orm-cli generate entity` 生成新 entity
4. 新建 `apps/api/src/template/` 模块：model + repo + service + route
5. 注册路由到 `lib.rs`

### Phase 2: 前端基础（~1.5h）
1. `packages/shared/src/types/template.ts` 共享类型
2. `apps/web/src/lib/api/index.ts` 新增 `templatesApi`
3. `TemplatesListPage.tsx` — 列表 + 筛选
4. `TemplateDetailPage.tsx` — 详情（含代码高亮）
5. `TemplateFormPage.tsx` — 创建/编辑表单
6. 更新 `router.tsx` 和 `Sidebar.tsx`

### Phase 3: AI 集成（~1h）
1. 扩展 `ExtractedTemplate` 结构体和 LLM prompt
2. 改 `problem_analyzer.rs` 保存逻辑：存入 template 表
3. 自动关联题目 + 去重逻辑

### Phase 4: 增强功能（~0.5h）
1. `GET /templates/stats` 统计接口
2. 前端 Sidebar badge 显示模板总数
3. ProblemDetailPage 中显示关联的模板列表

---

## 9. 风险与决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 新表 vs 扩展 knowledge | 新表 | 模板有独特的结构化需求（代码、语言、复杂度），混入 knowledge 会使该表过度膨胀 |
| 代码高亮方案 | highlight.js CDN | 轻量，满足需求，无需引入 Monaco |
| 去重策略 | (user, category, language, title) 唯一 | AI 提取同一题目的相同模板不会重复 |
| 已有 knowledge template | 不迁移 | 保持渐进式，不破坏现有数据 |
