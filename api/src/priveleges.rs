use axum::{middleware::Next, response::IntoResponse, Extension, Json};
use entity::account;
use http::{Request, StatusCode};

use crate::jwt::ErrorResponse;

// TODO: Would be great to figure out how to chain the 'route_layer's and use this
pub async fn admin_auth<B>(
    Extension(user): Extension<account::Model>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if user.role != "admin" {
        let json_error = ErrorResponse {
            status: "Authentication Error",
            message: "Insufficient priveleges to complete the request".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(json_error)));
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// TODO: ditch this in favor of the methodology
pub fn check_admin(user: &account::Model) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Insufficient privileges to complete the request",
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    } else {
        Ok(())
    }
}
