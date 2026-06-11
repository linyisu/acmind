use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiAnalysis::Table)
                    .if_not_exists()
                    .col(pk_auto(AiAnalysis::Id).big_integer())
                    .col(ColumnDef::new(AiAnalysis::UserId).big_integer().not_null())
                    .col(string(AiAnalysis::TargetType).not_null())
                    .col(
                        ColumnDef::new(AiAnalysis::TargetId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AiAnalysis::Result).json_binary().not_null())
                    .col(
                        timestamp_with_time_zone(AiAnalysis::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_analysis_user")
                            .from(AiAnalysis::Table, AiAnalysis::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_analysis_user")
                    .table(AiAnalysis::Table)
                    .col(AiAnalysis::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_analysis_target")
                    .table(AiAnalysis::Table)
                    .col(AiAnalysis::TargetType)
                    .col(AiAnalysis::TargetId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiAnalysis::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AiAnalysis {
    Table,
    Id,
    UserId,
    TargetType,
    TargetId,
    Result,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
