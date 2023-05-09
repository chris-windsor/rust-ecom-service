use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilteredProduct {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub stock: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductGridImage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub stock: i32,
    pub img: String,
}
