use sea_orm_migration::{prelude::*, schema::*};

use super::m20260101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Problem::Table)
                    .if_not_exists()
                    .col(pk_auto(Problem::Id).big_integer())
                    .col(big_integer(Problem::UserId))
                    .col(string(Problem::Source))
                    .col(string_null(Problem::ExternalId))
                    .col(string(Problem::Title))
                    .col(string_null(Problem::Url))
                    .col(integer_null(Problem::Difficulty))
                    .col(text_null(Problem::Statement))
                    .col(
                        timestamp_with_time_zone(Problem::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Problem::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_problem_user")
                            .from(Problem::Table, Problem::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_problem_user")
                    .table(Problem::Table)
                    .col(Problem::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Problem::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Problem {
    Table,
    Id,
    UserId,
    Source,
    ExternalId,
    Title,
    Url,
    Difficulty,
    Statement,
    CreatedAt,
    UpdatedAt,
}
