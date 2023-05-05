mod config;
mod handler;
mod jwt;
mod model;
mod response;
mod route;

use axum::Server;
use config::Config;
use dotenvy::dotenv;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use lemon_tree_core::sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};
use route::{create_auth_router, create_product_router};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Config::init();

    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url =
        env::var("QUALIFIED_DATABASE_URL").expect("QUALIFIED_DATABASE_URL is not set in .env file");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app_state = Arc::new(AppState {
        db: conn.clone(),
        env: config.clone(),
    });

    let app = create_auth_router(&app_state)
        .merge(create_product_router(&app_state))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4567));
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    env: Config,
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
