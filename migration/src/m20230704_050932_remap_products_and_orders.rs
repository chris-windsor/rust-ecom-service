use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .add_column(ColumnDef::new(Product::SharedId).uuid().not_null())
                    .add_column(ColumnDef::new(Product::ActiveRevision).boolean().not_null())
                    .drop_column(Product::Stock)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Stock::Table)
                    .col(
                        ColumnDef::new(Stock::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Stock::SharedId).uuid().not_null())
                    .col(ColumnDef::new(Stock::Amount).integer().not_null())
                    .col(ColumnDef::new(Stock::EntryDate).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OrderItem::Table)
                    .drop_foreign_key(Alias::new("fk-order_item-product_id"))
                    .drop_column(OrderItem::ProductId)
                    .add_column(ColumnDef::new(OrderItem::SharedId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "\n\nPlease run `sea migrate fresh`... theres too much going on.\nWill probably reset migrations before proper release\n\n".to_string(),
        ))
    }
}

#[derive(Iden)]
enum Product {
    Table,
    Stock,
    SharedId,
    ActiveRevision,
}

#[derive(Iden)]
enum Stock {
    Table,
    Id,
    SharedId,
    Amount,
    EntryDate,
}

#[derive(Iden)]
enum OrderItem {
    Table,
    ProductId,
    SharedId,
}
