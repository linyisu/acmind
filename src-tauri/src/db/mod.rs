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
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA journal_mode = WAL")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA busy_timeout = 5000")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&db_url)
        .await?;

    run_migrations(&pool).await?;
    clean_orphaned_records(&pool).await?;
    seed_knowledge_points(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("../../migrations/001_init.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("../../migrations/002_settings.sql"))
        .execute(pool)
        .await?;

    ensure_column(pool, "submissions", "external_run_id", "TEXT").await?;

    sqlx::query(include_str!(
        "../../migrations/003_external_submissions.sql"
    ))
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), sqlx::Error> {
    let pragma = format!("PRAGMA table_info({})", table);
    let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&pragma).fetch_all(pool).await?;

    if rows.iter().any(|(_, name, _, _, _, _)| name == column) {
        return Ok(());
    }

    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition);
    sqlx::query(&sql).execute(pool).await?;
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

/// Clean up orphaned records caused by missing FK enforcement in earlier versions.
async fn clean_orphaned_records(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM submissions WHERE problem_id NOT IN (SELECT id FROM problems)")
        .execute(pool)
        .await?;

    sqlx::query(
        "DELETE FROM error_analyses WHERE problem_id NOT IN (SELECT id FROM problems) OR submission_id NOT IN (SELECT id FROM submissions)",
    )
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM solution_notes WHERE problem_id NOT IN (SELECT id FROM problems)")
        .execute(pool)
        .await?;

    Ok(())
}
