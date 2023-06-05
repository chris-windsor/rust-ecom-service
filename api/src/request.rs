use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NewProduct {
    pub name: String,
    pub short_url: String,
    pub description: String,
    pub price: f32,
    pub categories: Vec<u32>,
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
