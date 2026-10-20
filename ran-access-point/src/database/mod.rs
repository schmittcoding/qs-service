use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use ran_db_core::{
    gamemanager::GameDB, logmanager::LogDB, shopmanager::ShopDB, usermanager::UserDB,
    webmanager::WebDB, DBManager, DatabaseTrait,
};
use std::{env, sync::Arc};
use tracing::info;

use crate::{routes::utils::R2Config, services::r2::build_r2_client};

#[derive(Clone)]
pub struct DBState {
    pub user_db: Arc<UserDB>,
    pub web_db: Arc<WebDB>,
    pub log_db: Arc<LogDB>,
    pub game_db: Arc<GameDB>,
    pub shop_db: Arc<ShopDB>,
    pub r2_client: Client,
    pub r2_config: Arc<R2Config>,
}

impl DBState {
    pub async fn initialize() -> Result<Arc<DBState>> {
        let db_manager = Arc::new(
            DBManager::new()
                .await
                .context("Failed to initialize DBManager")?,
        );

        let user_db = Arc::new(
            UserDB::new(&db_manager)
                .await
                .context("Failed to initialize UserDB")?,
        );
        let web_db = Arc::new(
            WebDB::new(&db_manager)
                .await
                .context("Failed to initialize WebDB")?,
        );
        let log_db = Arc::new(
            LogDB::new(&db_manager)
                .await
                .context("Failed to initialize LogDB")?,
        );
        let game_db = Arc::new(
            GameDB::new(&db_manager)
                .await
                .context("Failed to initialize GameDB")?,
        );
        let shop_db = Arc::new(
            ShopDB::new(&db_manager)
                .await
                .context("Failed to initialize ShopDB")?,
        );

        let r2_config = Arc::new(R2Config {
            bucket: env::var("R2_BUCKET").context("R2_BUCKET must be set").unwrap(),
            access_key: env::var("R2_ACCESS_KEY").context("R2_ACCESS_KEY must be set").unwrap(),
            secret_key: env::var("R2_SECRET_KEY").context("R2_SECRET_KEY must be set").unwrap(),
            account_id: env::var("R2_ACCOUNT_ID").context("R2_ACCOUNT_ID must be set").unwrap(),
        });

        let r2_client = build_r2_client(&r2_config).await;

        info!("✅ All databases initialized successfully.");

        Ok(Arc::new(DBState {
            user_db,
            web_db,
            log_db,
            game_db,
            shop_db,
            r2_client,
            r2_config,
        }))
    }

    pub fn cleanup(self) {
        info!("🛑 Closing database connections...");

        macro_rules! try_close_db {
            ($db:expr, $name:expr) => {
                match Arc::try_unwrap($db) {
                    Ok(db) => db.pool.close(),
                    Err(_) => tracing::error!("⚠️ Could not drop {}, still in use", $name),
                }
            };
        }

        try_close_db!(self.user_db, "user_db");
        try_close_db!(self.web_db, "web_db");
        try_close_db!(self.log_db, "log_db");
        try_close_db!(self.game_db, "game_db");
        try_close_db!(self.shop_db, "shop_db");

        info!("✅ All database connections closed.");
    }
}
