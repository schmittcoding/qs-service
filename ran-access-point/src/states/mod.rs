use actix_web::web;
use anyhow::Context;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use paypal_rs::{Client, PaypalEnv};
use ran_db_core::gamemanager::CommonRankingData;
use ran_models::{
    download::DownloadItem, email::CampaignProgressEvent,
    events::level_cap_race_score::LevelCapRaceScore, news::NewsItem,
    payments::WebTransaction, shop::ShopDataResponse,
};
use std::{collections::{HashMap, HashSet}, sync::Arc, time::Duration};
use tokio::{sync::{broadcast, Mutex, RwLock}, time::Instant};
use uuid::Uuid;

use crate::services::{cache::{LevelRankingCache, RankingCache, SimpleVecCache}, smtp};

#[derive(Debug, Clone)]
pub struct CachedLevelCapRaceLeaderboard {
    pub scores: Vec<LevelCapRaceScore>,
    pub total_count: i64,
    pub last_fetched_at: Instant,
}

#[derive(Debug)]
pub struct AppState {
    pub news_data: Mutex<HashMap<i32, Vec<NewsItem>>>,  // Every Update of Admin (HOLD THE CACHE)
    pub shop_data: Mutex<HashMap<i32, Vec<ShopDataResponse>>>,
    pub download_data: Mutex<HashMap<i32, Vec<DownloadItem>>>, // pub shop_data : Mutex<HashMap<i32, ShopRank>>, // Every Update of Admin (HOLD THE CACHE)
    // String - UserNum + Pagination | Logs
    pub ranking_data: Mutex<HashMap<String, Vec<CommonRankingData>>>,
    pub gold_ranking_cache: Mutex<RankingCache>,
    pub level_ranking_cache: Mutex<LevelRankingCache>,
    pub transaction_cache: Mutex<HashMap<String, CachedTransactions>>, // i64 - UserNum | Logs
    pub paypal_client: Mutex<paypal_rs::Client>,

    pub download_links_cache: Mutex<SimpleVecCache<DownloadItem>>,
    pub online_players_cache: Mutex<SimpleVecCache<String>>,
    pub level_cap_race_leaderboard_cache: Mutex<HashMap<Uuid, CachedLevelCapRaceLeaderboard>>,
    pub campaign_progress_tx: broadcast::Sender<CampaignProgressEvent>,
    pub smtp_transport: AsyncSmtpTransport<Tokio1Executor>,
    pub smtp_from_email: String,
    pub suppression_cache: Arc<RwLock<HashSet<String>>>,
}

impl AppState {
    pub async fn new() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();
        let clientid = std::env::var("PAYPAL_CLIENT_ID")
            .context("PAYPAL_CLIENT_ID must be set")
            .unwrap();
        let secret = std::env::var("PAYPAL_CLIENT_SECRET")
            .context("PAYPAL_CLIENT_SECRET must be set")
            .unwrap();

        let env = if cfg!(debug_assertions) {
            PaypalEnv::Sandbox
        } else {
            PaypalEnv::Live
        };
        let client = Client::new(clientid, secret, env);

        let smtp_config = smtp::SmtpConfig::from_env()?;
        let smtp_from_email = smtp_config.smtp_email.clone();
        let smtp_transport = smtp::build_smtp_transport(&smtp_config)?;

        let state = AppState {
            news_data: Mutex::new(HashMap::new()),
            shop_data: Mutex::new(HashMap::new()),
            download_data: Mutex::new(HashMap::new()),
            paypal_client: Mutex::new(client),
            ranking_data: Mutex::new(HashMap::new()),
            gold_ranking_cache: Mutex::new(RankingCache::new(Duration::from_secs(3 * 60 * 60))),
            level_ranking_cache: Mutex::new(LevelRankingCache::new(Duration::from_secs(30 * 60))),
            transaction_cache: Mutex::new(HashMap::new()),

            download_links_cache: Mutex::new(SimpleVecCache::new(Duration::from_secs(0))),
            online_players_cache: Mutex::new(
                SimpleVecCache::new(Duration::from_secs(600))
            ),
            level_cap_race_leaderboard_cache: Mutex::new(HashMap::new()),
            campaign_progress_tx: broadcast::channel(32).0,
            smtp_transport,
            smtp_from_email,
            suppression_cache: Arc::new(RwLock::new(HashSet::new())),
        };

        // Pre-fetch the PayPal token and handle errors gracefully
        // let token = state.get_paypal_token().await;
        // match token {
        //     Ok(_) => info!("Successfully fetched PayPal token"),
        //     Err(err) => {
        //         log::error!("Failed to fetch PayPal token: {:?}", err);
        //         return Err(anyhow::anyhow!("PayPal token fetch failed").into());
        //     }
        // }

        Ok(state)
    }

    pub async fn get_paypal_token(&self) -> Result<(), paypal_rs::errors::ResponseError> {
        let mut client = self.paypal_client.lock().await;
        client.get_access_token().await
    }
}

#[derive(Debug)]
pub struct CachedTransactions {
    pub last_updated: Instant,
    pub transactions: Vec<WebTransaction>,
}

pub async fn get_shop_cache(states: web::Data<Arc<AppState>>) -> Option<Vec<ShopDataResponse>> {
    let cache = states.shop_data.lock().await;
    cache.get(&0).cloned() // Clone to avoid holding the lock
}
