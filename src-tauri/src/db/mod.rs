pub mod models;
pub mod repo;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

/// Initialize the database: create pool, run migrations, seed initial data.
pub async fn init_db(app_data_dir: &PathBuf) -> Result<SqlitePool, sqlx::Error> {
    // Ensure data directory exists
    std::fs::create_dir_all(app_data_dir).ok();

    let db_path = app_data_dir.join("acmind.sqlite");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await?;

    run_migrations(&pool).await?;
    seed_knowledge_points(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS problems (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            source_problem_id TEXT NOT NULL,
            title TEXT NOT NULL,
            url TEXT,
            difficulty INTEGER,
            tags TEXT NOT NULL DEFAULT '[]',
            statement_path TEXT,
            created_at DATETIME NOT NULL DEFAULT (datetime('now')),
            updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS submissions (
            id TEXT PRIMARY KEY,
            problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('AC','WA','TLE','RE','MLE','CE')),
            language TEXT NOT NULL DEFAULT 'C++',
            code_path TEXT NOT NULL,
            runtime INTEGER,
            memory INTEGER,
            note TEXT,
            submitted_at DATETIME NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS solution_notes (
            id TEXT PRIMARY KEY,
            problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
            note_type TEXT NOT NULL CHECK(note_type IN ('official','community','self','ai')),
            content TEXT NOT NULL DEFAULT '',
            source_url TEXT,
            created_at DATETIME NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS error_analyses (
            id TEXT PRIMARY KEY,
            problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
            submission_id TEXT NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
            error_type TEXT NOT NULL,
            root_cause TEXT NOT NULL DEFAULT '',
            fix_summary TEXT NOT NULL DEFAULT '',
            related_knowledge TEXT NOT NULL DEFAULT '[]',
            created_at DATETIME NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS knowledge_points (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL,
            parent_id TEXT
        );

        CREATE TABLE IF NOT EXISTS problem_knowledge (
            problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
            knowledge_point_id TEXT NOT NULL REFERENCES knowledge_points(id) ON DELETE CASCADE,
            confidence REAL NOT NULL DEFAULT 1.0,
            PRIMARY KEY (problem_id, knowledge_point_id)
        );

        CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            report_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_submissions_problem ON submissions(problem_id);
        CREATE INDEX IF NOT EXISTS idx_notes_problem ON solution_notes(problem_id);
        CREATE INDEX IF NOT EXISTS idx_errors_problem ON error_analyses(problem_id);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Seed initial knowledge points if the table is empty.
async fn seed_knowledge_points(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM knowledge_points")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    let categories = vec![
        ("DP", "Dynamic Programming"),
        ("Graph", "Graph Theory"),
        ("Math", "Mathematics"),
        ("DS", "Data Structures"),
        ("String", "String Algorithms"),
        ("Greedy", "Greedy Algorithms"),
        ("Geometry", "Computational Geometry"),
        ("Search", "Search & Enumeration"),
        ("Other", "Other"),
    ];

    for (id, name) in &categories {
        sqlx::query(
            "INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES (?1, ?2, ?3, NULL)",
        )
        .bind(*id)
        .bind(*name)
        .bind(*id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
