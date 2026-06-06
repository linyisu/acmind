# ACMind 技术栈迁移进度

> 一次性记录：把 ACMind 从 Tauri 桌面应用改造为 Web 全栈项目，对齐 JD（Rust 基础平台实习）与简历技术栈。

## 起点（旧架构，已删除）

- Tauri 桌面端（`src-tauri/`）
- GPUI 实验 crate（`crates/acmind-gpui/`）
- Firefox 浏览器扩展（`browser-extension-firefox/`）

## 目标架构

- 后端：Rust 1.77+, Axum 0.7, SeaORM 1.1, PostgreSQL 16, Tokio, JWT, DataFusion 41 + Arrow 53
- 前端：React 19 + Vite 6 + TypeScript 5 + Tailwind 4 + shadcn/ui, TanStack Query, Zustand
- Monorepo：pnpm workspace + turbo + Cargo workspace
- 部署：Docker Compose（postgres + api + web via nginx）

## 落地记录

按 phase 推进，已全部完成：

| Phase | 内容 | Commit |
|-------|------|--------|
| 0 | 仓库骨架（删除旧技术栈残留，pnpm + turbo + Cargo workspace 根，docker-compose 占位） | `71d06833` ~ `8be99d1c` |
| 1 | 后端骨架 + Auth（config / state / db / auth 模块，register/login/me） | `2540d393` |
| 2 | 业务实体模块（problem / submission / knowledge / tag CRUD） | `6a70730d` |
| 3 | DataFusion Analysis（3 个聚合端点 + RecordBatch + SQL） | `98d12dc3` |
| 4 | 前端迁移（Vite + React 19 + shadcn/ui + 7 个页面 + Zustand + TanStack Query） | `86c907e7` |
| 5 | 验收与文档（docker-compose 全栈 + CI + README + 简历同步） | `aef1ab6a` / `a9ceef39` / `c52c9ed0` |

## 决策与权衡

- **数据库迁移方式**：使用内联 `CREATE TABLE IF NOT EXISTS`（在 `db/mod.rs` 里），不引入 SeaORM Migration CLI。简单且足够 demo。
- **数据访问**：用 SeaORM 的 `ConnectionTrait` 跑原始 SQL（`query_one` / `query_all` / `execute`），只在需要类型安全的地方用 Entity。两者并存避免在复杂 JOIN 上硬塞 ORM。
- **DataFusion 用法**：从 PG 拉出 user 的 submissions，转 Arrow RecordBatch 灌进 `SessionContext`，跑 SQL `SELECT verdict, COUNT(*) ... GROUP BY verdict` 等。difficulty_distribution 用 SQL JOIN 内存表解决问题表，避免每条 SQL 触发数据库往返。
- **CORS**：dev 时 vite 跑 5173、api 跑 8080，跨域不可避免。`tower_http::cors::CorsLayer` 用 `allow_origin(Any)`，生产用 nginx 同源代理后这个层就不需要了。
- **shadcn/ui**：没有真用 CLI 拉组件（依赖问题 + 想要全控），而是手写 8 个 Radix UI 原子 + CVA 风格的组件，约定跟 shadcn 一致，迁移成本低。

## 验证

- `cargo check` ✓
- `cargo test` ✓（8 个测试全过：JWT round-trip、密码 hash/error 映射、DataFusion 聚合）
- `pnpm exec tsc -b` ✓
- 端到端 Playwright 走查：注册 → 登录 → 创建 problem → 创建 submission → 创建 knowledge → 创建 tag → 看到 Analysis 图表数据

## 仍可改进

- i18n zh/en 词条（spec 提到，未实装）
- tower_governor 限流（spec 提到，未实装）
- OpenAPI / Swagger UI
- 前端 E2E 测试（Playwright Test 套件，目前只用 Playwright MCP 走查）

## 简历同步

- `/home/mengh04/Workspace/CV/codecv_resume.md`：ACMind 描述补 Vite / DataFusion / Docker Compose，DataFusion SQL 写入"难点"与"成果"
- `/home/mengh04/Workspace/CV/resume_codecv.typ` 与 `resume.typ`：项目名改为 ACMind，副标题加入 DataFusion / React
- 两个 PDF 用 typst 重新编译（已 commit 到 CV 仓）
