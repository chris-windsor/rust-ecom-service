use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FilteredSimpleProduct {
    pub short_url: String,
    pub name: String,
    pub price: f32,
    pub img: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FilteredProduct {
    pub id: String,
    pub short_url: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub img: String,
    pub categories: Vec<FilteredCategory>,
    pub attributes: Vec<FilteredProductAttribute>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FilteredAttributeOption {
    pub id: i32,
    pub label: String,
    pub content: String,
    pub attribute_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilteredAttribute {
    pub id: i32,
    pub kind: String,
    pub label: String,
    pub options: Vec<FilteredAttributeOption>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FilteredCategory {
    pub id: i32,
    pub label: String,
    pub parent_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilteredProductAttribute {
    pub id: i32,
    pub kind: String,
    pub label: String,
    pub content: String,
}
