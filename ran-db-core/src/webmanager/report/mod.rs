use anyhow::{Context, Result};
use tiberius::{Row};
use crate::webmanager::WebDB;

pub struct TransactionReportResult {
    pub summary: Vec<Row>,
    pub transactions: Vec<Row>,
    pub total_records: i32,
}

#[async_trait::async_trait]
pub trait WebReportTrait {
    async fn get_transaction_report(&self, 
        page: i16,
        page_size: i16,
        r#type: Option<i16>,
        status: Option<i16>,
        search: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<TransactionReportResult>;
}

#[async_trait::async_trait]
impl WebReportTrait for WebDB {
    async fn get_transaction_report(
        &self,
        page: i16,
        page_size: i16,
        r#type: Option<i16>,
        status: Option<i16>,
        search: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>
    ) -> Result<TransactionReportResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Prepare stored procedure call
        let sp_query = "EXEC GetWebTransactionSummaryAndList 
            @UserName = @P1, 
            @TransactionType = @P2, 
            @TransactionStatus = @P3, 
            @StartDate = @P4, 
            @EndDate = @P5, 
            @Page = @P6, 
            @PageSize = @P7;";

        let results = client
            .query(
                sp_query,
                &[
                    &search,        // @P1
                    &r#type,        // @P2
                    &status,        // @P3
                    &start_date,    // @P4
                    &end_date,      // @P5
                    &page,          // @P6
                    &page_size,     // @P7
                ],
            )
            .await?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();
        let summary = result_sets.next().unwrap_or_default();
        
        let total_count_rows = result_sets.next().unwrap_or_default();
        let total_records = total_count_rows
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let transactions = result_sets.next().unwrap_or_default();

        Ok(TransactionReportResult {
            summary,
            transactions,
            total_records,
        })

    }
}