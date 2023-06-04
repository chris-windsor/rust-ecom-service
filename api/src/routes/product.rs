use crate::{
    priveleges::check_admin,
    request::NewProduct,
    response::{FilteredProduct, ProductGridImage},
    storage::{convert_image_to_webp, get_uploaded_images, upload_image},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use entity::{prelude::*, *};
use lemon_tree_core::{
    sea_orm::{ActiveValue, EntityTrait},
    AppState, Query as QueryCore,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

fn filter_product_record(product: &product::Model) -> FilteredProduct {
    FilteredProduct {
        id: product.id.to_string(),
        name: product.name.to_owned(),
        description: product.description.to_owned(),
        price: product.price.to_string().parse::<f32>().unwrap(),
        stock: product.stock.to_owned(),
    }
}

#[derive(Deserialize)]
pub struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

pub async fn all_products(
    state: State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(10);

    let (products, num_pages) = QueryCore::find_products_in_page(&state.db, page, posts_per_page)
        .await
        .expect("Cannot find products in page");

    let response = json!({"products": products.iter().map(|product_and_image| {
        let (product, product_image): &(entity::product::Model, std::option::Option<entity::product_image::Model>) = product_and_image;

        let mut image_hash: String = String::new();
        if let Some(image_data) = product_image {
            image_hash = image_data.id.to_string();
        }

        ProductGridImage { 
            id: product.id.to_string(),
            name: product.name.to_owned(),
            description: product.description.to_owned(),
            price: product.price.to_string().parse::<f32>().unwrap(),
            stock: product.stock.to_owned(),
            img: image_hash
        }
    }).collect::<Vec<ProductGridImage>>(), "pageCount": num_pages});

    Ok(Json(response))
}

pub async fn list_product(
    State(data): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("requesting product {}", product_id);

    let product = Product::find_by_id(Uuid::parse_str(&product_id).unwrap())
        .find_also_related(ProductImage)
        .one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if let Some(product) = product {
        let (product, product_image): (
            entity::product::Model,
            std::option::Option<entity::product_image::Model>,
        ) = product;

        let mut image_hash: String = String::new();
        if let Some(image_data) = product_image {
            image_hash = image_data.id.to_string();
        }

        let product = ProductGridImage {
            id: product.id.to_string(),
            name: product.name.to_owned(),
            description: product.description.to_owned(),
            price: product.price.to_string().parse::<f32>().unwrap(),
            stock: product.stock.to_owned(),
            img: image_hash,
        };

        let product_response = json!({ "status": "success", "data": product });

        Ok(Json(product_response))
    } else {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Unable to locate a product with that ID",
        });

        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }
}

pub async fn create_product(
    Extension(user): Extension<account::Model>,
    State(data): State<Arc<AppState>>,
    Json(req_product): Json<NewProduct>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let new_product = product::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::set(req_product.name),
        description: ActiveValue::set(req_product.description),
        price: ActiveValue::set(Decimal::from_f32_retain(req_product.price).unwrap()),
        stock: ActiveValue::set(req_product.stock),
    };

    let product = Product::insert(new_product)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let product_image_hash = req_product.image_id;

    let new_product_image = product_image::ActiveModel {
        id: ActiveValue::Set(Uuid::parse_str(&product_image_hash).unwrap()),
        product_id: ActiveValue::Set(product.id),
        position: ActiveValue::Set(1),
    };

    let _product_image = ProductImage::insert(new_product_image)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let product_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "product": filter_product_record(&product)
    })});

    Ok(Json(product_response))
}

pub async fn upload_product_image(
    Extension(user): Extension<account::Model>,
    mut files: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let mut file_locations: HashSet<String> = HashSet::new();

    while let Some(file) = files.next_field().await.unwrap() {
        let data = file.bytes().await.unwrap();
        let image = convert_image_to_webp(data.as_ref()).to_owned();

        let s3_resp = upload_image(image).await.unwrap();
        file_locations.insert(s3_resp);
    }

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "files": file_locations
    })});

    Ok(Json(user_response))
}

pub async fn list_uploaded_images(
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let file_locations = get_uploaded_images().await.unwrap();

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "images": file_locations
    })});

    Ok(Json(user_response))
}
