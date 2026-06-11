use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(pk_auto(Task::Id).big_integer())
                    .col(ColumnDef::new(Task::UserId).big_integer().not_null())
                    .col(string(Task::Kind).not_null())
                    .col(string(Task::Status).not_null().default("pending"))
                    .col(string(Task::TargetType).not_null())
                    .col(ColumnDef::new(Task::TargetId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Task::Progress)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .col(ColumnDef::new(Task::Result).json_binary().null())
                    .col(ColumnDef::new(Task::Error).text().null())
                    .col(
                        timestamp_with_time_zone(Task::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Task::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Task::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_user")
                            .from(Task::Table, Task::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_user")
                    .table(Task::Table)
                    .col(Task::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_user_status")
                    .table(Task::Table)
                    .col(Task::UserId)
                    .col(Task::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    UserId,
    Kind,
    Status,
    TargetType,
    TargetId,
    Progress,
    Result,
    Error,
    CreatedAt,
    StartedAt,
    CompletedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
