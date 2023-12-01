use ::entity::{product, product::Entity as Product};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_product(
        db: &DbConn,
        new_data: product::Model,
    ) -> Result<product::ActiveModel, DbErr> {
        product::ActiveModel {
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_product_by_id(
        db: &DbConn,
        id: i32,
        new_data: product::Model,
    ) -> Result<product::Model, DbErr> {
        let post: product::ActiveModel = Product::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find product.".to_owned()))
            .map(Into::into)?;

        product::ActiveModel {
            id: post.id,
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn delete_product(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let post: product::ActiveModel = Product::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find product.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_products(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Product::delete_many().exec(db).await
    }
}
