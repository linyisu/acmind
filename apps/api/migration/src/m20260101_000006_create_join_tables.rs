use sea_orm_migration::{prelude::*, schema::*};

use super::m20260101_000002_create_problem::Problem;
use super::m20260101_000004_create_knowledge::Knowledge;
use super::m20260101_000005_create_tag::Tag;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // problem_tag (many-to-many between problem and tag)
        manager
            .create_table(
                Table::create()
                    .table(ProblemTag::Table)
                    .if_not_exists()
                    .col(big_integer(ProblemTag::ProblemId))
                    .col(big_integer(ProblemTag::TagId))
                    .primary_key(
                        Index::create()
                            .col(ProblemTag::ProblemId)
                            .col(ProblemTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_problem_tag_problem")
                            .from(ProblemTag::Table, ProblemTag::ProblemId)
                            .to(Problem::Table, Problem::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_problem_tag_tag")
                            .from(ProblemTag::Table, ProblemTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // knowledge_tag (many-to-many between knowledge and tag)
        manager
            .create_table(
                Table::create()
                    .table(KnowledgeTag::Table)
                    .if_not_exists()
                    .col(big_integer(KnowledgeTag::KnowledgeId))
                    .col(big_integer(KnowledgeTag::TagId))
                    .primary_key(
                        Index::create()
                            .col(KnowledgeTag::KnowledgeId)
                            .col(KnowledgeTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledge_tag_knowledge")
                            .from(KnowledgeTag::Table, KnowledgeTag::KnowledgeId)
                            .to(Knowledge::Table, Knowledge::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledge_tag_tag")
                            .from(KnowledgeTag::Table, KnowledgeTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KnowledgeTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProblemTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ProblemTag {
    Table,
    ProblemId,
    TagId,
}

#[derive(DeriveIden)]
pub enum KnowledgeTag {
    Table,
    KnowledgeId,
    TagId,
}
