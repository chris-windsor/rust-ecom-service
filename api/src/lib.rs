mod email;
mod jwt;
mod model;
mod priveleges;
mod request;
mod response;
mod route;
mod routes;
mod storage;

use axum::Server;
use dotenvy::dotenv;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use route::{
    create_auth_router, create_content_router, create_order_router, create_product_router,
};
use rust_ecom_service_core::{
    payment_processing::manager::get_payment_processor,
    sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr},
    AppState, Config,
};
use rust_ecom_service_plugins::load_plugin_routers;
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{add_extension::AddExtensionLayer, cors::CorsLayer};

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Config::init();

    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    let conn = match establish_db_conection(&config).await {
        Ok(connection) => connection,
        Err(error) => panic!("Error encountered while connecting to DB: {:?}", error),
    };

    let cors = CorsLayer::new()
        .allow_origin(
            format!("http://{}", config.web_host)
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app_state = Arc::new(AppState {
        db: conn.clone(),
        env: config.clone(),
        message_channel: Default::default(),
        payment_processor: get_payment_processor(),
    });

    let plugin_routers = load_plugin_routers(&app_state);

    let mut app = create_auth_router(&app_state)
        .merge(create_product_router(&app_state))
        .merge(create_order_router(&app_state))
        .merge(create_content_router(&app_state));

    // TODO: improve builder of app
    for router in plugin_routers {
        app = app.clone().merge(router);
    }

    app = app
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(SharedState::default()))
                .into_inner(),
        )
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn establish_db_conection(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let database_url = config.database_url.to_owned();
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));

    let db = Database::connect(opt).await?;
    Ok(db)
}

type SharedState = Arc<Mutex<State>>;

#[derive(Default)]
pub struct State {
    reset_tokens: ResetTokenDB,
    reset_requests: ResetRequestDB,
}

#[derive(Default)]
pub struct ResetTokenDB {
    tokens: HashMap<String, i32>,
}

impl ResetTokenDB {
    fn add_token(&mut self, token: String, user_id: i32) {
        self.tokens.insert(token, user_id);
    }

    fn get_token(&self, token: &String) -> Option<i32> {
        self.tokens.get(token).copied()
    }

    fn remove_token(&mut self, token: &String) {
        self.tokens.remove(&token.to_owned());
    }
}

#[derive(Default)]
pub struct ResetRequestDB {
    requests: HashMap<String, usize>,
}

impl ResetRequestDB {
    fn add_request(&mut self, email: String, time: usize) {
        self.requests.insert(email, time);
    }

    fn get_request(&self, email: &String) -> Option<&usize> {
        self.requests.get(email)
    }
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
