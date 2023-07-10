mod config;
pub mod ecommerce;
mod mutation;
pub mod payment_processing;
mod query;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub use config::Config;
pub use mutation::*;
use payment_processing::manager::PaymentProcessor;
pub use query::*;

pub use sea_orm;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub env: Config,
    pub message_channel: Arc<Mutex<VecDeque<Arc<str>>>>,
    pub payment_processor: Arc<dyn PaymentProcessor>,
}
