pub mod auth;
pub mod download;
pub mod email;
pub mod events;
pub mod news;
pub mod transaction;
pub mod user;
pub mod report;
pub mod admin;
pub mod transactions;

use crate::{metrics, DBManager, DatabaseTrait};
use deadpool_tiberius::Pool;
use ran_models::errors::DatabaseError;
use tracing::{error, info};

#[derive(Clone)]
pub struct WebDB {
    pub pool: Pool,
}

#[async_trait::async_trait]
impl DatabaseTrait for WebDB {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError> {
        metrics::record_connection_attempt("web");
        let pool = match db_manager.get_database("web") {
            Some(pool) => {
                metrics::record_connection_success("web");
                pool.clone()
            }
            None => {
                metrics::record_connection_failure("web");
                return Err(DatabaseError::UserNotFound);
        }
        };

        Ok(Self { pool })
    }

    async fn test_db_connection(&self) -> Result<(), DatabaseError> {
        let mut timer = metrics::QueryTimer::new("test_connection_web");

        let mut client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get web DB connection: {}", e);
                timer.finish_failure();
                return Err(DatabaseError::ConnectionError(e));
            }
        };

        let stream = match client.query("SELECT 1", &[]).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to execute test query: {}", e);
                timer.finish_failure();
                return Err(DatabaseError::QueryError(e));
            }
        };

        let row = match stream.into_row().await {
            Ok(Some(row)) => row,
            Ok(None) => {
                error!("No rows returned from test query");
                timer.finish_failure();
                return Err(DatabaseError::TestConnectionFailed);
            }
            Err(e) => {
                error!("Error fetching row: {}", e);
                timer.finish_failure();
                return Err(DatabaseError::QueryError(e));
            }
        };

        match row.get::<i32, _>(0) {
            Some(1) => {
                info!("✅ Web DB connection verified");
                timer.finish_success();
                Ok(())
            }
            _ => {
                error!("Unexpected test value");
                timer.finish_failure();
                Err(DatabaseError::TestConnectionFailed)
            }
        }
    }
}
