pub use sea_orm_migration::prelude::*;

mod m20220120_000001_create_preliminary_tables;
mod m20230509_024857_add_product_images;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220120_000001_create_preliminary_tables::Migration),
            Box::new(m20230509_024857_add_product_images::Migration),
        ]
    }
}
