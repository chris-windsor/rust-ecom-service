use crate::{
    email::send_password_reset_email,
    model::{
        InquirePasswordResetSchema, LoginUserSchema, RegisterUserSchema, ResetPasswordSchema,
        TokenClaims,
    },
    priveleges::check_admin,
    request::NewProduct,
    response::{FilteredProduct, FilteredUser, ProductGridImage},
    storage::upload_image,
    SharedState,
};
use argon2::{
    password_hash::{rand_core, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use entity::{prelude::*, *};
use jsonwebtoken::{encode, EncodingKey, Header};
use lemon_tree_core::{
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter},
    AppState, Query as QueryCore,
};
use rand_core::OsRng;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

fn filter_user_record(user: &account::Model) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        role: user.role.to_owned(),
    }
}

fn filter_product_record(product: &product::Model) -> FilteredProduct {
    FilteredProduct {
        id: product.id.to_string(),
        name: product.name.to_owned(),
        description: product.description.to_owned(),
        price: product.price.to_string().parse::<f32>().unwrap(),
        stock: product.stock.to_owned(),
    }
}

const PASSWWORD_RESET_REQUEST_DELAY: usize = 3600;

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_exists = Account::find()
        .filter(account::Column::Email.eq(body.email.to_owned().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if let Some(_exists) = user_exists {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "User with that email already exists",
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let new_user = account::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::Set(body.name.to_string()),
        email: ActiveValue::Set(body.email.to_string().to_ascii_lowercase()),
        password: ActiveValue::Set(hashed_password),
        ..Default::default()
    };

    let user = Account::insert(new_user)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "user": filter_user_record(&user)
    })});

    Ok(Json(user_response))
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = Account::find()
        .filter(account::Column::Email.eq(body.email.to_owned().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Invalid email or password",
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::seconds(data.env.jwt_expiry)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
        name: user.name,
        role: user.role,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(time::Duration::seconds(data.env.jwt_expiry))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let mut response = Response::new(json!({"status": "success", "token": token}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn logout_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn get_me_handler(
    Extension(user): Extension<account::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "user": filter_user_record(&user)
        })
    });

    Ok(Json(json_response))
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "JWT Authentication in Rust using Axum, Postgres, and SQLX";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn inquire_password_reset_handler(
    Extension(state): Extension<SharedState>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<InquirePasswordResetSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_exists = Account::find()
        .filter(account::Column::Email.eq(body.email.to_owned().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if let None = user_exists {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "User with that email could not be found",
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let user = user_exists.unwrap();
    let mut db_state = state.lock().await;
    let now = chrono::Utc::now();
    let now = now.timestamp() as usize;

    if let Some(req_time) = db_state.reset_requests.get_request(&user.email) {
        if (req_time + PASSWWORD_RESET_REQUEST_DELAY) > now {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "A previous reset request has occured too recently",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
    }

    let reset_token = Uuid::new_v4().to_string();
    db_state
        .reset_tokens
        .add_token(reset_token.clone(), user.id);
    db_state.reset_requests.add_request(user.email.clone(), now);

    let reset_url = format!(
        "http://{}/auth/resetpassword?token={}",
        &data.env.web_host, reset_token
    );
    let reset_email_content = format!("<a href='{}'>Reset your password</a>", reset_url);
    send_password_reset_email(&user.email, &reset_email_content).await;

    let json_response = serde_json::json!({
        "status":  "success",
        "data": {
            "message": "Check your email for a link to reset your password",
        }
    });

    Ok(Json(json_response))
}

pub async fn change_password_handler(
    Extension(state): Extension<SharedState>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<ResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut other_state = state.lock().await;
    let user_id: Uuid;

    if let Some(value) = other_state.reset_tokens.get_token(&body.token) {
        user_id = value.clone();
    } else {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid reset token provided",
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let updated_account = account::ActiveModel {
        id: ActiveValue::Set(user_id),
        password: ActiveValue::Set(hashed_password),
        ..Default::default()
    };

    let _ = updated_account.update(&data.db).await.map_err(|e| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    });

    other_state.reset_tokens.remove_token(&body.token);

    let json_response = serde_json::json!({
        "status":  "success",
        "data": {
            "message": "Password has been successfully changed",
        }
    });

    Ok(Json(json_response))
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

        let product_response = json!({ "product": product });

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

        let s3_resp = upload_image(data.into()).await.unwrap();

        file_locations.insert(s3_resp);
    }

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "files": file_locations
    })});

    Ok(Json(user_response))
}
