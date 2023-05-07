use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use lemon_tree_core::sea_orm;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{model::TokenClaims, AppState};
use entity::{prelude::*, *};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

const BEARER_PREFIX: &str = "Bearer ";

pub async fn auth<B>(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with(BEARER_PREFIX) {
                        Some(auth_value[BEARER_PREFIX.len()..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Request does not contain an Authorization Bearer".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Request provided an invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?
    .claims;

    // TODO: I did not want to write this and I hope to remove it
    // The block above should be returning the errors from the validation
    // but its not...
    let now = chrono::Utc::now();
    let now = now.timestamp() as usize;
    if claims.exp < now {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Request provided an expired token".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(json_error)));
    }

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Request provided an invalid tokenn".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let user = Account::find()
        .filter(account::Column::Id.eq(user_id))
        .one(&data.db)
        .await
        .map_err(|e| {
            println!("Unable to locate user for authentication process: {}", e);
            let json_error = ErrorResponse {
                status: "Authentication Error",
                message: "Error encountered while retrieving user from database".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let user = user.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Unable to locate the user that owns the provided token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
