# ACMind — Agent Instructions

## Overview
ACMind is a personal ACM/ICPC training desktop app built with:
- **Frontend**: React 19 + Vite + TypeScript + TailwindCSS v4 + shadcn/ui
- **Backend**: Tauri v2 (Rust) with SQLite (sqlx), file-system storage
- **Package manager**: pnpm

## Project Structure
```
src/                          # React frontend
  components/ui/              # shadcn/ui components (button, card, badge, etc.)
  components/layout/          # Layout components (AppSidebar)
  pages/                      # Route pages (Dashboard, Problems, ProblemDetail, etc.)
  lib/                        # Types, utils, API layer
  hooks/                      # Custom React hooks
  test/                       # Test setup
src-tauri/
  src/
    db/                       # models.rs, repo.rs, mod.rs (SQLite schema + CRUD)
    commands/                 # Tauri command handlers
    storage/                  # File-system operations
    ai/                       # AI analysis module (placeholder)
    error.rs                  # AppError type
  migrations/                 # SQL migration files
```

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
