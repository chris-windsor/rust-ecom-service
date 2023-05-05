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
    pub price: rust_decimal::Decimal,
    pub stock: i32,
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub user: FilteredUser,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData,
}
