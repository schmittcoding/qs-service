use crate::{DBManager, DatabaseTrait};
use anyhow::{Context, Result};
use base64::Engine;
use deadpool_tiberius::Pool;
use futures_util::StreamExt;
use ran_models::{
    errors::DatabaseError,
    shop::{EditShopData, ShopData},
};
use tiberius::Row;
use tracing::{error, info};

pub mod items;
pub mod item_shop;
pub mod item_bank;
pub mod orders;
pub mod purchases;

pub mod mileage_shop;
pub mod mileage_orders;
pub mod mileage_config;

fn parse_shop_info(row: &Row) -> Option<ShopData> {
    Some(ShopData {
        product_num: row.get::<i32, _>("ProductNum")?,
        item_main: row
            .try_get::<i32, _>("ItemMain")
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_sub: row
            .try_get::<i32, _>("ItemSub")
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_name: row
            .try_get::<&str, _>("ItemName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        item_price: row
            .try_get::<f64, _>("ItemPrice") // use f64 for MONEY
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_category: row
            .try_get::<i16, _>("ItemCategory")
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_description: row
            .try_get::<&str, _>("ItemDescription")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        item_discount: row.try_get::<f64, _>("ItemDiscount").ok().flatten(),
        item_discount_type: row.try_get::<i16, _>("ItemDiscountType").ok().flatten(),
        item_image: row
            .try_get::<&[u8], _>("ItemImage")
            .ok()
            .flatten()
            .map(|slice| slice.to_vec())
            .unwrap_or_default(),
        item_status: row
            .try_get::<i16, _>("ItemStatus")
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_stock: row
            .try_get::<i32, _>("ItemStock")
            .ok()
            .flatten()
            .unwrap_or_default(),
        item_purchase_limit: row
            .try_get::<i32, _>("PurchaseLimitPerAccount")
            .ok()
            .flatten(),
    })
}

#[derive(Clone)]
pub struct ShopDB {
    pub pool: Pool,
}

#[async_trait::async_trait]
impl DatabaseTrait for ShopDB {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError> {
        let pool = db_manager
            .get_database("shop")
            .ok_or(DatabaseError::UserNotFound)?
            .clone();

        // Verify pool configuration
        let status = pool.status();
        tracing::debug!(
            "ShopDB pool - max_size: {}, idle: {}, available: {}",
            status.max_size,
            status.size - status.available,
            status.available
        );

        Ok(Self { pool })
    }

    async fn test_db_connection(&self) -> Result<(), DatabaseError> {
        let mut client = self.pool.get().await?;
        let stream = client.query("SELECT 1", &[]).await?;
        let row = stream
            .into_row()
            .await?
            .ok_or(DatabaseError::TestConnectionFailed)?;

        match row.get::<i32, _>(0) {
            Some(_) => {
                info!("✅ Shop DB connection verified");
                Ok(())
            }
            _ => {
                error!("Unexpected test value");
                Err(DatabaseError::TestConnectionFailed)
            }
        }
    }
}

#[async_trait::async_trait]
pub trait ShopDBTrait {
    async fn get_shop_data(&self) -> Result<Vec<ShopData>>;
    async fn get_shop_item_by_id(&self, product_num: i32) -> Result<ShopData>;
    async fn edit_shop_item_by_id(
        &self,
        product_num: i32,
        shop_data: EditShopData,
    ) -> Result<ShopData>;
    async fn delete_shop_item_by_id(&self, product_num: i32) -> Result<ShopData>;
}

#[async_trait::async_trait]
impl ShopDBTrait for ShopDB {
    async fn get_shop_data(&self) -> Result<Vec<ShopData>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Call the updated stored procedure
        let mut stream = client
            .query("SELECT * FROM YXShopItemMap", &[])
            .await
            .context("Failed to execute query")?;

        let mut shop_data = Vec::new();

        while let Some(result) = stream.next().await {
            let row = result.context("Error processing row")?;
            if let Some(row_data) = row.as_row() {
                if let Some(product_num) = row_data.try_get::<i32, _>("ProductNum").ok().flatten() {
                    let item_name = row_data
                        .try_get::<&str, _>("ItemName")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_price = row_data
                        .try_get::<f64, _>("ItemPrice") // use f64 for MONEY
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_category = row_data
                        .try_get::<i16, _>("ItemCategory")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_description = row_data
                        .try_get::<&str, _>("ItemDescription")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_discount = row_data.try_get::<f64, _>("ItemDiscount").ok().flatten();

                    let item_discount_type = row_data
                        .try_get::<i16, _>("ItemDiscountType")
                        .ok()
                        .flatten();

                    let item_image = row_data
                        .try_get::<&[u8], _>("ItemImage")
                        .ok()
                        .flatten()
                        .map(|slice| slice.to_vec())
                        .unwrap_or_default();

                    let item_main = row_data
                        .try_get::<i32, _>("ItemMain")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_sub = row_data
                        .try_get::<i32, _>("ItemSub")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_status = row_data
                        .try_get::<i16, _>("ItemStatus")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_stock: i32 = row_data
                        .try_get::<i32, _>("ItemStock")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let item_purchase_limit = row_data
                        .try_get::<i32, _>("PurchaseLimitPerAccount")
                        .ok()
                        .flatten();

                    let data: ShopData = ShopData::new(
                        product_num,
                        item_main,
                        item_sub,
                        item_name.to_string(),
                        item_price,
                        item_category,
                        item_description.to_string(),
                        item_discount,
                        item_discount_type,
                        item_image,
                        item_status,
                        item_stock,
                        item_purchase_limit
                    );

                    shop_data.push(data);
                }
            }
        }

        info!("✅ Retrieved {} items in shop", shop_data.len());
        Ok(shop_data)
    }

    async fn get_shop_item_by_id(&self, product_num: i32) -> Result<ShopData> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM YXShopItemMap WHERE ProductNum = @P1";
        let mut stream = client
            .query(query, &[&product_num])
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.as_row() {
                        if let Some(item) = parse_shop_info(&row_data) {
                            return Ok(item);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Shop item with num {} not found",
            product_num
        ))
    }

    async fn edit_shop_item_by_id(
        &self,
        product_num: i32,
        shop_data: EditShopData,
    ) -> Result<ShopData> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let image_bytes: Vec<u8> = if let Some(encoded) = shop_data.item_image {
            match base64::engine::general_purpose::STANDARD.decode(encoded.trim()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    log::error!("Failed to decode image data: {:?}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let query = "
        UPDATE YXShopItemMap
        SET
            ItemMain = @P2,
            ItemSub = @P3,
            ItemName = @P4,
            ItemPrice = @P5,
            ItemCategory = @P6,
            ItemDescription = @P7,
            ItemDiscount = @P8,
            ItemImage = @P9,
            ItemDiscountType = @P10,
            ItemStatus = @P11,
            ItemStock = @P12,
            PurchaseLimitPerAccount = @P13,
            PurchaseLimitType = @P14
        OUTPUT inserted.*
        WHERE ProductNum = @P1
        ";

        let mut stream = client
            .query(
                query,
                &[
                    &product_num,
                    &shop_data.item_main,
                    &shop_data.item_sub,
                    &shop_data.item_name,
                    &shop_data.item_price,
                    &shop_data.item_category,
                    &shop_data.item_description,
                    &shop_data.item_discount,
                    &image_bytes,
                    &shop_data.item_discount_type,
                    &shop_data.item_status,
                    &shop_data.item_stock,
                    &shop_data.item_purchase_limit,
                    &shop_data.item_purchase_limit_type
                ],
            )
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.as_row() {
                        if let Some(item) = parse_shop_info(&row_data) {
                            return Ok(item);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Shop item with num {} not found",
            product_num
        ))
    }

    async fn delete_shop_item_by_id(&self, product_num: i32) -> Result<ShopData> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "
            DELETE FROM YXShopItemMap
            OUTPUT deleted.*
            WHERE ProductNum = @P1
        ";

        let mut stream = client
            .query(query, &[&product_num])
            .await
            .context("Failed to execute delete query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.as_row() {
                        if let Some(item) = parse_shop_info(&row_data) {
                            return Ok(item);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row during deletion: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Shop item with num {} not found",
            product_num
        ))
    }
}
