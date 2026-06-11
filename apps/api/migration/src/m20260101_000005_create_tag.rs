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
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(pk_auto(Tag::Id).big_integer())
                    .col(big_integer(Tag::UserId))
                    .col(string(Tag::Name))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tag_user")
                            .from(Tag::Table, Tag::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uniq_tag_user_name")
                    .table(Tag::Table)
                    .col(Tag::UserId)
                    .col(Tag::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Id,
    UserId,
    Name,
}
