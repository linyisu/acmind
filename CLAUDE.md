<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **acmind** (1203 symbols, 2565 relationships, 101 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/acmind/context` | Codebase overview, check index freshness |
| `gitnexus://repo/acmind/clusters` | All functional areas |
| `gitnexus://repo/acmind/processes` | All execution flows |
| `gitnexus://repo/acmind/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->

## 项目架构

后端模块遵循 **model → repo → service → route** 四层结构：
- `model.rs` — 请求/响应 DTO
- `repo.rs` — SeaORM 数据库查询
- `service.rs` — 业务逻辑
- `route.rs` — Axum handler

AI 模块结构：
- `ai/agents/` — 各 Agent 独立文件（classifier/template/error/knowledge）
- `ai/orchestrator.rs` — 编排器，协调各 Agent + 管理任务进度
- `ai/provider.rs` — LLM 调用抽象（含重试 + timeout）
- `ai/parse.rs` — LLM 响应 JSON 解析（含 `<think>` 标签剥离）
- `ai/prompt.rs` — System prompt 定义
- `ai/context.rs` — 上下文收集 + diff 生成

前端：
- 页面在 `apps/web/src/pages/`，路由在 `router.tsx`
- 全中文 UI，所有文案用中文
- LaTeX 渲染用 `react-markdown` + `remark-math` + `rehype-katex`

## SeaORM Schema 变更工作流

acmind 用 SeaORM 管理 Postgres schema。**所有 schema 变更必须走 migration 文件，绝不直接改老 migration**。

### 改 schema 的标准流程

1. **改 migration 文件**（`apps/api/migration/src/m20260101_NNNNNN_xxx.rs`）
   - 新增列 / 改类型 / 加索引 → **手写 `up()` 里的 DDL**
   - 同步写 `down()`（哪怕只是 drop table，保留回滚能力）
2. **本地重建 DB 验证**：
   ```bash
   docker compose down -v        # 删 volume 重建
   docker compose up -d --build  # 启动时自动跑 Migrator::up
   ```
3. **同步 entity 代码**：
   ```bash
   pnpm exec sea-orm-cli generate entity -u postgres://acmind:acmind@localhost:5432/acmind -o apps/api/src/entity
   ```
   - 这一步**会重写 entity 目录里的所有 model 文件**（自动生成，不要手改）
   - 如果新增了外键关系，**手工检查 `Relation` enum 和 `impl Related<...>` 是否需要补**

### 铁律

- **已存在的 migration 文件**（已跑过 DB 的）**不要改 up/down 函数**——只能新建一个 migration 去"修正"（生产场景）
- 当前项目还在开发阶段，**已经写过的 migration 文件可以直接改**，配合 `docker compose down -v` 重建 DB
- `entity/` 目录是自动生成代码，**不要手改**
- 新加表前先想清楚：哪些列 NOT NULL / UNIQUE / 需要 INDEX / 用 `timestamptz` 还是 `timestamp` / 软删还是硬删
