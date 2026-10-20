use aws_sdk_s3::{Client, primitives::ByteStream};
use sha2::{Digest, Sha256};

pub struct R2Config {
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub account_id: String,
}

pub async fn upload_image_to_r2(
    client: &Client,
    cfg: &R2Config,
    path_name: &str,
    image_bytes: Vec<u8>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Compute SHA-256 for dedupe
    let mut hasher = Sha256::new();
    hasher.update(&image_bytes);
    let hash = format!("{:x}", hasher.finalize());

    let object_key = format!("{}/{}.png", path_name, hash);

    // Check if object exists
    let head_result = client
        .head_object()
        .bucket(&cfg.bucket)
        .key(&object_key)
        .send()
        .await;

    if head_result.is_ok() {
        let url = format!(
            "{}/{}",
            std::env::var("R2_IMAGE_DOMAIN").unwrap(), object_key
        );
        return Ok(url);
    }

    client
        .put_object()
        .bucket(&cfg.bucket)
        .key(&object_key)
        .body(ByteStream::from(image_bytes))
        .content_type("image/png")
        .send()
        .await?;

    let url = format!(
        "{}/{}",
        std::env::var("R2_IMAGE_DOMAIN").unwrap(), object_key
    );

    Ok(url)
}
