use anyhow::{Context, Result};
use ran_models::{admin::players::{UserAccountInfo}, BaseDataCountResult};

use crate::webmanager::{report::TransactionReportResult, WebDB};

#[async_trait::async_trait]
pub trait WebPlayerTrait {
    async fn get_players(
        &self,
        page: i16,
        page_size: i16,
        category_values: Option<Vec<i16>>,
        status_values: Option<Vec<i16>>,
        search: Option<String>,
    ) -> Result<BaseDataCountResult>;

    async fn get_player_by_id(
        &self,
        user_num: i64
    ) -> Result<UserAccountInfo>;

    async fn get_account_transaction(
        &self,
        page: i16,
        page_size: i16,
        user_num: i64,
        start_date: Option<String>,
        end_date: Option<String>
    ) -> Result<TransactionReportResult>;
}

#[async_trait::async_trait]
impl WebPlayerTrait for WebDB {
    async fn get_players(
        &self,
        page: i16,
        page_size: i16,
        category_values: Option<Vec<i16>>,
        status_values: Option<Vec<i16>>,
        search: Option<String>,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Parse search string as i64 for numeric ID search, fallback to None if not a number
        let parsed_search: Option<i64> = match search.clone() {
            Some(s) => match s.parse::<i64>() {
                Ok(parsed) => Some(parsed),
                Err(_) => None,
            },
            None => None,
        };

        // Convert vectors to comma-separated strings
        let user_type_list = category_values
            .as_ref()
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let user_status_list = status_values
            .as_ref()
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let sp_query = "EXEC sp_GetPlayersList
            @SearchNum = @P1,
            @Search = @P2,
            @UserType = @P3,
            @UserStatus = @P4,
            @Page = @P5,
            @PageSize = @P6";

        let results = client
            .query(sp_query, &[
                &parsed_search,
                &search,
                &user_type_list,
                &user_status_list,
                &page,
                &page_size
            ])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        let mut result_sets = results.into_iter();

        let total_count = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        Ok(BaseDataCountResult {
            data: result_sets
                .next()
                .unwrap_or_default(),
            total_records: total_count
        })
    }

    async fn get_player_by_id(
        &self,
        user_num: i64
    ) -> Result<UserAccountInfo> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetPlayerById @UserNum = @P1";

        let results = client
            .query(sp_query, &[&user_num])
            .await?
            .into_row()
            .await?;

        match results {
            Some(row) => {
                Ok(UserAccountInfo::from_row(&row))
            }
            None => {
                Err(anyhow::anyhow!(
                    "User {} not found",
                    user_num
                ))
            }
        }
    }

    async fn get_account_transaction(
        &self,
        page: i16,
        page_size: i16,
        user_num: i64,
        start_date: Option<String>,
        end_date: Option<String>
    ) -> Result<TransactionReportResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Prepare stored procedure call
        let sp_query = "EXEC sp_GetAccountTransactionHistory 
            @UserNum = @P1,
            @StartDate = @P2, 
            @EndDate = @P3, 
            @Page = @P4, 
            @PageSize = @P5;";

        let results = client
            .query(
                sp_query,
                &[
                    &user_num,      // @P1
                    &start_date,    // @P2
                    &end_date,      // @P3
                    &page,          // @P4
                    &page_size,     // @P5
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
            total_records,
            transactions
        })
    }
}