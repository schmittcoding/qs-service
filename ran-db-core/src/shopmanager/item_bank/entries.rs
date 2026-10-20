use anyhow::{Context, Result};
use ran_models::{BaseDataCountResult, admin::item_bank::ItemBankEntry};
use tiberius::Row;

use crate::{shopmanager::ShopDB, utils::list_response};

#[async_trait::async_trait]
pub trait ItemBankEntriesTrait {
    async fn get_shop_purchases(&self, page: i16, limit: i16, search: Option<String>, status: Option<i16>) -> Result<BaseDataCountResult>;
    async fn get_shop_purchase_by_id(&self, purchase_id: i32) -> Result<Row>;
    async fn get_user_shop_purchases(&self, user_num: i32, page: i16, limit: i16, status: Option<i16>) -> Result<BaseDataCountResult>;
}

#[async_trait::async_trait]
impl ItemBankEntriesTrait for ShopDB {
    async fn get_shop_purchases(&self, page: i16, limit: i16, search: Option<String>, status: Option<i16>) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let parsed_search: Option<i32> = match search.clone() {
            Some(s) => match s.parse::<i32>() {
                Ok(parsed) => Some(parsed),
                Err(_) => None,
            },
            None => None,
        };

        let results = client
            .query("EXEC sp_GetShopPurchases @SearchNum = @P1, @Search = @P2, @Status = @P3, @Page = @P4, @PageSize = @P5", &[
                &parsed_search,
                &search,
                &status,
                &page,
                &limit
            ])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();

        let total_records = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        Ok(BaseDataCountResult {
            data: result_sets.next().unwrap_or_default(),
            total_records,
        })
    }

    async fn get_shop_purchase_by_id(&self, purchase_id: i32) -> Result<Row> {
        let mut client = self.pool.get().await?;

        let results = client
            .query("EXEC sp_GetShopPurchaseById @PurchaseID = @P1", &[&purchase_id])
            .await
            .context("Failed to execute query")?
            .into_row()
            .await?;

        match results {
            Some(row) => Ok(row),
            None => Err(anyhow::anyhow!("Shop purchase with ID {} not found", purchase_id))
        }
    }

    async fn get_user_shop_purchases(&self, user_num: i32, page: i16, limit: i16, status: Option<i16>) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetUserShopPurchases @UserNum = @P1, @Status = @P2, @Page = @P3, @PageSize = @P4", &[
                &user_num,
                &status,
                &page,
                &limit
            ])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        list_response(results)
    }
}
