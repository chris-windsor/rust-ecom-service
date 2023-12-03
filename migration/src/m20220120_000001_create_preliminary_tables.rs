use sea_orm_migration::prelude::*;

const DEFAULT_ACCOUNT_ROLE: &str = "customer";
const DEFAULT_ORDER_STATUS: &str = "created";

const ADDRESS_STREET_INDEX_NAME: &str = "idx_address_street";
const PRODUCT_DETAIL_NAME_INDEX_NAME: &str = "idx_product-detail_name";
const PRODUCT_SHORT_URL_INDEX_NAME: &str = "idx_product_short-url";
const PRODUCT_ATTRIBUTE_PRODUCT_ID_INDEX_NAME: &str = "idx_product-attribute_product-id";
const ORDER_STATUS_INDEX_NAME: &str = "idx_order_status";
const ORDER_EMAIL_INDEX_NAME: &str = "idx_order_email";

const PRODUCT_DETAIL_PARENT_ID_FK_NAME: &str = "fk_product-detail_parent-id";
const PRODUCT_CATEGORY_ID_FK_NAME: &str = "fk_product_category-id";
const PRODUCT_REVISION_ID_FK_NAME: &str = "fk_product_revision-id";
const PRODUCT_IMAGE_PRODUCT_ID_FK_NAME: &str = "fk_product-image_product-id";
const ATTRIBUTE_OPTION_ATTRIBUTE_ID_FK_NAME: &str = "fk_attribute-option_attribute-id";
const PRODUCT_ATTRIBUTE_PRODUCT_ID_FK_NAME: &str = "fk_product-attribute_product-id";
const PRODUCT_ATTRIBUTE_ATTRIBUTE_ID_FK_NAME: &str = "fk_product-attribute_attribute-id";
const CATEGORY_PARENT_ID_FK_NAME: &str = "fk_category_parent-id";
const ORDER_ACCOUNT_ID_FK_NAME: &str = "fk_order_account-id";
const ORDER_BILLING_ADDRESS_ID_FK_NAME: &str = "fk_order_billing-address-id";
const ORDER_SHIPPING_ADDRESS_ID_FK_NAME: &str = "fk_order_shipping-address-id";
const ORDER_ATTRIBUTE_ORDER_ID_FK_NAME: &str = "fk_order-attribute_order-id";
const ORDER_ITEM_ORDER_ID_FK_NAME: &str = "fk_order-item_order-id";
const ORDER_ITEM_PRODUCT_ID_FK_NAME: &str = "fk_order-item_product-id";
const ORDER_ITEM_ATTRIBUTE_ORDER_ITEM_ID_FK_NAME: &str = "fk_order-item-attribute_order-item-id";
const ORDER_NOTE_ORDER_ID_FK_NAME: &str = "fk_order-note_order-id";
const STOCK_PRODUCT_SHARED_ID_FK_NAME: &str = "fk_stock_product-shared-id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .col(
                        ColumnDef::new(Account::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Account::Name).string().not_null())
                    .col(ColumnDef::new(Account::Email).string().not_null())
                    .col(ColumnDef::new(Account::Password).string().not_null())
                    .col(
                        ColumnDef::new(Account::Role)
                            .string()
                            .not_null()
                            .default(DEFAULT_ACCOUNT_ROLE),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Address::Table)
                    .col(
                        ColumnDef::new(Address::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Address::FirstName).string().not_null())
                    .col(ColumnDef::new(Address::LastName).string().not_null())
                    .col(ColumnDef::new(Address::Company).string())
                    .col(ColumnDef::new(Address::Street).string().not_null())
                    .col(ColumnDef::new(Address::Street2).string())
                    .col(ColumnDef::new(Address::Street3).string())
                    .col(ColumnDef::new(Address::City).string().not_null())
                    .col(ColumnDef::new(Address::State).string().not_null())
                    .col(ColumnDef::new(Address::PostalCode).string().not_null())
                    .col(ColumnDef::new(Address::PhoneNumber).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(ADDRESS_STREET_INDEX_NAME)
                    .table(Address::Table)
                    .col(Address::Street)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .col(
                        ColumnDef::new(Category::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Category::Label).string().not_null())
                    .col(ColumnDef::new(Category::ParentId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name(CATEGORY_PARENT_ID_FK_NAME)
                            .from(Category::Table, Category::ParentId)
                            .to(Category::Table, Category::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductDetail::Table)
                    .col(
                        ColumnDef::new(ProductDetail::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(ProductDetail::Name).string().not_null())
                    .col(
                        ColumnDef::new(ProductDetail::Description)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProductDetail::Price).decimal().not_null())
                    .col(ColumnDef::new(ProductDetail::Upc).string())
                    .col(
                        ColumnDef::new(ProductDetail::RealWeight)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProductDetail::ShipWeight)
                            .integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(ProductDetail::ParentId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_DETAIL_PARENT_ID_FK_NAME)
                            .from(ProductDetail::Table, ProductDetail::ParentId)
                            .to(ProductDetail::Table, ProductDetail::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PRODUCT_DETAIL_NAME_INDEX_NAME)
                    .table(ProductDetail::Table)
                    .col(ProductDetail::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .col(
                        ColumnDef::new(Product::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Product::ShortUrl).string().not_null())
                    .col(ColumnDef::new(Product::CategoryId).integer())
                    .col(ColumnDef::new(Product::RevisionId).integer().not_null())
                    .col(
                        ColumnDef::new(Product::AllowBackOrder)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Product::AllowRestockNotifications)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_CATEGORY_ID_FK_NAME)
                            .from(Product::Table, Product::CategoryId)
                            .to(Category::Table, Category::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_REVISION_ID_FK_NAME)
                            .from(Product::Table, Product::RevisionId)
                            .to(ProductDetail::Table, ProductDetail::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PRODUCT_SHORT_URL_INDEX_NAME)
                    .table(Product::Table)
                    .col(Product::ShortUrl)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductImage::Table)
                    .col(
                        ColumnDef::new(ProductImage::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(ProductImage::HashId).uuid().not_null())
                    .col(ColumnDef::new(ProductImage::ProductId).integer().not_null())
                    .col(ColumnDef::new(ProductImage::Position).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_IMAGE_PRODUCT_ID_FK_NAME)
                            .from(ProductImage::Table, ProductImage::ProductId)
                            .to(ProductDetail::Table, ProductDetail::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Attribute::Table)
                    .col(
                        ColumnDef::new(Attribute::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Attribute::Kind).string().not_null())
                    .col(ColumnDef::new(Attribute::Label).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AttributeOption::Table)
                    .col(
                        ColumnDef::new(AttributeOption::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(AttributeOption::AttributeId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AttributeOption::Label).string().not_null())
                    .col(ColumnDef::new(AttributeOption::Content).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name(ATTRIBUTE_OPTION_ATTRIBUTE_ID_FK_NAME)
                            .from(AttributeOption::Table, AttributeOption::AttributeId)
                            .to(Attribute::Table, Attribute::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductAttribute::Table)
                    .col(
                        ColumnDef::new(ProductAttribute::Id)
                            .integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProductAttribute::ProductId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductAttribute::AttributeId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductAttribute::Content)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_ATTRIBUTE_PRODUCT_ID_FK_NAME)
                            .from(ProductAttribute::Table, ProductAttribute::ProductId)
                            .to(ProductDetail::Table, ProductDetail::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(PRODUCT_ATTRIBUTE_ATTRIBUTE_ID_FK_NAME)
                            .from(ProductAttribute::Table, ProductAttribute::AttributeId)
                            .to(Attribute::Table, Attribute::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(PRODUCT_ATTRIBUTE_PRODUCT_ID_INDEX_NAME)
                    .table(ProductAttribute::Table)
                    .col(ProductAttribute::ProductId)
                    .to_owned(),
            )
            .await?;

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
                            .default(DEFAULT_ORDER_STATUS),
                    )
                    .col(ColumnDef::new(Order::AccountId).integer())
                    .col(ColumnDef::new(Order::Email).string().not_null())
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
                            .name(ORDER_ACCOUNT_ID_FK_NAME)
                            .from(Order::Table, Order::AccountId)
                            .to(Account::Table, Account::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(ORDER_BILLING_ADDRESS_ID_FK_NAME)
                            .from(Order::Table, Order::BillingAddressId)
                            .to(Address::Table, Address::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(ORDER_SHIPPING_ADDRESS_ID_FK_NAME)
                            .from(Order::Table, Order::ShippingAddressId)
                            .to(Address::Table, Address::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(ORDER_STATUS_INDEX_NAME)
                    .table(Order::Table)
                    .col(Order::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
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
                    .col(ColumnDef::new(OrderAttribute::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderAttribute::Label).string().not_null())
                    .col(ColumnDef::new(OrderAttribute::Value).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name(ORDER_ATTRIBUTE_ORDER_ID_FK_NAME)
                            .from(OrderAttribute::Table, OrderAttribute::OrderId)
                            .to(Order::Table, Order::Id),
                    )
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
                    .col(ColumnDef::new(OrderItem::ProductId).integer().not_null())
                    .col(ColumnDef::new(OrderItem::Price).decimal().not_null())
                    .col(ColumnDef::new(OrderItem::Qty).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name(ORDER_ITEM_ORDER_ID_FK_NAME)
                            .from(OrderItem::Table, OrderItem::OrderId)
                            .to(Order::Table, Order::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(ORDER_ITEM_PRODUCT_ID_FK_NAME)
                            .from(OrderItem::Table, OrderItem::ProductId)
                            .to(ProductDetail::Table, ProductDetail::Id),
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
                            .name(ORDER_ITEM_ATTRIBUTE_ORDER_ITEM_ID_FK_NAME)
                            .from(OrderItemAttribute::Table, OrderItemAttribute::OrderItemId)
                            .to(OrderItem::Table, OrderItem::Id),
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
                            .name(ORDER_NOTE_ORDER_ID_FK_NAME)
                            .from(OrderNote::Table, OrderNote::OrderId)
                            .to(Order::Table, Order::Id),
                    )
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
                    .col(ColumnDef::new(Stock::ProductId).integer().not_null())
                    .col(ColumnDef::new(Stock::Amount).integer().not_null())
                    .col(ColumnDef::new(Stock::AdditionDate).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name(STOCK_PRODUCT_SHARED_ID_FK_NAME)
                            .from(Stock::Table, Stock::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

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

        manager
            .drop_table(Table::drop().table(Stock::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrderNote::Table).to_owned())
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

        manager
            .drop_table(Table::drop().table(AttributeOption::Table).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(PRODUCT_ATTRIBUTE_PRODUCT_ID_INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ProductAttribute::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Attribute::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProductImage::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(PRODUCT_SHORT_URL_INDEX_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(PRODUCT_DETAIL_NAME_INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ProductDetail::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Address::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
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
enum Address {
    Table,
    Id,
    FirstName,
    LastName,
    Company,
    Street,
    Street2,
    Street3,
    City,
    State,
    PostalCode,
    PhoneNumber,
}

#[derive(Iden)]
pub enum ProductDetail {
    Table,
    Id,
    Name,
    Description,
    Price,
    Upc,
    RealWeight,
    ShipWeight,
    ParentId,
}

#[derive(Iden)]
pub enum Product {
    Table,
    Id,
    ShortUrl,
    CategoryId,
    RevisionId,
    AllowBackOrder,
    AllowRestockNotifications,
}

#[derive(Iden)]
enum ProductImage {
    Table,
    Id,
    HashId,
    ProductId,
    Position,
}

#[derive(Iden)]
pub enum Attribute {
    Table,
    Id,
    Label,
    Kind,
}

#[derive(Iden)]
pub enum AttributeOption {
    Table,
    Id,
    AttributeId,
    Label,
    Content,
}

#[derive(Iden)]
pub enum ProductAttribute {
    Table,
    Id,
    ProductId,
    AttributeId,
    Content,
}

#[derive(Iden)]
pub enum Category {
    Table,
    Id,
    Label,
    ParentId,
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
    OrderId,
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
enum OrderNote {
    Table,
    Id,
    OrderId,
    Content,
    CreationDate,
}

#[derive(Iden)]
enum Stock {
    Table,
    Id,
    ProductId,
    Amount,
    AdditionDate,
}

#[derive(Iden)]
enum StaticPage {
    Table,
    Id,
    Slug,
    Title,
    Content,
}
