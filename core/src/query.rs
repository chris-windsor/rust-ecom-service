use ::entity::{product, product::Entity as Product};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_product_by_id(db: &DbConn, id: i32) -> Result<Option<product::Model>, DbErr> {
        Product::find_by_id(id).one(db).await
    }

    pub async fn find_products_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<product::Model>, u64), DbErr> {
        let paginator = Product::find()
            .select()
            .column(product::Column::Id)
            .column(product::Column::ShortUrl)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
