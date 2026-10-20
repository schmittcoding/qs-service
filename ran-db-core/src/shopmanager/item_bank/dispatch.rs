use anyhow::{Context, Result};
use ran_models::{BaseDataCountResult, admin::item_bank::ItemDispatch};
use uuid::Uuid;

use crate::{shopmanager::ShopDB, utils::list_response};

#[async_trait::async_trait]
pub trait ItemBankDispatchTrait {
    async fn get_dispatch_list(
        &self,
        page: i16,
        page_size: i16,
        search: Option<String>,
    ) -> Result<BaseDataCountResult>;
    async fn dispatch_item(
        &self,
        items_json: String,
        characters_json: String,
        notes: Option<String>,
        admin_id: String,
    ) -> Result<(String, Uuid)>;
    async fn get_dispatch_item_by_id(&self, id: Uuid) -> Result<ItemDispatch>;
}

#[async_trait::async_trait]
impl ItemBankDispatchTrait for ShopDB {
    async fn get_dispatch_list(
        &self,
        page: i16,
        page_size: i16,
        search: Option<String>,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetItemDispatchList
            @Search = @P1,
            @Page = @P2,
            @PageSize = @P3";

        let results = client
            .query(sp_query, &[
                &search,
                &page,
                &page_size
            ])
            .await?
            .into_results()
            .await?;

        list_response(results)
    }

    async fn dispatch_item(
        &self,
        items_json: String,
        characters_json: String,
        notes: Option<String>,
        admin_id: String,
    ) -> Result<(String, Uuid)> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_DispatchItem
            @DispatchBy = @P1,
            @DispatchNotes = @P2,
            @ItemsJson = @P3,
            @CharsJson = @P4";

        let result = client
            .query(sp_query, &[
                &admin_id,
                &notes,
                &items_json,
                &characters_json
            ])
            .await?
            .into_row()
            .await?;

        match result {
            Some(row) => {
                let message = row
                    .try_get::<&str, _>("Message")
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                let id = if message == "SUCCESS" {
                    row
                        .get::<Uuid, _>("DispatchID")
                        .unwrap_or_else(Uuid::nil)
                } else {
                    Uuid::nil()
                };

                Ok((message.to_string(), id))
            },
            None => {
                anyhow::bail!("No response from sp_DispatchItem procedure");
            }
        }
    }

    async fn get_dispatch_item_by_id(&self, id: Uuid) -> Result<ItemDispatch> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetDispatchItemById @DispatchId = @P1";

        let results = client
            .query(sp_query, &[&id])
            .await
            .context("Failed to execute stored procedure: sp_GetDispatchItemById")?
            .into_results()
            .await
            .context("Failed to parse result sets from MSSQL query")?;

        let mut result_sets = results.into_iter();

        let details_rows = result_sets.next().unwrap_or_default();
        let details_row = details_rows.first();

        let chars_row = result_sets
            .next()
            .unwrap_or_else(Vec::new);

        let items_row = result_sets
            .next()
            .unwrap_or_else(Vec::new);

        match details_row {
            Some(row) => Ok(ItemDispatch::from_row(row, items_row, chars_row)),
            None => anyhow::bail!("Dispatch with ID {} not found", id),
        }
    }
}
