use acmind_migration::{Migrator, MigratorTrait};
use sea_orm_migration::sea_orm::Database;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://acmind:acmind@localhost:5432/acmind)");

    let db = Database::connect(&database_url)
        .await
        .expect("connect to db");

    Migrator::up(&db, None).await.expect("run migrations");

    println!("✓ migrations applied");
}
