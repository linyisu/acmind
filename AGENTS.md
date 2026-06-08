# ACMind — Agent Instructions

## Overview
See `CLAUDE.md` for project architecture, tech stack, and SeaORM schema workflow.

## Rules (MUST FOLLOW)

### 1. TDD — Test First, Always
- **Every feature starts with a failing test.**
- Rust: `#[cfg(test)]` modules with in-memory SQLite (no mocking the database).
- Frontend: vitest + @testing-library/react + jsdom.
- Run `pnpm test` (frontend) and `cargo test --manifest-path src-tauri/Cargo.toml` (Rust) after every change.
- If tests fail after a change, fix the code or update the test — **never skip or delete a test to make things pass.**
- Target: all tests green before any commit.

### 2. Git — Commit After Every Working Step
- Commit frequently and atomically: one logical change per commit.
- Commit message format: `<type>: <description>` (e.g., `feat: add problem CRUD`, `test: repo layer tests`, `fix: badge variant class names`).
- Types: `feat`, `fix`, `test`, `refactor`, `chore`, `docs`.
- **Never commit broken code** — all tests must pass first.
- After implementing a feature or fixing a bug, run tests then commit immediately.

### 3. Minimal Changes
- Only change what's needed to implement the feature or fix the bug.
- Don't refactor unrelated code, rename things unnecessarily, or "clean up" code outside your task scope.
- Follow existing patterns in the file you're editing.

### 4. Read Before Write
- Always read the file you're about to edit before making changes.
- During iterative edits, prefer targeted compile/type checks over repeated formatting to avoid unnecessary file churn.
- Only run formatting and Clippy as the final pre-commit gate, right before committing.
- For Rust: check compilation with `cargo check` before committing.
- For TypeScript: check with `pnpm tsc -b`.

### 4. Architecture Hygiene — No God Files
- Keep boundaries explicit: HTTP/Tauri handlers parse requests and return responses; service modules own business workflows; repo modules only do persistence; storage modules only do file IO.
- Do not add new business logic to files already mixing 3+ responsibilities. Extract a small service/helper first, then call it.
- Avoid duplicate workflows. If browser-extension import and GUI import need the same behavior, share a Rust service or a shared JS helper instead of copy-pasting logic.
- Do not use fire-and-forget success semantics. UI success must mean the backend operation actually completed successfully.
- Do not swallow production errors with `let _ =`, `unwrap_or_default()`, empty `catch`, or fake success responses. Either propagate the error or log it with enough context and return an accurate partial-success result.
- Keep import/sync flows idempotent: check whether data already exists before writing files, and report `created`, `updated`, `skipped`, and `source_synced` according to what actually happened.

### 5. Error Handling (Rust)
- Use `AppError` (defined in `src-tauri/src/error.rs`) for all fallible operations.
- Commands return `Result<T, AppError>` — Tauri serializes it for the frontend.
- Don't `unwrap()` in production code; use `?` and proper error propagation.

### 6. Database
- SQLite via sqlx, connection pool managed by Tauri state.
- All schema changes go in `src-tauri/migrations/` as numbered SQL files.
- Use `sqlx::query_as` with derived `FromRow` structs for type-safe queries.
- Foreign keys with `ON DELETE CASCADE` for referential integrity.

### 7. Frontend API Calls
- Use `@tauri-apps/api/core` `invoke()` for all backend communication.
- Wrap in try/catch with fallback mock data for browser-only dev mode.
- Use `@tanstack/react-query` for data fetching with proper cache invalidation.

### 8. Styling
- TailwindCSS v4 with `@tailwindcss/vite` plugin.
- Use the `cn()` utility for merging class names.
- shadcn/ui components in `src/components/ui/` follow the cva (class-variance-authority) pattern.
- Custom ACM theme colors: `success`, `warning`, `error` (defined in `globals.css`).

## Code Review Checklist
Before committing work:
- [ ] Run formatting once at the end (`cargo fmt --manifest-path src-tauri/Cargo.toml` and applicable frontend formatter)
- [ ] Rust Clippy passes (`cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`)
- [ ] All tests pass (`pnpm test` + `cargo test`)
- [ ] TypeScript compiles (`pnpm tsc -b`)
- [ ] Rust compiles (`cargo check`)
- [ ] No leftover debug prints (`console.log`, `dbg!`, `println!`)
- [ ] Git committed with descriptive message

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
