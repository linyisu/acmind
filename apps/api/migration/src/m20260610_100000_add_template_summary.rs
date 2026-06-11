use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Template::Table)
                    .add_column(
                        ColumnDef::new(Template::Summary)
                            .string_len(500)
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Template::Table)
                    .drop_column(Template::Summary)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Template {
    Table,
    Summary,
}
