use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StaticPage::Table)
                    .col(
                        ColumnDef::new(StaticPage::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(StaticPage::Slug).string().not_null())
                    .col(ColumnDef::new(StaticPage::Title).string().not_null())
                    .col(ColumnDef::new(StaticPage::Content).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StaticPage::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum StaticPage {
    Table,
    Id,
    Slug,
    Title,
    Content,
}
