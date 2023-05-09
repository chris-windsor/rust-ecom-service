use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: f32,
    pub stock: i32,
    pub image_id: String,
}
