# ACMind

个人算法训练知识库 — 记录题目、提交、模板、错误模式，用 AI 自动分析和整理。

## 功能

- **题目管理** — 记录来自各 OJ 的题目，支持标签分类和难度标注
- **提交记录** — 手动记录或通过 VJudge 导入提交历史
- **模板库** — 结构化代码模板，支持分类/语言/复杂度/关联题目，AI 自动提取和匹配
- **知识库** — 笔记、技巧、代码片段，Markdown + LaTeX 公式
- **AI 全量分析** — 对一道题的全部提交进行深度分析，自动识别算法类型、提取模板、诊断错误、梳理知识点
- **任务中心** — 实时查看 AI 分析进度，支持取消
- **数据分析** — 提交统计、判题结果分布、难度分布图表

## 快速开始

```bash
cp .env.example .env   # 编辑 JWT_SECRET 和 LLM_API_KEY
docker compose up -d --build
```

- 前端：http://localhost:5173
- API：http://localhost:8080

### 本地开发

```bash
docker compose up -d postgres

# 后端
cd apps/api
DATABASE_URL=postgres://acmind:acmind@localhost:5432/acmind \
  JWT_SECRET=devsecret LLM_PROVIDER=noop cargo run

# 前端
cd apps/web && pnpm install && pnpm dev
```

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DATABASE_URL` | Postgres 连接串 | — |
| `JWT_SECRET` | JWT 签名密钥 | — |
| `LLM_PROVIDER` | AI 提供商（`openai` / `noop`） | noop |
| `LLM_API_KEY` | AI API Key | — |
| `LLM_BASE_URL` | AI API 地址 | `https://api.openai.com/v1` |
| `LLM_MODEL` | 模型名 | `gpt-4o-mini` |
| `ALLOW_REGISTER` | 是否开放注册 | true |

完整配置见 `.env.example`。

## 技术栈

**后端：** Rust · Axum · SeaORM · PostgreSQL
**前端：** React · TypeScript · Vite · Tailwind CSS · shadcn/ui
**AI：** OpenAI 兼容 API，可配置任意 provider

## License

MIT
