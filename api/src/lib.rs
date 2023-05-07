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
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{add_extension::AddExtensionLayer, cors::CorsLayer};
use uuid::Uuid;

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
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(SharedState::default()))
                .into_inner(),
        )
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

type SharedState = Arc<Mutex<State>>;

#[derive(Default)]
pub struct State {
    reset_tokens: ResetTokenDB,
}

#[derive(Default)]
pub struct ResetTokenDB {
    tokens: HashMap<String, Uuid>,
}

impl ResetTokenDB {
    fn add_token(&mut self, token: String, email: Uuid) {
        self.tokens.insert(token, email);
    }

    fn get_token(&self, token: &String) -> Option<&Uuid> {
        self.tokens.get(token)
    }

    fn remove_token(&mut self, token: &String) {
        self.tokens.remove(&token.to_owned());
    }
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
