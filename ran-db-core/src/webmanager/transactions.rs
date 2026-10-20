use super::WebDB;
use anyhow::{Context, Result};
use ran_models::{BaseDataCountResult};

#[async_trait::async_trait]
pub trait WebTransactionUserTrait {
    async fn get_user_transactions(
        &self,
        page: i16,
        page_size: i16,
        user_id: i64,
    ) -> Result<BaseDataCountResult>;
}

#[async_trait::async_trait]
impl WebTransactionUserTrait for WebDB {
    async fn get_user_transactions(
        &self,
        page: i16,
        page_size: i16,
        user_id: i64,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetWebTransactions @UserID = @P1, @Page = @P2, @PageSize = @P3;";

        let results = client
            .query(sp_query, &[
                &page,
                &page_size,
                &user_id
            ])
            .await?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();

        let total_count_rows = result_sets.next().unwrap_or_default();
        let total_records = total_count_rows
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let transactions = result_sets.next().unwrap_or_default();

        Ok(BaseDataCountResult { data: transactions, total_records })
    }
}
