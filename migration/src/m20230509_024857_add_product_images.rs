use super::m20220120_000001_create_preliminary_tables::Product;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProductImage::Table)
                    .col(
                        ColumnDef::new(ProductImage::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProductImage::ProductId).uuid().not_null())
                    .col(ColumnDef::new(ProductImage::Position).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product-image_id")
                            .from(ProductImage::Table, ProductImage::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductImage::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum ProductImage {
    Table,
    Id,
    ProductId,
    Position,
}
