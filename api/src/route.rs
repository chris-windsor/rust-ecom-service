use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        all_products, create_product, get_me_handler, health_checker_handler, login_user_handler,
        logout_handler, register_user_handler,
    },
    jwt::auth,
    AppState,
};

pub fn create_auth_router(app_state: &Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
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
        .route("/products", get(all_products))
        .route("/product", post(create_product))
        .with_state(app_state.to_owned())
}
