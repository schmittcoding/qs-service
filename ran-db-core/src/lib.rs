pub mod errors;
pub mod utils;
pub mod game;
pub mod gamemanager;
pub mod logmanager;
pub mod metrics;
pub mod shopmanager;
pub mod usermanager;
pub mod webmanager;

use anyhow::{Context, Result};
use deadpool_tiberius::{Manager, Pool};
use ran_models::errors::DatabaseError;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tiberius::ToSql;
use tokio::sync::mpsc;
use tracing::info;

/// Initialize the metrics system and set up periodic metrics saving
pub async fn init_db_metrics(metrics_dir: Option<&str>) {
    // Initialize metrics
    metrics::init_metrics();

    // Set custom path if provided
    if let Some(dir) = metrics_dir {
        metrics::set_metrics_path(dir);
    }

    // Set up periodic metrics saving
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // Every 30 minutes
        loop {
            interval.tick().await;
            if let Err(e) = metrics::save_metrics_to_file("db_metrics") {
                tracing::error!("Failed to save metrics to file: {}", e);
            }
        }
    });
}

#[derive(Debug, Clone)]
pub struct DBConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug)]
pub struct QueryTask {
    pub db_name: String,
    pub query: String,
    pub params: Vec<String>, // Adjust for actual parameter types
}

#[derive(Clone)]
pub struct DBManager {
    pub databases: Arc<HashMap<String, Pool>>,
    pub query_tx: mpsc::Sender<QueryTask>, // Query queue sender
}

impl DBManager {
    pub async fn new() -> Result<Self> {
        let databases = Arc::new(Self::initialize_databases().await?);
        let (query_tx, query_rx) = mpsc::channel(100); // Queue size 100

        let db_clone = Arc::clone(&databases);
        tokio::spawn(async move {
            DBManager::query_worker(db_clone, query_rx).await;
        });

        // Start pool stats tracking
        let db_metrics_clone = Arc::clone(&databases);
        tokio::spawn(async move {
            DBManager::track_pool_metrics(db_metrics_clone).await;
        });

        Ok(Self {
            databases,
            query_tx,
        })
    }

    async fn query_worker(
        databases: Arc<HashMap<String, Pool>>,
        mut query_rx: mpsc::Receiver<QueryTask>,
    ) {
        while let Some(task) = query_rx.recv().await {
            if let Some(pool) = databases.get(&task.db_name) {
                let _ = Self::execute_query(pool, &task.query, &task.params).await;
            } else {
                eprintln!("Database '{}' not found", task.db_name);
            }
        }
    }

    pub async fn enqueue_query(
        &self,
        db_name: &str,
        query: String,
        params: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task = QueryTask {
            db_name: db_name.to_string(),
            query,
            params,
        };
        self.query_tx.send(task).await?;
        Ok(())
    }

    async fn execute_query(
        pool: &Pool,
        query: &str,
        params: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = pool.get().await?;

        // Convert params from String to &dyn ToSql
        let params: Vec<&dyn ToSql> = params.iter().map(|p| p as &dyn ToSql).collect();

        let _ = client.query(query, &params).await?;
        info!("✅ Executed: {}", query);

        Ok(())
    }

    async fn initialize_databases() -> Result<HashMap<String, Pool>> {
        let mut databases = HashMap::new();

        let web_config = Self::load_db_config("WEB_DB").await?;
        let web_pool = Self::create_pool(web_config)
            .await
            .context("Failed to create web database pool")?;
        Self::test_connection(&web_pool, "web").await?;
        databases.insert("web".to_string(), web_pool);

        let game_config = Self::load_db_config("GAME_DB").await?;
        let game_pool = Self::create_pool(game_config)
            .await
            .context("Failed to create game database pool")?;
        Self::test_connection(&game_pool, "game").await?;
        databases.insert("game".to_string(), game_pool);

        let shop_config = Self::load_db_config("SHOP_DB").await?;
        let shop_pool = Self::create_pool(shop_config)
            .await
            .context("Failed to create shop database pool")?;
        Self::test_connection(&shop_pool, "shop").await?;
        databases.insert("shop".to_string(), shop_pool);

        let user_config = Self::load_db_config("USER_DB").await?;
        let user_pool = Self::create_pool(user_config)
            .await
            .context("Failed to create user database pool")?;
        Self::test_connection(&user_pool, "user").await?;
        databases.insert("user".to_string(), user_pool);

        let log_config = Self::load_db_config("LOG_DB").await?;
        let log_pool = Self::create_pool(log_config)
            .await
            .context("Failed to create log database pool")?;
        Self::test_connection(&log_pool, "log").await?;
        databases.insert("log".to_string(), log_pool);

        info!("✅ Databases initialized and added to the pool");
        Ok(databases)
    }

    async fn load_db_config(prefix: &str) -> Result<DBConfig> {
        Ok(DBConfig {
            host: env::var(format!("{}_HOST", prefix))
                .context(format!("Missing environment variable: {}_HOST", prefix))?,
            port: env::var(format!("{}_PORT", prefix))
                .context(format!("Missing environment variable: {}_PORT", prefix))?
                .parse()
                .context(format!("Invalid port number for {}_PORT", prefix))?,
            name: env::var(format!("{}_NAME", prefix))
                .context(format!("Missing environment variable: {}_NAME", prefix))?,
            user: env::var(format!("{}_USER", prefix))
                .context(format!("Missing environment variable: {}_USER", prefix))?,
            password: env::var(format!("{}_PASSWORD", prefix))
                .context(format!("Missing environment variable: {}_PASSWORD", prefix))?,
        })
    }

    async fn create_pool(cfg: DBConfig) -> Result<Pool> {
        let manager = Manager::new()
            .host(&cfg.host)
            .port(cfg.port)
            .database(&cfg.name)
            .authentication(tiberius::AuthMethod::sql_server(&cfg.user, &cfg.password))
            .encryption(tiberius::EncryptionLevel::NotSupported)
            .trust_cert()
            .application_name("gs-api");

        deadpool_tiberius::Pool::builder(manager)
            .max_size(
                env::var("MAX_DATABASE_POOL")
                    .context("Missing MAX_DATABASE_POOL environment variable")?
                    .parse::<usize>()
                    .context("Failed to parse MAX_DATABASE_POOL")?,
            )
            .build()
            .map_err(anyhow::Error::from)
    }

    async fn test_connection(pool: &Pool, name: &str) -> Result<()> {
        let mut client = pool
            .get()
            .await
            .context("Failed to get database connection")?;
        let stream = client
            .query("SELECT 1", &[])
            .await
            .context("Failed to execute test query")?;
        let row = stream
            .into_row()
            .await?
            .context("No rows returned from test query")?;
        let value: i32 = row.get(0).context("Unexpected empty row")?;

        if value == 1 {
            info!("✅ Successfully connected to {} database", name);
            Ok(())
        } else {
            anyhow::bail!("Unexpected test value from {} database: {}", name, value);
        }
    }

    pub fn get_database(&self, name: &str) -> Option<&Pool> {
        self.databases.get(name)
    }

    // Add this method to track pool metrics
    async fn track_pool_metrics(databases: Arc<HashMap<String, Pool>>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Every minute
        loop {
            interval.tick().await;

            for (name, pool) in databases.iter() {
                // Record current stats
                metrics::record_pool_usage(name, pool.status().size as u64);
                metrics::record_pool_usage(
                    &format!("{}_available", name),
                    pool.status().available as u64,
                );
            }
        }
    }
}

#[async_trait::async_trait]
pub trait DatabaseTrait {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError>
    where
        Self: Sized;
    async fn test_db_connection(&self) -> Result<(), DatabaseError>;
}

pub async fn test_db_connection(db: &impl DatabaseTrait) -> Result<bool, ()> {
    match db.test_db_connection().await {
        Ok(()) => Ok(true),  // If connection is successful, return Ok(true)
        Err(_) => Ok(false), // If connection fails, return Ok(false)
    }
}
