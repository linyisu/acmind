# ACMind

Personal algorithm training knowledge base — Web fullstack project.

Algorithm training leaves a lot of breadcrumbs: problems, submissions, verdict
histories, templates, snippets, and notes. ACMind is a single place to keep
them, with enough structured querying to ask "where am I struggling?" without
leaving the app.

## Tech Stack

**Backend:** Rust · Axum 0.7 · SeaORM 1 · PostgreSQL 16 · Tokio
**Frontend:** React 19 · Vite 6 · TypeScript 5 · Tailwind 4 · shadcn/ui · TanStack Query 5 · Zustand 5
**Infrastructure:** pnpm workspace · turbo · Docker Compose · GitHub Actions

## Quick Start (Docker)

```bash
cp .env.example .env  # set JWT_SECRET
docker compose up -d --build
```

Open http://localhost:5173 — the React app. API at :8080, Postgres at :5432.

## Local Development (without Docker)

```bash
# Postgres (in another shell)
docker compose up -d postgres

# Backend
cd apps/api
DATABASE_URL=postgres://acmind:acmind@localhost:5432/acmind \
  JWT_SECRET=devsecret cargo run

# Frontend
cd apps/web
pnpm install
pnpm dev  # http://localhost:5173
```

The first API start runs SeaORM migrations automatically.

## Project Structure

```
acmind/
├── apps/
│   ├── api/                Rust Axum backend (auth, problem, submission,
│   │                       knowledge, tag, analysis modules)
│   └── web/                React + Vite SPA
├── packages/
│   └── shared/             Shared TypeScript types
├── docs/superpowers/       Design spec and implementation plan
├── docker-compose.yml
└── .github/workflows/ci.yml
```

## API

All endpoints under `/api/v1` (except `/auth/register` and `/auth/login`)
require `Authorization: Bearer <jwt>`.

| Method | Path | Description |
|--------|------|-------------|
| GET    | `/health` | Liveness probe |
| POST   | `/api/v1/auth/register` | Register a user |
| POST   | `/api/v1/auth/login` | Login, returns JWT |
| GET    | `/api/v1/auth/me` | Current user |
| GET    | `/api/v1/problems` | List problems (filter by `tag_id`) |
| POST   | `/api/v1/problems` | Create problem |
| GET    | `/api/v1/problems/:id` | Get problem |
| PATCH  | `/api/v1/problems/:id` | Update problem |
| DELETE | `/api/v1/problems/:id` | Delete problem |
| GET    | `/api/v1/submissions` | List submissions |
| POST   | `/api/v1/submissions` | Log a submission |
| GET    | `/api/v1/submissions/:id` | Get submission |
| GET    | `/api/v1/knowledge` | List knowledge entries |
| POST   | `/api/v1/knowledge` | Create knowledge entry |
| GET    | `/api/v1/knowledge/:id` | Get knowledge entry |
| PATCH  | `/api/v1/knowledge/:id` | Update knowledge entry |
| DELETE | `/api/v1/knowledge/:id` | Delete knowledge entry |
| GET    | `/api/v1/tags` | List tags |
| POST   | `/api/v1/tags` | Create tag |
| DELETE | `/api/v1/tags/:id` | Delete tag |
| GET    | `/api/v1/analysis/submissions/summary` | Total / AC rate / verdict histogram |
| GET    | `/api/v1/analysis/submissions/timeline` | Per-day submission counts |
| GET    | `/api/v1/analysis/problems/difficulty-distribution` | Difficulty histogram with JOIN |

## Analysis

The `analysis` module runs SQL aggregations directly on PostgreSQL via SeaORM:

- `summary` — total submissions, AC rate, verdict counts
- `timeline` — daily submission and AC counts (filterable by date range)
- `difficulty_distribution` — joins `submission` and `problem` to bucket
  attempts by problem difficulty

## Testing

```bash
# Backend
cd apps/api
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo test

# Frontend
cd apps/web
pnpm exec tsc -b
```

## License

MIT
