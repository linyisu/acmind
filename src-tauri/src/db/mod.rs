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
    sqlx::query(include_str!("../../migrations/001_init.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("../../migrations/002_settings.sql"))
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
