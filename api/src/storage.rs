use aws_sdk_s3::Client;
use aws_smithy_http::byte_stream::ByteStream;
use axum::Json;
use http::StatusCode;
use image::{imageops, DynamicImage, GenericImageView};
use uuid::Uuid;
use webp::{Encoder, WebPMemory};

const BUCKET_NAME: &str = "lemonseeds";

pub fn convert_image_to_webp(bytes: &[u8]) -> WebPMemory {
    let img = image::load_from_memory(bytes).unwrap();
    let (w, h) = img.dimensions();
    let size_factor = 1.0;
    let img: DynamicImage = image::DynamicImage::ImageRgba8(imageops::resize(
        &img,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));
    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    let webp: WebPMemory = encoder.encode(50f32);
    webp
}

pub async fn upload_image(image: Vec<u8>) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let file_key = Uuid::new_v4().to_string();
    let image_bytestream = ByteStream::from(image);
    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(&file_key)
        .body(image_bytestream)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Image upload error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    Ok(file_key)
}
