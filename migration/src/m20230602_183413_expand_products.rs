use sea_orm_migration::prelude::*;

const PRODUCT_SHORTURL_INDEX_NAME: &str = "idx-product_shorturl";
const PRODUCTATTRIBUTE_PRODUCTID_INDEX_NAME: &str = "idx-productattribute_productid";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .add_column(
                        ColumnDef::new(Product::AllowBackorder)
                            .boolean()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Product::AllowRestockNotifications)
                            .boolean()
                            .default(true),
                    )
                    .add_column(ColumnDef::new(Product::ShortUrl).string().not_null())
                    .add_column(ColumnDef::new(Product::Upc).integer())
                    .add_column(ColumnDef::new(Product::RealWeight).integer().default(0))
                    .add_column(ColumnDef::new(Product::ShipWeight).integer().default(0))
                    .add_column(ColumnDef::new(Product::ParentId).uuid())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-product_parent-id")
                            .from_tbl(Product::Table)
                            .from_col(Product::ParentId)
                            .to_tbl(Product::Table)
                            .to_col(Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(PRODUCT_SHORTURL_INDEX_NAME)
                    .table(Product::Table)
                    .col(Product::ShortUrl)
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
                            .not_null()
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
                            .not_null()
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
                            .name("fk-attribute-option_attribute-id")
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
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProductAttribute::ProductId)
                            .uuid()
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
                            .name("fk-product-attribute_product-id")
                            .from(ProductAttribute::Table, ProductAttribute::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product-attribute_attribute-id")
                            .from(ProductAttribute::Table, ProductAttribute::AttributeId)
                            .to(Attribute::Table, Attribute::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(PRODUCTATTRIBUTE_PRODUCTID_INDEX_NAME)
                    .table(ProductAttribute::Table)
                    .col(ProductAttribute::ProductId)
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
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Category::Label).string().not_null())
                    .col(ColumnDef::new(Category::ParentId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-category_parent-id")
                            .from(Category::Table, Category::ParentId)
                            .to(Category::Table, Category::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductCategory::Table)
                    .col(
                        ColumnDef::new(ProductCategory::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(ProductCategory::ProductId).uuid().not_null())
                    .col(
                        ColumnDef::new(ProductCategory::CategoryId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product-category_product-id")
                            .from(ProductCategory::Table, ProductCategory::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product-category_category-id")
                            .from(ProductCategory::Table, ProductCategory::CategoryId)
                            .to(Category::Table, Category::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(PRODUCT_SHORTURL_INDEX_NAME).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .drop_foreign_key(Alias::new("fk-product_parent-id"))
                    .drop_column(Product::AllowBackorder)
                    .drop_column(Product::AllowRestockNotifications)
                    .drop_column(Product::ShortUrl)
                    .drop_column(Product::Upc)
                    .drop_column(Product::RealWeight)
                    .drop_column(Product::ShipWeight)
                    .drop_column(Product::ParentId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AttributeOption::Table).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(PRODUCTATTRIBUTE_PRODUCTID_INDEX_NAME)
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
            .drop_table(Table::drop().table(ProductCategory::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Product {
    Table,
    Id,
    AllowBackorder,
    AllowRestockNotifications,
    ShortUrl,
    Upc,
    RealWeight,
    ShipWeight,
    ParentId,
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
pub enum ProductCategory {
    Table,
    Id,
    ProductId,
    CategoryId,
}
