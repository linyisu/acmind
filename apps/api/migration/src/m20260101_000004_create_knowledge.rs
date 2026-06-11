use sea_orm_migration::{prelude::*, schema::*};

use super::m20260101_000001_create_user::User;
use super::m20260101_000002_create_problem::Problem;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Knowledge::Table)
                    .if_not_exists()
                    .col(pk_auto(Knowledge::Id).big_integer())
                    .col(big_integer(Knowledge::UserId))
                    .col(big_integer_null(Knowledge::ProblemId))
                    .col(string(Knowledge::Kind))
                    .col(string(Knowledge::Title))
                    .col(text(Knowledge::Content))
                    .col(
                        timestamp_with_time_zone(Knowledge::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Knowledge::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledge_user")
                            .from(Knowledge::Table, Knowledge::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledge_problem")
                            .from(Knowledge::Table, Knowledge::ProblemId)
                            .to(Problem::Table, Problem::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_knowledge_user")
                    .table(Knowledge::Table)
                    .col(Knowledge::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Knowledge::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Knowledge {
    Table,
    Id,
    UserId,
    ProblemId,
    Kind,
    Title,
    Content,
    CreatedAt,
    UpdatedAt,
}
