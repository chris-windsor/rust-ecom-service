use crate::{
    priveleges::check_admin,
    request::{NewAttribute, NewCategory, NewProduct},
    response::{
        FilteredAttribute, FilteredAttributeOption, FilteredCategory, FilteredProduct,
        FilteredProductAttribute, FilteredSimpleProduct,
    },
    storage::{convert_image_to_webp, get_uploaded_images, upload_image},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use entity::{prelude::*, *};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_ecom_service_core::{
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter},
    AppState, Query as QueryCore,
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

fn filter_simple_product_record(
    product: &product::Model,
    product_image: Option<&product_image::Model>,
) -> FilteredSimpleProduct {
    FilteredSimpleProduct {
        short_url: product.short_url.to_string(),
        name: product.name.to_owned(),
        price: product.price.to_string().parse::<f32>().unwrap(),
        img: match product_image {
            Some(image) => image.id.to_string(),
            None => String::from("unknown"),
        },
    }
}

fn filter_product_record(
    product: &product::Model,
    product_image: Option<&product_image::Model>,
    product_categories: Option<Vec<&category::Model>>,
    product_attributes: Option<
        Vec<(
            entity::product_attribute::Model,
            Vec<entity::attribute::Model>,
        )>,
    >,
) -> FilteredProduct {
    FilteredProduct {
        id: product.shared_id.to_string(),
        short_url: product.short_url.to_string(),
        name: product.name.to_owned(),
        description: product.description.to_owned(),
        price: product.price.to_string().parse::<f32>().unwrap(),
        img: match product_image {
            Some(image) => image.id.to_string(),
            None => String::from("unknown"),
        },
        categories: match product_categories {
            Some(categories) => categories
                .iter()
                .map(|category| filter_category_record(category))
                .collect::<Vec<_>>(),
            None => vec![],
        },
        attributes: match product_attributes {
            Some(attributes) => attributes
                .iter()
                .map(|category| {
                    filter_product_attribute_record(
                        &category.0,
                        &category.1.get(0).as_ref().unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
            None => vec![],
        },
    }
}

fn filter_attribute_option_record(
    attribute_option: &attribute_option::Model,
) -> FilteredAttributeOption {
    FilteredAttributeOption {
        id: attribute_option.id,
        label: attribute_option.label.to_string(),
        content: attribute_option.content.to_string(),
        attribute_id: attribute_option.attribute_id,
    }
}

fn filter_attribute_record(
    attribute: &attribute::Model,
    attribute_options: Vec<attribute_option::Model>,
) -> FilteredAttribute {
    FilteredAttribute {
        id: attribute.id,
        kind: attribute.kind.to_string(),
        label: attribute.label.to_string(),
        options: attribute_options
            .iter()
            .map(|opt| filter_attribute_option_record(opt))
            .collect(),
    }
}

fn filter_category_record(category: &category::Model) -> FilteredCategory {
    FilteredCategory {
        id: category.id,
        label: category.label.to_string(),
        parent_id: category.parent_id.unwrap_or(-1),
    }
}

fn filter_product_attribute_record(
    product_attribute: &product_attribute::Model,
    attribute: &attribute::Model,
) -> FilteredProductAttribute {
    FilteredProductAttribute {
        id: attribute.id,
        kind: attribute.kind.to_string(),
        label: attribute.label.to_string(),
        content: product_attribute.content.to_string(),
    }
}

#[derive(Deserialize)]
pub struct ProductRetrievalParams {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

pub async fn all_products(
    state: State<Arc<AppState>>,
    Query(params): Query<ProductRetrievalParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(10);

    let (products, num_pages) = QueryCore::find_products_in_page(&state.db, page, posts_per_page)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let response = json!({"products": products.iter().map(|product_and_image| {
        let (product, product_image): &(entity::product::Model, std::option::Option<entity::product_image::Model>) = product_and_image;
        filter_simple_product_record(&product, product_image.as_ref())
    }).collect::<Vec<FilteredSimpleProduct>>(), "pageCount": num_pages});

    Ok(Json(response))
}

pub async fn list_product(
    State(data): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let product = Product::find()
        .filter(product::Column::ShortUrl.eq(product_id))
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

    if let Some(product) = &product {
        let (product, product_image): &(product::Model, Option<product_image::Model>) = product;

        let product_categories = ProductCategory::find()
            .filter(product_category::Column::ProductId.eq(product.id))
            .find_also_related(Category)
            .all(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        let filtered_categories = product_categories
            .iter()
            .map(|category| category.1.as_ref().unwrap())
            .collect::<Vec<_>>();

        let product_attributes = ProductAttribute::find()
            .filter(product_attribute::Column::ProductId.eq(product.id))
            .find_with_related(Attribute)
            .all(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        let product = filter_product_record(
            &product,
            product_image.as_ref(),
            Some(filtered_categories),
            Some(product_attributes),
        );

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
        id: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(req_product.name),
        description: ActiveValue::set(req_product.description),
        price: ActiveValue::set(Decimal::from_f32(req_product.price).unwrap()),
        short_url: ActiveValue::set(req_product.short_url),
        shared_id: ActiveValue::set(Uuid::new_v4()),
        active_revision: ActiveValue::set(true),
        ..Default::default()
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

    let product_image = ProductImage::insert(new_product_image)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    for category in req_product.categories.into_iter() {
        let new_product_category = product_category::ActiveModel {
            id: ActiveValue::NotSet,
            product_id: ActiveValue::set(product.id),
            category_id: ActiveValue::set(i32::from_u32(category).unwrap()),
        };

        ProductCategory::insert(new_product_category)
            .exec_with_returning(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    }

    for attribute in req_product.attributes.into_iter() {
        let new_product_attribute = product_attribute::ActiveModel {
            id: ActiveValue::NotSet,
            product_id: ActiveValue::set(product.id),
            attribute_id: ActiveValue::set(i32::from_u32(attribute).unwrap()),
            content: ActiveValue::set(String::from("")),
        };

        ProductAttribute::insert(new_product_attribute)
            .exec_with_returning(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    }

    let product_response = serde_json::json!({"status": "success","data": serde_json::json!({
        // TODO: return back categories and attributes
        "product": filter_product_record(&product, Some(&product_image), None, None)
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

pub async fn list_attributes(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_attributes = Attribute::find()
        .find_with_related(AttributeOption)
        .all(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let attributes_response = serde_json::json!({"attributes": all_attributes.iter().map(|attribute| filter_attribute_record(&attribute.0, attribute.1.to_owned())).collect::<Vec<_>>()});

    Ok(Json(attributes_response))
}

pub async fn retrieve_attribute(
    State(data): State<Arc<AppState>>,
    Path(attribute_id): Path<String>,
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    Ok(())
}

pub async fn create_attribute(
    Extension(user): Extension<account::Model>,
    State(data): State<Arc<AppState>>,
    Json(req_attribute): Json<NewAttribute>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let new_attribute = attribute::ActiveModel {
        id: ActiveValue::NotSet,
        kind: ActiveValue::set(req_attribute.kind),
        label: ActiveValue::set(req_attribute.label),
    };

    let attribute = Attribute::insert(new_attribute)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let mut attribute_options = vec![];

    for attribute_option in req_attribute.options {
        let new_attribute_option = attribute_option::ActiveModel {
            id: ActiveValue::NotSet,
            attribute_id: ActiveValue::Set(attribute.id),
            label: ActiveValue::Set(attribute_option.label),
            content: ActiveValue::Set(attribute_option.content),
        };

        let attribute_option = AttributeOption::insert(new_attribute_option)
            .exec_with_returning(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

        attribute_options.push(attribute_option);
    }

    let attribute_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "attribute": filter_attribute_record(&attribute, attribute_options)
    })});

    Ok(Json(attribute_response))
}

pub async fn update_attribute(
    State(data): State<Arc<AppState>>,
    Path(attribute_id): Path<String>,
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    Ok(())
}

pub async fn list_categories(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_categories = Category::find().all(&data.db).await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let categories_response = serde_json::json!({"categories": all_categories.iter().map(|category| filter_category_record(category)).collect::<Vec<_>>()});

    Ok(Json(categories_response))
}

pub async fn retrieve_category(
    State(data): State<Arc<AppState>>,
    Path(category_id): Path<String>,
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    Ok(())
}

pub async fn create_category(
    Extension(user): Extension<account::Model>,
    State(data): State<Arc<AppState>>,
    Json(req_category): Json<NewCategory>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let new_category = category::ActiveModel {
        id: ActiveValue::NotSet,
        label: ActiveValue::set(req_category.label),
        parent_id: ActiveValue::set(req_category.parent_id),
    };

    let category = Category::insert(new_category)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let category_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "category": filter_category_record(&category)
    })});

    Ok(Json(category_response))
}

pub async fn update_category(
    State(data): State<Arc<AppState>>,
    Path(category_id): Path<String>,
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    Ok(())
}
