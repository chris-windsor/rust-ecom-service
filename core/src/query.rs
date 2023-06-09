use ::entity::{product, product::Entity as Product};
use ::entity::{product_image, product_image::Entity as ProductImage};
use sea_orm::{prelude::Uuid, *};

pub struct Query;

impl Query {
    pub async fn find_product_by_id(
        db: &DbConn,
        id: Uuid,
    ) -> Result<Option<product::Model>, DbErr> {
        Product::find_by_id(id).one(db).await
    }

    pub async fn find_products_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<(product::Model, Option<product_image::Model>)>, u64), DbErr> {
        let paginator = Product::find()
            .select()
            .column(product::Column::Id)
            .column(product::Column::ShortUrl)
            .column(product::Column::Name)
            .column(product::Column::Price)
            .column(product::Column::Stock)
            .find_also_related(ProductImage)
            .order_by_asc(product::Column::Name)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
