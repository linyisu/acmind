pub use sea_orm_migration::prelude::*;

mod m20260101_000001_create_user;
mod m20260101_000002_create_problem;
mod m20260101_000003_create_submission;
mod m20260101_000004_create_knowledge;
mod m20260101_000005_create_tag;
mod m20260101_000006_create_join_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260101_000001_create_user::Migration),
            Box::new(m20260101_000002_create_problem::Migration),
            Box::new(m20260101_000003_create_submission::Migration),
            Box::new(m20260101_000004_create_knowledge::Migration),
            Box::new(m20260101_000005_create_tag::Migration),
            Box::new(m20260101_000006_create_join_tables::Migration),
        ]
    }
}
