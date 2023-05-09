use aws_sdk_s3::Client;
use aws_smithy_http::byte_stream::ByteStream;
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

const BUCKET_NAME: &str = "lemonseeds";

pub async fn upload_image(
    body: ByteStream,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let file_key = Uuid::new_v4().to_string();
    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(&file_key)
        .body(body)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("File upload error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    Ok(file_key)
}
