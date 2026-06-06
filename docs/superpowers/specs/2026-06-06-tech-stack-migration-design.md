# ACMind 技术栈迁移设计文档

**日期**：2026-06-06
**分支**：`feat/web-fullstack-migration`（待用户指定后确认）
**目标**：把 ACMind 从 Tauri 桌面应用改造为 Web 全栈项目，对齐 JD（Rust 基础平台实习）与简历描述的技术栈

---

## 1. 背景与目标

### 1.1 当前问题

- JD（`/home/mengh04/Workspace/CV/jd_rust_platform_intern.md`）和简历（`/home/mengh04/Workspace/CV/codecv_resume.md`）都把 ACMind 描述为：
  - 后端：Rust + **Axum** + **Tokio** + **SeaORM** + **PostgreSQL** + RESTful API
  - 前端：React + TailwindCSS + shadcn/ui
- 实际项目（`/home/mengh04/Workspace/acmind`）是 **Tauri 2.11 桌面应用**：
  - 后端用 **SQLx**（不是 SeaORM）、**SQLite**（不是 PostgreSQL）
  - 没有独立 HTTP 后端（通过 Tauri command 通信）
  - ORM 和数据库与简历描述完全不一致

### 1.2 改造目标

把项目实际改造成简历和 JD 描述的样子：
1. 丢掉 Tauri 桌面壳
2. 后端改为 **Rust + Axum + SeaORM + PostgreSQL** 独立 HTTP 服务
3. 前端改为 **React + Vite + shadcn/ui**，通过 HTTP API 调用后端
4. 新增 **DataFusion Analysis** 模块（JD 加分项：Apache Arrow / DataFusion）
5. 引入 JWT 鉴权、限流、结构化日志
6. Docker Compose 一键启动

### 1.3 非目标

- 不重做业务逻辑（题目/提交/知识资产的核心功能保持一致）
- 不引入微服务架构
- 不引入 Kubernetes / Helm（仅 Docker Compose）
- 不做生产级 HA / 多租户
- 不做端到端 E2E（Playwright）—— 仅集成测试 + 单元测试

---

## 2. 技术栈总览

| 层 | 选型 | 说明 |
|---|---|---|
| 后端语言 | Rust 1.77+ | workspace 风格 |
| Web 框架 | **Axum 0.7+** | JD 明确要求 |
| 异步运行时 | **Tokio 1** | JD 明确要求（full features） |
| ORM | **SeaORM 1.x** | JD 明确要求；替代 SQLx |
| 数据库 | **PostgreSQL 16** | 替代 SQLite，简历明确 |
| 迁移 | SeaORM Migration | 内置支持 |
| 鉴权 | **jsonwebtoken** + tower 中间件 | JWT + Bearer |
| 限流 | **tower_governor** | 简历提到的"请求限流" |
| 日志 | **tracing + tracing-subscriber** | JSON 格式，结构化 |
| 测试 | cargo test + reqwest 测试客户端 | 后端 |
| 数据分析 | **Apache Arrow + DataFusion 41+** | JD 加分项 |
| 序列化 | serde + serde_json | — |
| 错误处理 | thiserror + anyhow | — |
| 配置 | dotenvy + envy | .env 文件 |

| 层 | 选型 | 说明 |
|---|---|---|
| 前端框架 | **React 19** | 保留 |
| 语言 | **TypeScript 6** | 保留 |
| 构建工具 | **Vite 8** | 保留 |
| 样式 | **Tailwind CSS 4** | 保留 |
| 组件库 | **shadcn/ui**（CLI 方式） | 替代当前的 Radix UI 散装调用 |
| 路由 | **React Router 7** | 保留 |
| 数据请求 | **TanStack Query 5** | 保留 |
| 状态 | **Zustand 5** | 保留 |
| 表单 | **react-hook-form + zod** | 新增（shadcn 标配） |
| HTTP | 原生 fetch + 自写客户端 | 不引 axios |
| 测试 | vitest + Testing Library | 保留 |
| i18n | i18next + react-i18next | 保留 |
| 图表 | recharts | 保留（Analysis 模块用得到） |

| 层 | 选型 | 说明 |
|---|---|---|
| 共享 | TypeScript 类型 | packages/shared |
| 单仓 | **pnpm workspace + turbo** | 单仓多包 |
| 部署 | **Docker Compose** | postgres + api + web |

---

## 3. 物理布局

```
acmind/
├── apps/
│   ├── api/                         # Rust Axum 后端
│   │   ├── src/
│   │   │   ├── main.rs              # 启动入口、路由组装
│   │   │   ├── config.rs            # 配置加载（dotenvy）
│   │   │   ├── error.rs             # AppError + IntoResponse
│   │   │   ├── state.rs             # AppState（DB、JWT key）
│   │   │   ├── db/
│   │   │   │   ├── mod.rs           # 连接池、初始化
│   │   │   │   └── seed.rs          # 初始数据（dev 环境）
│   │   │   ├── auth/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── jwt.rs           # 签发/校验
│   │   │   │   ├── password.rs      # bcrypt 包装
│   │   │   │   ├── middleware.rs    # AuthLayer
│   │   │   │   ├── service.rs
│   │   │   │   ├── repo.rs
│   │   │   │   └── route.rs         # POST /auth/login, /auth/register
│   │   │   ├── problem/             # 题目
│   │   │   │   ├── mod.rs
│   │   │   │   ├── model.rs         # DTO
│   │   │   │   ├── service.rs
│   │   │   │   ├── repo.rs
│   │   │   │   └── route.rs
│   │   │   ├── submission/          # 提交记录
│   │   │   ├── knowledge/           # 知识资产
│   │   │   ├── analysis/            # DataFusion 统计（新增）
│   │   │   │   ├── mod.rs
│   │   │   │   ├── service.rs       # 编排 DataFusion 查询
│   │   │   │   ├── datafusion_ctx.rs
│   │   │   │   └── route.rs
│   │   │   └── health.rs            # GET /health
│   │   ├── migration/               # SeaORM 迁移文件
│   │   ├── tests/                   # 集成测试
│   │   ├── Cargo.toml
│   │   ├── .env.example
│   │   └── Dockerfile
│   └── web/                         # React 前端
│       ├── src/
│       │   ├── main.tsx
│       │   ├── App.tsx
│       │   ├── components/
│       │   │   └── ui/              # shadcn/ui 生成
│       │   ├── pages/
│       │   ├── hooks/
│       │   ├── lib/
│       │   │   ├── api/             # HTTP 客户端 + TanStack Query
│       │   │   └── stores/          # Zustand
│       │   └── i18n/
│       ├── components.json          # shadcn 配置
│       ├── tailwind.config.ts
│       ├── vite.config.ts
│       ├── tsconfig.json
│       └── package.json
├── packages/
│   └── shared/                      # 共享 TS 类型
│       ├── src/
│       │   ├── types/
│       │   │   ├── problem.ts
│       │   │   ├── submission.ts
│       │   │   ├── knowledge.ts
│       │   │   ├── analysis.ts
│       │   │   └── auth.ts
│       │   └── index.ts
│       ├── tsconfig.json
│       └── package.json
├── docker-compose.yml
├── pnpm-workspace.yaml
├── turbo.json
├── Cargo.toml                       # workspace 根（成员 = apps/api）
├── package.json                     # 根 scripts（turbo run dev/build/test）
├── .gitignore
└── README.md
```

### 3.1 删除项（迁移完成后）

- `src-tauri/`（整个目录）
- `crates/acmind-gpui/`（实验性）
- `browser-extension-firefox/`（依赖 Tauri 导入服务的特性；后端改造为普通 HTTP POST /import/external 即可上传，不再需要本地端口接收）
- `crates/acmind-core/` 暂合并入 `apps/api/src/`（业务边界不复杂时不分 crate；如果用户后续想要多 crate 边界再拆）

注：浏览器扩展的本地端口接收逻辑由 `apps/api/src/analysis`（或独立的 `import` 子模块）接管，改造为标准 HTTP 端点。前端如有"导入"按钮，调用 `POST /import/external` 即可。

---

## 4. 后端模块设计

### 4.1 路由总览

| 方法 | 路径 | 鉴权 | 说明 |
|---|---|---|---|
| GET | /health | 否 | 健康检查 |
| POST | /auth/register | 否 | 注册（dev 模式可关） |
| POST | /auth/login | 否 | 登录，返回 JWT |
| GET | /auth/me | 是 | 当前用户信息 |
| GET | /problems | 是 | 列表（支持 ?tag=&difficulty=） |
| POST | /problems | 是 | 创建 |
| GET | /problems/:id | 是 | 详情 |
| PATCH | /problems/:id | 是 | 更新 |
| DELETE | /problems/:id | 是 | 删除 |
| GET | /submissions | 是 | 列表（?problem_id=） |
| POST | /submissions | 是 | 创建 |
| GET | /submissions/:id | 是 | 详情 |
| GET | /knowledge | 是 | 列表 |
| POST | /knowledge | 是 | 创建 |
| GET | /knowledge/:id | 是 | 详情 |
| PATCH | /knowledge/:id | 是 | 更新 |
| DELETE | /knowledge/:id | 是 | 删除 |
| GET | /analysis/submissions/summary | 是 | 提交总数/按结果分布 |
| GET | /analysis/submissions/timeline | 是 | 每日提交量时间序列 |
| GET | /analysis/problems/difficulty-distribution | 是 | 难度分布 |
| POST | /import/external | 是 | 外部导入（替代 vjudge 路径） |

### 4.2 实体（Entity）设计

基于现有 SQLite schema 改造，字段基本不变，但调整为主键/外键类型适配 PG：

**User**
- id: i64 (PG BIGSERIAL)
- username: String UNIQUE NOT NULL
- email: String UNIQUE NOT NULL
- password_hash: String NOT NULL
- created_at, updated_at

**Problem**
- id: i64 BIGSERIAL
- source: String（如 Codeforces、AtCoder、VJudge、Custom）
- external_id: String NULL
- title: String NOT NULL
- url: String NULL
- difficulty: i32 NULL（1-5）
- tags: Vec<String>（独立表 + 多对多）
- statement: Text NULL
- created_at, updated_at

**Submission**
- id: i64 BIGSERIAL
- problem_id: i64 FK
- language: String
- code: Text NOT NULL
- verdict: String（AC / WA / TLE / RE / CE / Pending）
- runtime_ms: i32 NULL
- memory_kb: i32 NULL
- notes: Text NULL
- submitted_at: Timestamp

**KnowledgeAsset**
- id: i64 BIGSERIAL
- problem_id: i64 NULL FK
- kind: String（template / technique / note / snippet）
- title: String
- content: Text
- tags: Vec<String>
- created_at, updated_at

**Tag**
- id: i64, name: String UNIQUE
- 多对多关联 problem、knowledge

**AIModel** / **AIPrompt**（保留现有 AI 相关表）
- 复用当前 src-tauri/src/ai/ 的 schema

### 4.3 鉴权设计

- 密码哈希：`bcrypt`（`bcrypt` crate）
- JWT：`jsonwebtoken` crate
  - Claims: `{ sub: user_id, username, exp, iat }`
  - 默认有效期 24h，可通过 env 配置
- 中间件：自定义 `axum::middleware::from_fn_with_state`
  - 从 `Authorization: Bearer <token>` 解析
  - 把 `UserContext` 注入到 `Extension`
- 注册：默认开启，但生产环境可通过 `ALLOW_REGISTER=false` 关闭

### 4.4 限流设计

- `tower_governor`（基于 governor 库）
- 限流 key：登录用户 ID（已登录）/ IP（未登录）
- 默认配置：100 req/s 突发 200

### 4.5 日志设计

- `tracing` + `tracing-subscriber` JSON 输出
- 字段：timestamp, level, target, span fields, message
- 集成 `tower-http::trace::TraceLayer` 自动记录 HTTP 请求

### 4.6 Analysis 模块（DataFusion）

**职责**：批量分析用户的训练数据，生成统计报表。

**数据流**：
1. service.rs 调用 `repo::find_all_in_range(...)` 拉取 submission 数据
2. 转成 Arrow `RecordBatch`（`datafusion::arrow::array::RecordBatch`）
3. 在 DataFusion SessionContext 中注册为内存表 `submissions`
4. 执行 SQL 查询，输出 Arrow result
5. 序列化为 JSON 返回前端

**示例接口**：

```
GET /analysis/submissions/summary
→ {
    total: 1234,
    by_verdict: { AC: 500, WA: 400, TLE: 200, RE: 100, CE: 34 },
    ac_rate: 0.405
  }
```

```
GET /analysis/submissions/timeline?from=2025-01-01&to=2025-12-31
→ [
    { date: "2025-01-01", count: 10, ac_count: 4 },
    ...
  ]
```

**复用**：
- `datafusion` 41+ 内置 SQL 引擎
- 提交数据量预期在 10k-100k 行，DataFusion 内存表完全能 handle
- 后续可扩展为：题目难度 vs AC 率热力图、提交代码相似度分析

---

## 5. 前端模块设计

### 5.1 路由结构

| 路径 | 页面 | 鉴权 |
|---|---|---|
| /login | LoginPage | 否 |
| /register | RegisterPage | 否（如果 ALLOW_REGISTER） |
| / | HomePage (Dashboard) | 是 |
| /problems | ProblemsListPage | 是 |
| /problems/:id | ProblemDetailPage | 是 |
| /submissions | SubmissionsListPage | 是 |
| /knowledge | KnowledgeListPage | 是 |
| /analysis | AnalysisPage | 是 |
| /settings | SettingsPage | 是 |

### 5.2 shadcn/ui 接入

- 使用 `pnpm dlx shadcn@latest init` 初始化
- 配置 `components.json`
- 需要的组件：button、input、label、card、dialog、dropdown-menu、select、tabs、tooltip、toast、table、form、badge、separator、skeleton、sheet、popover、command

### 5.3 状态与数据流

- 全局 token 存放在 `localStorage`（key: `acmind_token`）
- HTTP 客户端拦截器：自动添加 `Authorization: Bearer <token>`，401 时跳转登录
- TanStack Query：所有服务端数据
- Zustand：当前用户、UI 状态（侧边栏折叠等）

### 5.4 API 客户端

- 基础 URL 通过 Vite env 注入（`VITE_API_BASE_URL`）
- 默认 `http://localhost:8080`
- 类型通过 `packages/shared` 共享

---

## 6. 数据流图

```
┌──────────┐   HTTP/JSON   ┌─────────────┐   SQL   ┌──────────────┐
│  React   │ ────────────> │   Axum API  │ ──────> │ PostgreSQL   │
│ (shadcn) │ <──────────── │  (Tokio)    │ <────── │   (SeaORM)   │
└──────────┘               └─────────────┘         └──────────────┘
                                  │
                                  │ DataFusion SQL
                                  ▼
                            ┌─────────────┐
                            │   Arrow     │
                            │  Memory     │
                            │   Tables    │
                            └─────────────┘
```

---

## 7. 错误处理

- 后端 `AppError` 实现 `IntoResponse`
- 错误变体：`NotFound`, `Unauthorized`, `Forbidden`, `BadRequest`, `Conflict`, `Internal`, `Database`, `Validation`
- HTTP 状态码：400 / 401 / 403 / 404 / 409 / 500
- 响应体：`{ error: { code, message } }`
- 前端在拦截器里统一 toast 展示

---

## 8. 测试策略

### 8.1 后端

- **单元测试**：service 层、纯函数（密码哈希、DataFusion SQL 生成）
- **集成测试**：使用 `testcontainers` 启动一次性 PostgreSQL 实例
  - 每个实体一个集成测试文件
  - 覆盖 happy path + 主要 error path
  - 测试在 CI 中跑（无需本地 PG）
- **覆盖率目标**：service + route 层 ≥ 70%

### 8.2 前端

- **组件测试**：关键交互组件（ProblemForm、SubmissionRow）
- **Hook 测试**：useAuth、useApi
- **不写 E2E**（如前述非目标）

### 8.3 CI

- GitHub Actions（项目托管在 GitHub 时）
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`（使用 testcontainers 启动 PG）
  - `pnpm install && pnpm turbo run lint test build`
- CI 配置文件：`.github/workflows/ci.yml`

---

## 9. 部署 / 启动

### 9.1 本地开发

```bash
# 启动 PG + API + web
docker compose up -d

# 仅后端
cd apps/api && cargo run

# 仅前端
cd apps/web && pnpm dev
```

### 9.2 docker-compose.yml 服务

| 服务 | 端口 | 镜像 | 备注 |
|---|---|---|---|
| postgres | 5432 | postgres:16-alpine | 数据卷：postgres_data |
| api | 8080 | Dockerfile build | 等 pg healthy 后启动 |
| web | 5173 | Dockerfile build (nginx) | 反向代理 /api → api:8080 |

### 9.3 .env.example

```
DATABASE_URL=postgres://acmind:acmind@localhost:5432/acmind
JWT_SECRET=change-me-in-production
JWT_EXPIRES_IN=86400
ALLOW_REGISTER=true
RUST_LOG=info,acmind_api=debug
WEB_PORT=5173
API_PORT=8080
```

---

## 10. 迁移步骤概览（实现期 plan 详细展开）

> **范围说明**：本迁移工作量较大（涉及后端重写、前端大幅调整、部署基础设施重建、测试补齐）。实现阶段会拆分为多个顺序执行的子 plan，每个子 plan 可独立提交和评审。
>
> 子 plan 切分（建议顺序）：
>
> 1. **子 plan A — 仓库骨架与基础设施**：建分支、删旧代码、建立 monorepo 结构（pnpm workspace + turbo + Cargo workspace）、docker-compose 骨架、.env.example
> 2. **子 plan B — 后端骨架与 auth**：Axum 启动、配置加载、SeaORM 连接、迁移、错误类型、JWT 鉴权、auth 模块（含测试）
> 3. **子 plan C — 业务实体模块**：problem / submission / knowledge 三个模块的 service + repo + route + 测试
> 4. **子 plan D — DataFusion Analysis 模块**：analysis service + DataFusion 集成 + 至少 3 个分析接口 + 测试
> 5. **子 plan E — 前端迁移**：Vite + shadcn/ui 接入、HTTP 客户端、调整所有页面调用方式、登录注册页面
> 6. **子 plan F — 端到端验收与文档**：docker compose up 跑通、CI 配通、README 完善、简历同步

### 10.1 关键路径节点

- 子 plan A 完成后：仓库结构成型
- 子 plan B 完成后：`docker compose up` 可启动 PG + API，能 register/login
- 子 plan C 完成后：核心 CRUD 全通
- 子 plan D 完成后：Analysis 报表可用（亮点功能落地）
- 子 plan E 完成后：UI 完整可交互
- 子 plan F 完成后：交付状态

---

## 11. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 浏览器扩展 / 桌面端用户数据丢失 | 不保留旧 Tauri 数据；提供 README 说明 |
| SeaORM 学习曲线 | 文档齐全，参考 SeaORM 官方示例 |
| 现有 React 组件大量调整 | shadcn CLI 一次生成基础组件，逐页替换 |
| Docker 构建慢 | api 用 cargo-chef 预热；web 用多阶段构建 |
| DataFusion 体积大 | 仅 api 依赖；按需 feature 关闭 |

---

## 12. 验收标准

- [ ] `docker compose up` 一次启动全部服务
- [ ] `pnpm install && pnpm turbo run dev` 可启动开发
- [ ] 注册 → 登录 → 创建题目 → 创建提交 → 查看分析 全流程可走通
- [ ] 后端 `cargo test` 全部通过
- [ ] 后端 `cargo clippy -- -D warnings` 无警告
- [ ] 前端 `pnpm turbo run lint test build` 全部通过
- [ ] README 描述完整
- [ ] 简历中"acmind-rs"项目描述与技术栈对得上
- [ ] DataFusion Analysis 至少 3 个接口可用
- [ ] JWT 鉴权中间件覆盖所有受保护路由
