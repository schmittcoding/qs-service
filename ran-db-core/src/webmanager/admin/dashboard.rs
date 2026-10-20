use anyhow::Context;
use anyhow::Result;

use crate::webmanager::WebDB;

#[async_trait::async_trait]
pub trait WebDashboardTrait {
    async fn get_registered_user_count(&self) -> Result<i32>;
    async fn get_transaction_count_by_status(&self, status: i16) -> Result<i32>;
    async fn get_total_recharge_amount(&self, period: &str) -> Result<f64>;
}

#[async_trait::async_trait]
impl WebDashboardTrait for WebDB {
    async fn get_registered_user_count(&self) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_GetRegisteredUserCount";
        let row = client
            .query(query, &[])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(r) => {
                let count = r
                    .try_get::<i32, _>("RegisteredCount")
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                Ok(count)
            }
            None => Ok(0),
        }
    }

    async fn get_transaction_count_by_status(&self, status: i16) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_GetTransactionCountByStatus @Status = @P1";
        let row = client
            .query(query, &[&status])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(r) => {
                let count = r
                    .try_get::<i32, _>("TransactionCount")
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                Ok(count)
            }
            None => Ok(0),
        }
    }

    async fn get_total_recharge_amount(&self, period: &str) -> Result<f64> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_GetTotalRechargeAmount @Period = @P1";
        let row = client
            .query(query, &[&period])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(r) => {
                let amount = r
                    .try_get::<f64, _>("TotalAmount")
                    .ok()
                    .flatten()
                    .unwrap_or(0.0);
                Ok(amount)
            }
            None => Ok(0.0),
        }
    }
}
