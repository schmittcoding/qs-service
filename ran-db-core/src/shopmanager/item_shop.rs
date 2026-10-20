use anyhow::{Context, Result};
use ran_models::{BaseDataCountResult, item_shop::{ItemGalleryImage, ItemShopBase, ItemShopData, GetItemShopResponse}};
use tiberius::Row;
use uuid::Uuid;

use crate::{shopmanager::ShopDB, utils::list_response};

#[async_trait::async_trait]
pub trait ItemShopTrait {
    async fn get_shop_items(&self, user_num: i64) -> Result<Vec<GetItemShopResponse>>;
    async fn get_admin_shop_items(
        &self,
        page: i32,
        limit: i32,
        search: Option<String>,
        category_ids: Option<Vec<i32>>,
        status_values: Option<Vec<i32>>,
        purchase_limit_values: Option<Vec<i32>>,
        shop_type: i16,
    ) -> Result<BaseDataCountResult>;

    async fn get_shop_item_by_id(&self, product_num: i32) -> Result<Row>;
    async fn add_shop_item(&self, item_data: ItemShopBase, image_url: String) -> Result<ItemShopData>;
    async fn edit_shop_item(&self, product_num: i32, item_data: ItemShopBase, image_url: String) -> Result<ItemShopData>;

    async fn shop_purchase(&self, user_num: i64, purchased_items: String) -> Result<Row>;

    async fn get_item_gallery(&self, product_num: i32) -> Result<Vec<ItemGalleryImage>>;
    async fn add_item_gallery(&self, product_num: i32, images_json: String) -> Result<Vec<ItemGalleryImage>>;
    async fn delete_item_gallery(&self, product_num: i32, image_ids: Vec<Uuid>) -> Result<()>;
}

#[async_trait::async_trait]
impl ItemShopTrait for ShopDB {
    async fn get_shop_items(&self, user_num: i64) -> Result<Vec<GetItemShopResponse>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetShopItemsWithLimits @UserNum = @P1", &[&user_num])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        let result_sets = results.into_iter().next().unwrap_or_default();

        let mut res: Vec<GetItemShopResponse> = result_sets
            .into_iter()
            .map(|row| GetItemShopResponse::from_row(&row))
            .collect();

        res
            .sort_by(|a, b| {
                let a_discounted = a.discount_percent > 0.0;
                let b_discounted = b.discount_percent > 0.0;

                b_discounted.cmp(&a_discounted)
                    .then(b.discount_percent.total_cmp(&a.discount_percent))
                    .then(a.final_price.total_cmp(&b.final_price))
                    .then(a.product_num.cmp(&b.product_num))
            });

        Ok(res)
    }

    async fn get_admin_shop_items(
        &self,
        page: i32,
        limit: i32,
        search: Option<String>,
        category_ids: Option<Vec<i32>>,
        status_values: Option<Vec<i32>>,
        purchase_limit_values: Option<Vec<i32>>,
        shop_type: i16,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Convert vectors to comma-separated strings
        let category_list = category_ids
            .as_ref()
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let status_list = status_values
            .as_ref()
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let purchase_limit_list = purchase_limit_values
            .as_ref()
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let sp_query = "EXEC sp_GetPaginatedShopData
            @Search = @P1,
            @Category = @P2,
            @Status = @P3,
            @PurchaseLimit = @P4,
            @ShopType = @P5,
            @Page = @P6,
            @PageSize = @P7";

        let results = client
            .query(sp_query, &[
                &search,
                &category_list,
                &status_list,
                &purchase_limit_list,
                &shop_type,
                &page,
                &limit,
            ])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        list_response(results)
    }

    async fn get_shop_item_by_id(&self, product_num: i32) -> Result<Row> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetShopItemByID @ProductNum = @P1", &[&product_num])
            .await
            .context("Failed to execute query")?
            .into_row()
            .await?;

        match results {
            Some(row) => Ok(row),
            None => Err(anyhow::anyhow!("Shop item with num {} not found", product_num))
        }
    }

    async fn edit_shop_item(&self, product_num: i32, item_data: ItemShopBase, image_url: String) -> Result<ItemShopData> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_EditShopItem
            @ProductNum = @P1,
            @ItemMain = @P2,
            @ItemSub = @P3,
            @ItemName = @P4,
            @ItemPrice = @P5,
            @ItemCategory = @P6,
            @ItemDescription = @P7,
            @ItemDiscount = @P8,
            @ItemDiscountType = @P9,
            @NewItemImage = @P10,
            @ItemStatus = @P11,
            @ItemStock = @P12,
            @ItemTag = @P13,
            @PurchaseLimitPerAccount = @P14,
            @PurchaseLimitType = @P15,
            @ShopType = @P16";

        let result = client
            .query(query, &[
                &product_num,
                &item_data.item_main,
                &item_data.item_sub,
                &item_data.item_name,
                &item_data.item_price,
                &item_data.item_category,
                &item_data.item_description,
                &item_data.item_discount,
                &item_data.item_discount_type,
                &image_url,
                &item_data.item_status,
                &item_data.item_stock,
                &item_data.item_tag,
                &item_data.item_purchase_limit,
                &item_data.item_purchase_limit_type,
                &item_data.shop_type
            ])
            .await
            .context("Failed to execute query")?
            .into_row()
            .await?;

        match result {
            Some(row) => Ok(ItemShopData::from_row(&row)),
            None => Err(anyhow::anyhow!("Failed to read updated row from database"))
        }
    }

    async fn add_shop_item(&self, item_data: ItemShopBase, image_url: String) -> Result<ItemShopData> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_AddShopItem
            @ItemMain = @P1,
            @ItemSub = @P2,
            @ItemName = @P3,
            @ItemPrice = @P4,
            @ItemCategory = @P5,
            @ItemDescription = @P6,
            @ItemDiscount = @P7,
            @ItemDiscountType = @P8,
            @NewItemImage = @P9,
            @ItemStatus = @P10,
            @ItemStock = @P11,
            @ItemTag = @P12,
            @PurchaseLimitPerAccount = @P13,
            @PurchaseLimitType = @P14,
            @ShopType = @P15";

        let result = client
            .query(query, &[
                &item_data.item_main,
                &item_data.item_sub,
                &item_data.item_name,
                &item_data.item_price,
                &item_data.item_category,
                &item_data.item_description,
                &item_data.item_discount,
                &item_data.item_discount_type,
                &image_url,
                &item_data.item_status,
                &item_data.item_stock,
                &item_data.item_tag,
                &item_data.item_purchase_limit,
                &item_data.item_purchase_limit_type,
                &item_data.shop_type
            ])
            .await
            .context("Failed to execute query")?
            .into_row()
            .await?;

        match result {
            Some(row) => Ok(ItemShopData::from_row(&row)),
            None => Err(anyhow::anyhow!("Failed to read inserted row from database"))
        }
    }

    async fn shop_purchase(&self, user_num: i64, purchased_items: String) -> Result<Row> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Call the stored procedure that accepts JSON.
        let sp_query = "EXEC sp_shop_purchase @UserNum = @P1, @PurchaseItemsJson = @P2";
        let result= client
            .query(sp_query, &[&user_num, &purchased_items])
            .await?
            .into_row()
            .await?;

        match result {
            Some(row) => Ok(row),
            None => Err(anyhow::anyhow!("❌ Error reading row: {:?}", result))
        }
    }

    async fn get_item_gallery(&self, product_num: i32) -> Result<Vec<ItemGalleryImage>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetItemImages @ProductNum = @P1", &[&product_num])
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        let rows = results.into_iter().next().unwrap_or_default();

        Ok(rows.iter().map(|row| ItemGalleryImage::from_row(row)).collect())
    }

    async fn add_item_gallery(&self, product_num: i32, images_json: String) -> Result<Vec<ItemGalleryImage>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query(
                "EXEC sp_AddItemImages @ProductNum = @P1, @ImagesJson = @P2",
                &[&product_num, &images_json]
            )
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        let rows = results.into_iter().next().unwrap_or_default();

        Ok(rows.iter().map(|row| ItemGalleryImage::from_row(row)).collect())
    }

    async fn delete_item_gallery(&self, product_num: i32, image_ids: Vec<Uuid>) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let ids_json = serde_json::to_string(&image_ids)
            .context("Failed to serialize image IDs")?;

        client
            .query(
                "EXEC sp_DeleteItemImages @ProductNum = @P1, @ImageIdsJson = @P2",
                &[&product_num, &ids_json]
            )
            .await
            .context("Failed to execute query")?
            .into_results()
            .await?;

        Ok(())
    }
}
