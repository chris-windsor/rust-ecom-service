use std::sync::Arc;

use argon2::{
    password_hash::{rand_core, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Query, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use entity::{prelude::*, *};
use jsonwebtoken::{encode, EncodingKey, Header};
use lemon_tree_core::{
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter},
    Query as QueryCore,
};
use rand_core::OsRng;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    model::{LoginUserSchema, RegisterUserSchema, TokenClaims},
    response::FilteredProduct,
    response::FilteredUser,
    AppState,
};

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
        price: product.price.to_owned(),
        stock: product.stock.to_owned(),
    }
}

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

    println!("{:?}", user);

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
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(time::Duration::hours(1))
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

    let response = json!({"products": products, "pageCount": num_pages});

    Ok(Json(response))
}

pub async fn create_product(
    State(data): State<Arc<AppState>>,
    Json(new_product): Json<FilteredProduct>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let new_product = product::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::set(new_product.name),
        description: ActiveValue::set(new_product.description),
        price: ActiveValue::set(new_product.price),
        stock: ActiveValue::set(new_product.stock),
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

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "product": filter_product_record(&product)
    })});

    Ok(Json(user_response))
}
