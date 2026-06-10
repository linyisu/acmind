use sea_orm_migration::{prelude::*, schema::*};

use super::m20260101_000001_create_user::User;
use super::m20260101_000002_create_problem::Problem;
use super::m20260101_000005_create_tag::Tag;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── template table ──────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Template::Table)
                    .if_not_exists()
                    .col(pk_auto(Template::Id).big_integer())
                    .col(big_integer(Template::UserId))
                    .col(string(Template::Title))
                    .col(string(Template::Category).default("other"))
                    .col(string(Template::Language).default("cpp"))
                    .col(text(Template::Code))
                    .col(text(Template::Description).default(""))
                    .col(string_null(Template::TimeComplexity))
                    .col(string_null(Template::SpaceComplexity))
                    .col(string(Template::Source).default("manual"))
                    .col(big_integer_null(Template::SourceProblemId))
                    .col(integer_null(Template::Difficulty))
                    .col(integer(Template::UsageCount).default(0))
                    .col(
                        timestamp_with_time_zone(Template::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Template::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_user")
                            .from(Template::Table, Template::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_source_problem")
                            .from(Template::Table, Template::SourceProblemId)
                            .to(Problem::Table, Problem::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // indexes for template
        manager
            .create_index(
                Index::create()
                    .name("idx_template_user")
                    .table(Template::Table)
                    .col(Template::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_user_category")
                    .table(Template::Table)
                    .col(Template::UserId)
                    .col(Template::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_user_language")
                    .table(Template::Table)
                    .col(Template::UserId)
                    .col(Template::Language)
                    .to_owned(),
            )
            .await?;

        // ── template_tag (many-to-many) ─────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(TemplateTag::Table)
                    .if_not_exists()
                    .col(big_integer(TemplateTag::TemplateId))
                    .col(big_integer(TemplateTag::TagId))
                    .primary_key(
                        Index::create()
                            .col(TemplateTag::TemplateId)
                            .col(TemplateTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_tag_template")
                            .from(TemplateTag::Table, TemplateTag::TemplateId)
                            .to(Template::Table, Template::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_tag_tag")
                            .from(TemplateTag::Table, TemplateTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ── template_problem (many-to-many) ─────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(TemplateProblem::Table)
                    .if_not_exists()
                    .col(big_integer(TemplateProblem::TemplateId))
                    .col(big_integer(TemplateProblem::ProblemId))
                    .col(
                        timestamp_with_time_zone(TemplateProblem::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(TemplateProblem::TemplateId)
                            .col(TemplateProblem::ProblemId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_problem_template")
                            .from(TemplateProblem::Table, TemplateProblem::TemplateId)
                            .to(Template::Table, Template::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_problem_problem")
                            .from(TemplateProblem::Table, TemplateProblem::ProblemId)
                            .to(Problem::Table, Problem::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TemplateProblem::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TemplateTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Template::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Template {
    Table,
    Id,
    UserId,
    Title,
    Category,
    Language,
    Code,
    Description,
    TimeComplexity,
    SpaceComplexity,
    Source,
    SourceProblemId,
    Difficulty,
    UsageCount,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum TemplateTag {
    Table,
    TemplateId,
    TagId,
}

#[derive(DeriveIden)]
pub enum TemplateProblem {
    Table,
    TemplateId,
    ProblemId,
    CreatedAt,
}
