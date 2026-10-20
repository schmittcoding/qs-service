use anyhow::{Context, Result};
use ran_models::item_shop::GetItemShopResponse;
use tiberius::Row;

use crate::shopmanager::ShopDB;

#[async_trait::async_trait]
pub trait MileageShopTrait {
    async fn get_mileage_shop_items(&self, user_num: i64) -> Result<Vec<GetItemShopResponse>>;
    async fn mileage_purchase(&self, user_num: i64, purchased_items: String) -> Result<Row>;
}

#[async_trait::async_trait]
impl MileageShopTrait for ShopDB {
    async fn get_mileage_shop_items(&self, user_num: i64) -> Result<Vec<GetItemShopResponse>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetMileageShopItems @UserNum = @P1", &[&user_num])
            .await
            .context("Failed to execute sp_GetMileageShopItems")?
            .into_results()
            .await?;

        let result_sets = results.into_iter().next().unwrap_or_default();

        let mut res: Vec<GetItemShopResponse> = result_sets
            .into_iter()
            .map(|row| GetItemShopResponse::from_row(&row))
            .collect();

        res.sort_by(|a, b| {
            let a_discounted = a.discount_percent > 0.0;
            let b_discounted = b.discount_percent > 0.0;

            b_discounted
                .cmp(&a_discounted)
                .then(b.discount_percent.total_cmp(&a.discount_percent))
                .then(a.final_price.total_cmp(&b.final_price))
                .then(a.product_num.cmp(&b.product_num))
        });

        Ok(res)
    }

    async fn mileage_purchase(&self, user_num: i64, purchased_items: String) -> Result<Row> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_mileage_purchase @UserNum = @P1, @PurchaseItemsJson = @P2";
        let result = client
            .query(sp_query, &[&user_num, &purchased_items])
            .await?
            .into_row()
            .await?;

        match result {
            Some(row) => Ok(row),
            None => Err(anyhow::anyhow!("Failed to read mileage purchase result"))
        }
    }
}
