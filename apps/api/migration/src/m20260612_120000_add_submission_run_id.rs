use sea_orm_migration::prelude::*;

use super::m20260101_000003_create_submission::Submission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // External run id (e.g. VJudge runId). Nullable: manual submissions and
        // submissions whose source fetch failed keep it NULL.
        // MySQL: use VARCHAR instead of TEXT for indexing
        manager
            .alter_table(
                Table::alter()
                    .table(Submission::Table)
                    .add_column(ColumnDef::new(Submission::ExternalRunId).string().null())
                    .to_owned(),
            )
            .await?;

        // MySQL doesn't support partial indexes with WHERE clause
        // Create a regular unique index instead
        // NULL values won't violate uniqueness constraint in MySQL
        manager
            .create_index(
                Index::create()
                    .name("idx_submission_user_run_id")
                    .table(Submission::Table)
                    .col(Submission::UserId)
                    .col(Submission::ExternalRunId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_submission_user_run_id")
                    .table(Submission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Submission::Table)
                    .drop_column(Submission::ExternalRunId)
                    .to_owned(),
            )
            .await
    }
}
