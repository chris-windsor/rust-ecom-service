mod config;
mod mutation;
mod query;

pub use config::Config;
pub use mutation::*;
pub use query::*;

pub use sea_orm;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub env: Config,
}
