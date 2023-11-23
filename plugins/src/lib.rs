use axum::Router;
use rust_ecom_service_core::AppState;
use std::sync::Arc;

// Pull in other modules from the plugins directory

pub fn load_plugin_routers(app_state: &Arc<AppState>) -> Vec<Router> {
    // Return plugin routers here
    // Example:
    // vec![store_search::plugin_router(app_state)]
    vec![]
}
