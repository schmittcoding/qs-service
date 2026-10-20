use std::fs;

use anyhow::{Context, Result};
use ran_models::{admin::items::{RawItemExchangeLog}, BaseDataCountResult};

use crate::logmanager::LogDB;

#[async_trait::async_trait]
pub trait ItemLogTrait {
    async fn get_item_exchange_logs(
        &self,
        page: i16,
        limit: i16,
        item_from: Option<i32>,
        item_to: Option<i32>,
        item_id: Option<String>
    ) -> Result<BaseDataCountResult<RawItemExchangeLog>>;

    async fn migrate_item_list(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl ItemLogTrait for LogDB {
    async fn migrate_item_list(&self) -> Result<()> {
        let json_str = fs::read_to_string("itemStrTable.json")?;
        // let json_data: ItemJsonFile = serde_json::from_str(&json_str)?;



        // let current = meta_coll
        //     .find_one(doc! {
        //         "type": "ItemList",
        //         "version": &json_data.version
        //     })
        //     .await?;

        // if current.is_some() {
        //     log::info!("✅ Item list already up-to-date (version: {}).", json_data.version);
        //     return Ok(());
        // }

        // log::info!("🔄 Syncing item list version {}...", json_data.version);
        // collection.delete_many(doc! {}).await?;
        // collection.insert_many(&json_data.items).await?;

        // meta_coll.find_one_and_update(
        //     doc! { "type": "ItemList" },
        //     doc! { "$set": { "version": &json_data.version } }
        // ).upsert(true).await?;

        // log::info!("✅ Item list synced to version {}.", json_data.version);
        Ok(())
    }

    async fn get_item_exchange_logs(
        &self,
        page: i16,
        limit: i16,
        item_from: Option<i32>,
        item_to: Option<i32>,
        item_id: Option<String>
    ) -> Result<BaseDataCountResult<RawItemExchangeLog>> {
        let mut client = self.pool
            .get()
            .await
            .context("❌ Failed to acquire MSSQL connection from pool")?;

        let query = "EXEC sp_GetItemExchangeLog @Page = @P1, @PageSize = @P2, @ItemFrom = @P3, @ItemTo = @P4, @ItemId = @P5";
        let results = client
            .query(query, &[
                &page,
                &limit,
                &item_from,
                &item_to,
                &item_id
            ])
            .await
            .context("❌ Failed to execute stored procedure: sp_GetItemExchangeLog")?
            .into_results()
            .await
            .context("❌ Failed to parse result sets from MSSQL query")?;

        let mut result_sets = results.into_iter();

        let total_records = result_sets.next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let logs = result_sets
            .next()
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|row| RawItemExchangeLog::from_row(&row))
            .collect::<Vec<_>>();

        Ok(BaseDataCountResult { data: logs, total_records })
    }
}
