use std::sync::Arc;

use actix_web::web;
use ran_db_core::{
    webmanager::{download::WebDownloadTrait, email::suppression::WebEmailSuppressionTrait},
};
use uuid::Uuid;

use crate::{database::DBState, states::AppState};
pub mod cache;
pub mod discord;
pub mod jobs;
pub mod metrics;
pub mod smtp;
pub mod ssl;
pub mod jwt;
pub mod r2;
pub mod rate_limit;
pub mod verification;

pub fn generate_reset_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn initialize_cache(states: web::Data<Arc<AppState>>, db: web::Data<Arc<DBState>>) {
    // initialize_shop_data(states.clone(), db.clone()).await;
    initialize_download_links(states.clone(), db.clone()).await;
    initialize_suppression_cache(states.clone(), db.clone()).await;
}

async fn initialize_download_links(states: web::Data<Arc<AppState>>, db: web::Data<Arc<DBState>>) {
    let download_cache = states.download_links_cache.lock().await;

    let mut data_cache = download_cache.data.lock().await;

    if data_cache.is_none() {
        tracing::info!("Loading download links from database...");
        match db.web_db.get_download_links(Some(1), None).await {
            Ok(initial_data) => {
                *data_cache = Some(initial_data);
                drop(data_cache);
                tracing::info!("Download items cache has been initialized");
            }
            Err(err) => {
                tracing::error!("Failed to load download links: {:?}", err);
            }
        }
    }
}

async fn initialize_suppression_cache(states: web::Data<Arc<AppState>>, db: web::Data<Arc<DBState>>) {
    tracing::info!("Loading email suppression cache...");
    match db.web_db.get_all_suppressed_emails().await {
        Ok(emails) => {
            let count = emails.len();
            let set: std::collections::HashSet<String> = emails.into_iter().collect();
            let mut cache = states.suppression_cache.write().await;
            *cache = set;
            tracing::info!("Email suppression cache loaded: {} addresses", count);
        }
        Err(err) => {
            tracing::error!("Failed to load email suppression cache: {:?}", err);
        }
    }
}

pub struct DiscordWebhookMsg {
    pub title: String,
    pub description: String,
}

// async fn discord_webhook(content: serde_json::Value) {
//     tokio::spawn(async move {
//         let webhook_url = "https://discord.com/api/webhooks/1335276138272129106/WrVzOx6dG0rhdb7TRvvBsp7A9My7HEXJlltWv-rKpDfc3qgQt0McZ6nBluamHwXeplc3";
//         let client = reqwest::Client::new();
//         let embed = json!({
//             "title": "Approve/Denied Notification",
//             "description": format!(
//                 "Transaction  {} has been processed",
//                 trans_id
//             ),
//             "color": 0x444444,
//             "timestamp": chrono::Utc::now().to_rfc3339(),
//             "footer": {
//                 "text": "RAN Online GS"
//             }
//         });
//         let payload = json!({
//             "username": "RAN Online GS",
//             "embeds": [ content ]
//         });

//         if let Err(err) = client.post(webhook_url).json(&payload).send().await {
//             tracing::error!("Error sending Discord webhook: {:?}", err);
//         }
//     });
// }
