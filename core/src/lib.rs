mod config;
mod mutation;
mod query;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub use config::Config;
pub use mutation::*;
pub use query::*;

pub use sea_orm;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub env: Config,
    pub message_channel: Arc<Mutex<VecDeque<Arc<str>>>>,
}
