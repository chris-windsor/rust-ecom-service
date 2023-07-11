use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use entity::{prelude::*, *};
use http::StatusCode;
use lemon_tree_core::{
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    AppState,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct StaticPageQuery {
    slug: Arc<str>,
}

pub async fn content_page(
    Query(query): Query<StaticPageQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let page = StaticPage::find()
        .filter(static_page::Column::Slug.like(&query.slug))
        .one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    match page {
        Some(page) => Ok(Json(json!({
            "content": page.content
        }))),
        None => Ok(Json(json!({
            "content": "nope".to_string()
        }))),
    }
}
