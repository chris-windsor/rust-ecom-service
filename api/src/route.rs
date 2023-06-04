use crate::{
    jwt::auth,
    routes::{
        auth::{
            change_password_handler, get_me_handler, inquire_password_reset_handler,
            login_user_handler, logout_handler, register_user_handler,
        },
        product::{
            all_products, create_product, list_product, list_uploaded_images, upload_product_image,
        },
    },
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use lemon_tree_core::AppState;
use std::sync::Arc;

pub fn create_auth_router(app_state: &Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .route(
            "/api/auth/inquire_password_reset",
            post(inquire_password_reset_handler),
        )
        .route("/api/auth/change_password", post(change_password_handler))
        .route(
            "/api/auth/logout",
            get(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state.to_owned())
}

pub fn create_product_router(app_state: &Arc<AppState>) -> Router {
    Router::new()
        .route("/api/products", get(all_products))
        .route("/api/product/:product_id", get(list_product))
        .route(
            "/api/product",
            post(create_product)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/upload_file",
            post(upload_product_image)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/list_files",
            get(list_uploaded_images)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state.to_owned())
}
