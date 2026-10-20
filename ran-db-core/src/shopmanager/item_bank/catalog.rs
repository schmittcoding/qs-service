use anyhow::{Context, Result};
use tiberius::Row;

use crate::shopmanager::ShopDB;

#[async_trait::async_trait]
pub trait ItemBankCatalogTrait {
    async fn get_item_map_options(&self) -> Result<Vec<Row>>;
}

#[async_trait::async_trait]
impl ItemBankCatalogTrait for ShopDB {
    async fn get_item_map_options(&self) -> Result<Vec<Row>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT ProductNum, ItemMain, ItemSub, ItemName FROM dbo.ShopItemMap ORDER BY ItemName ASC";

        let results = client
            .query(query, &[])
            .await?
            .into_results()
            .await?
            .into_iter()
            .next()
            .unwrap_or_default();

        Ok(results)
    }
}
