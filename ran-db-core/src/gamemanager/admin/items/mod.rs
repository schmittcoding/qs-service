use anyhow::{Context, Result};
use ran_models::admin::items::Item;
use tiberius::Row;
use uuid::Uuid;

use crate::gamemanager::GameDB;

#[async_trait::async_trait]
pub trait GameItemTrait {
    async fn get_items(
        &self,
        page: i16,
        page_size: i16,
        cha_num: Option<i32>,
        item_main: Option<i16>,
        item_sub: Option<i16>,
        inven_type: Option<u8>,
    ) -> Result<Vec<Row>>;

    async fn get_item_by_uuid(&self, item_uuid: Uuid) -> Result<Option<Item>>;

    async fn get_items_by_cha_num(
        &self,
        page: i16,
        page_size: i16,
        cha_num: i32,
        item_main: Option<i16>,
        item_sub: Option<i16>,
        inven_type: Option<u8>,
    ) -> Result<Vec<Row>>;
}

#[async_trait::async_trait]
impl GameItemTrait for GameDB {
    async fn get_items(
        &self,
        page: i16,
        page_size: i16,
        cha_num: Option<i32>,
        item_main: Option<i16>,
        item_sub: Option<i16>,
        inven_type: Option<u8>,
    ) -> Result<Vec<Row>> {
        let mut client = self.pool.get().await?;

        let sp_query = "EXEC sp_GetItems @Page = @P1, @PageSize = @P2, @ChaNum = @P3, @ItemMID = @P4, @ItemSID = @P5, @InvenType = @P6";

        let rows = client
            .query(
                sp_query,
                &[&page, &page_size, &cha_num, &item_main, &item_sub, &inven_type],
            )
            .await
            .context("Query execution failed")?
            .into_first_result()
            .await
            .context("Failed to fetch results")?;

        Ok(rows)
    }

    async fn get_item_by_uuid(&self, item_uuid: Uuid) -> Result<Option<Item>> {
        let mut client = self.pool.get().await?;

        let sp_query = "EXEC sp_GetItemByUUID @ItemUUID = @P1";

        let results = client
            .query(sp_query, &[&item_uuid])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        Ok(results.map(|row| Item::from_row(&row)))
    }

    async fn get_items_by_cha_num(
        &self,
        page: i16,
        page_size: i16,
        cha_num: i32,
        item_main: Option<i16>,
        item_sub: Option<i16>,
        inven_type: Option<u8>,
    ) -> Result<Vec<Row>> {
        let mut client = self.pool.get().await?;

        let sp_query = "EXEC sp_GetItemsByChaNum @ChaNum = @P1, @Page = @P2, @PageSize = @P3, @ItemMID = @P4, @ItemSID = @P5, @InvenType = @P6";

        let rows = client
            .query(sp_query, &[&cha_num, &page, &page_size, &item_main, &item_sub, &inven_type])
            .await
            .context("Query execution failed")?
            .into_first_result()
            .await
            .context("Failed to fetch results")?;

        Ok(rows)
    }
}
