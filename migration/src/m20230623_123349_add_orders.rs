use sea_orm_migration::prelude::*;

const ORDER_STATUS_INDEX_NAME: &str = "idx-order_status";
const ORDER_EMAIL_INDEX_NAME: &str = "idx-order_email";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .col(
                        ColumnDef::new(Order::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Order::Status)
                            .string()
                            .not_null()
                            .default("created"),
                    )
                    .col(ColumnDef::new(Order::AccountId).uuid())
                    .col(ColumnDef::new(Order::Email).string())
                    .col(ColumnDef::new(Order::BillingAddressId).integer().not_null())
                    .col(
                        ColumnDef::new(Order::ShippingAddressId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Order::TransactionId).string())
                    .col(ColumnDef::new(Order::CardType).string())
                    .col(ColumnDef::new(Order::CardLast4).string())
                    .col(ColumnDef::new(Order::TaxAmount).decimal().not_null())
                    .col(ColumnDef::new(Order::ShippingAmount).decimal().not_null())
                    .col(ColumnDef::new(Order::TotalAmount).decimal().not_null())
                    .col(ColumnDef::new(Order::CreationDate).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order-account_id")
                            .from(Order::Table, Order::AccountId)
                            .to(Account::Table, Account::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(ORDER_STATUS_INDEX_NAME)
                    .table(Order::Table)
                    .col(Order::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(ORDER_EMAIL_INDEX_NAME)
                    .table(Order::Table)
                    .col(Order::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderAttribute::Table)
                    .col(
                        ColumnDef::new(OrderAttribute::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrderAttribute::Label).string().not_null())
                    .col(ColumnDef::new(OrderAttribute::Value).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderItem::Table)
                    .col(
                        ColumnDef::new(OrderItem::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrderItem::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderItem::ProductId).uuid().not_null())
                    .col(ColumnDef::new(OrderItem::Price).decimal().not_null())
                    .col(ColumnDef::new(OrderItem::Qty).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_item-order_id")
                            .from(OrderItem::Table, OrderItem::OrderId)
                            .to(Order::Table, Order::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_item-product_id")
                            .from(OrderItem::Table, OrderItem::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderItemAttribute::Table)
                    .col(
                        ColumnDef::new(OrderItemAttribute::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(OrderItemAttribute::OrderItemId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrderItemAttribute::Label)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OrderItemAttribute::Value).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_item_attribute-order_item")
                            .from(OrderItemAttribute::Table, OrderItemAttribute::Id)
                            .to(OrderItem::Table, OrderItem::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderAddress::Table)
                    .col(
                        ColumnDef::new(OrderAddress::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrderAddress::FirstName).string().not_null())
                    .col(ColumnDef::new(OrderAddress::LastName).string().not_null())
                    .col(ColumnDef::new(OrderAddress::Company).string())
                    .col(ColumnDef::new(OrderAddress::Street).string().not_null())
                    .col(ColumnDef::new(OrderAddress::Street2).string())
                    .col(ColumnDef::new(OrderAddress::City).string().not_null())
                    .col(ColumnDef::new(OrderAddress::State).string().not_null())
                    .col(ColumnDef::new(OrderAddress::PostalCode).string().not_null())
                    .col(
                        ColumnDef::new(OrderAddress::PhoneNumber)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderNote::Table)
                    .col(
                        ColumnDef::new(OrderNote::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrderNote::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderNote::Content).string().not_null())
                    .col(
                        ColumnDef::new(OrderNote::CreationDate)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_note-order_id")
                            .from(OrderNote::Table, OrderNote::OrderId)
                            .to(Order::Table, Order::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderNote::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrderAddress::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrderItemAttribute::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrderItem::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrderAttribute::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(ORDER_STATUS_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(ORDER_EMAIL_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Order {
    Table,
    Id,
    Status,
    AccountId,
    Email,
    BillingAddressId,
    ShippingAddressId,
    TransactionId,
    CardType,
    CardLast4,
    TaxAmount,
    ShippingAmount,
    TotalAmount,
    CreationDate,
}

#[derive(Iden)]
enum OrderAttribute {
    Table,
    Id,
    Label,
    Value,
}

#[derive(Iden)]
enum OrderItem {
    Table,
    Id,
    OrderId,
    ProductId,
    Price,
    Qty,
}

#[derive(Iden)]
enum OrderItemAttribute {
    Table,
    Id,
    OrderItemId,
    Label,
    Value,
}

#[derive(Iden)]
enum OrderAddress {
    Table,
    Id,
    FirstName,
    LastName,
    Company,
    Street,
    Street2,
    City,
    State,
    PostalCode,
    PhoneNumber,
}

#[derive(Iden)]
enum OrderNote {
    Table,
    Id,
    OrderId,
    Content,
    CreationDate,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}

#[derive(Iden)]
enum Product {
    Table,
    Id,
}
