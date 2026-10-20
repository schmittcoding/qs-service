use std::{collections::HashSet, sync::Arc, time::Duration};

use ran_db_core::webmanager::{email::suppression::WebEmailSuppressionTrait, WebDB};
use tokio::time::sleep;
use tracing::{error, info};

use crate::states::AppState;

const REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes

pub fn spawn_suppression_refresh(web_db: Arc<WebDB>, app_state: Arc<AppState>) {
    tokio::spawn(async move {
        info!(
            "Suppression cache refresh started — refreshing every {}s",
            REFRESH_INTERVAL_SECS
        );

        loop {
            sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;

            match web_db.get_all_suppressed_emails().await {
                Ok(emails) => {
                    let count = emails.len();
                    let set: HashSet<String> = emails.into_iter().collect();
                    let mut cache = app_state.suppression_cache.write().await;
                    *cache = set;
                    tracing::debug!("Suppression cache refreshed: {} addresses", count);
                }
                Err(e) => {
                    error!("Failed to refresh suppression cache: {:?}", e);
                }
            }
        }
    });
}
