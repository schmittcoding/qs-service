use crate::{DBManager, DatabaseTrait};
use deadpool_tiberius::Pool;
use ran_models::errors::DatabaseError;
use tracing::{error, info};

pub mod news;
pub mod logs;

#[derive(Clone)]
pub struct LogDB {
    pub pool: Pool,
}

#[async_trait::async_trait]
impl DatabaseTrait for LogDB {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError> {
        let pool = db_manager
            .get_database("log")
            .ok_or(DatabaseError::UserNotFound)?;
        Ok(Self { pool: pool.clone() })
    }

    async fn test_db_connection(&self) -> Result<(), DatabaseError> {
        let mut client = self.pool.get().await?;
        let stream = client.query("SELECT 1", &[]).await?;
        let row = stream
            .into_row()
            .await?
            .ok_or(DatabaseError::TestConnectionFailed)?;

        match row.get::<i32, _>(0) {
            Some(_) => {
                info!("✅ Log DB connection verified");
                Ok(())
            }
            _ => {
                error!("Unexpected test value");
                Err(DatabaseError::TestConnectionFailed)
            }
        }
    }
}
