use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NewProduct {
    pub name: String,
    pub short_url: String,
    pub description: String,
    pub price: f32,
    pub categories: Vec<u32>,
    pub attributes: Vec<u32>,
    pub stock: i32,
    pub image_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewAttributeOption {
    pub label: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewAttribute {
    pub label: String,
    pub kind: String,
    pub options: Vec<NewAttributeOption>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NewCategory {
    pub label: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerDetails {
    pub email_address: Arc<str>,
    pub first_name: Arc<str>,
    pub last_name: Arc<str>,
    pub street_address_1: Arc<str>,
    pub street_address_2: Arc<str>,
    pub city: Arc<str>,
    pub state: Arc<str>,
    pub zip_code: Arc<str>,
    pub phone_number: Arc<str>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentDetails {
    pub credit_card_number: Arc<str>,
    pub credit_card_expiry: Arc<str>,
    pub credit_card_cvv: Arc<str>,
    pub credit_card_name: Arc<str>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]

pub struct OrderItem {
    pub id: Arc<str>,
    pub qty: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NewOrder {
    pub customer_details: CustomerDetails,
    pub payment_details: PaymentDetails,
    pub order_items: Vec<OrderItem>,
}
