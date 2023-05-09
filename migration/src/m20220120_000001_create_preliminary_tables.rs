use sea_orm_migration::prelude::*;

const ACCOUNT_EMAIL_INDEX_NAME: &str = "idx-account_email";
const PRODUCT_NAME_INDEX_NAME: &str = "idx-product_name";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .col(ColumnDef::new(Account::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Account::Name).string().not_null())
                    .col(ColumnDef::new(Account::Email).string().not_null())
                    .col(ColumnDef::new(Account::Password).string().not_null())
                    .col(
                        ColumnDef::new(Account::Role)
                            .string()
                            .not_null()
                            .default("customer"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(ACCOUNT_EMAIL_INDEX_NAME)
                    .table(Account::Table)
                    .col(Account::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .col(ColumnDef::new(Product::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Product::Name).string().not_null())
                    .col(ColumnDef::new(Product::Description).string().not_null())
                    .col(ColumnDef::new(Product::Price).decimal().not_null())
                    .col(ColumnDef::new(Product::Stock).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(PRODUCT_NAME_INDEX_NAME)
                    .table(Product::Table)
                    .col(Product::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(ACCOUNT_EMAIL_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(PRODUCT_NAME_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
    Name,
    Email,
    Password,
    Role,
}

#[derive(Iden)]
pub enum Product {
    Table,
    Id,
    Name,
    Description,
    Price,
    Stock,
}
