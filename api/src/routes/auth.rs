use crate::{
    email::send_password_reset_email,
    model::{
        InquirePasswordResetSchema, LoginUserSchema, RegisterUserSchema, ResetPasswordSchema,
        TokenClaims,
    },
    response::FilteredUser,
    SharedState,
};
use argon2::{
    password_hash::{rand_core, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::State,
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use entity::{prelude::*, *};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use rust_ecom_service_core::{
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter},
    AppState,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

fn filter_user_record(user: &account::Model) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        role: user.role.to_owned(),
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
        role: user.role.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let response =
        Response::new(json!({"status": "success", "token": token, "role": user.role}).to_string());
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
