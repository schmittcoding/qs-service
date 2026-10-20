use aws_config::Region;
use aws_sdk_s3::{Client, config::Credentials};

use crate::routes::utils::R2Config;

pub async fn build_r2_client(cfg: &R2Config) -> Client {
    let region = Region::new("auto");

    let shared_config = aws_sdk_s3::config::Builder::new()
        .region(region)
        .credentials_provider(
            Credentials::new(&cfg.access_key, &cfg.secret_key, None, None, "r2")
        )
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", cfg.account_id))
        .build();

    Client::from_conf(shared_config)
}
