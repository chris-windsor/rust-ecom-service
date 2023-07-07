pub use sea_orm_migration::prelude::*;

mod m20220120_000001_create_preliminary_tables;
mod m20230509_024857_add_product_images;
mod m20230602_183413_expand_products;
mod m20230623_123349_add_orders;
mod m20230704_050932_remap_products_and_orders;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220120_000001_create_preliminary_tables::Migration),
            Box::new(m20230509_024857_add_product_images::Migration),
            Box::new(m20230602_183413_expand_products::Migration),
            Box::new(m20230623_123349_add_orders::Migration),
            Box::new(m20230704_050932_remap_products_and_orders::Migration),
        ]
    }
}
