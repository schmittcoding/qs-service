use anyhow::{Context, Result, anyhow};
use ran_models::{BaseDataCountResult, shop::orders::{ShopOrder, ShopOrderDetail, ShopOrderItem}};
use tiberius::Row;

use crate::{shopmanager::ShopDB, utils::list_response};

#[async_trait::async_trait]
pub trait MileageOrdersTrait {
    async fn get_user_mileage_orders(&self, user_num: i32, page: i32, limit: i32) -> Result<BaseDataCountResult<Row>>;
    async fn get_mileage_order_detail(&self, order_id: i64, user_num: Option<i32>) -> Result<ShopOrderDetail>;
    async fn get_admin_mileage_orders(&self, page: i32, limit: i32, search: Option<String>, status: Option<u8>) -> Result<BaseDataCountResult<Row>>;
}

#[async_trait::async_trait]
impl MileageOrdersTrait for ShopDB {
    async fn get_user_mileage_orders(&self, user_num: i32, page: i32, limit: i32) -> Result<BaseDataCountResult<Row>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let results = client
            .query(
                "EXEC sp_GetUserMileageOrders @UserNum = @P1, @Page = @P2, @PageSize = @P3",
                &[&user_num, &page, &limit],
            )
            .await
            .context("Failed to execute sp_GetUserMileageOrders")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        list_response(results)
    }

    async fn get_mileage_order_detail(&self, order_id: i64, user_num: Option<i32>) -> Result<ShopOrderDetail> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let results = client
            .query(
                "EXEC sp_GetMileageOrderDetail @OrderID = @P1, @UserNum = @P2",
                &[&order_id, &user_num],
            )
            .await
            .context("Failed to execute sp_GetMileageOrderDetail")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        let mut sets = results.into_iter();

        let order_row = sets
            .next()
            .and_then(|rows| rows.into_iter().next())
            .ok_or_else(|| anyhow!("Mileage order {} not found", order_id))?;

        let order = ShopOrder::from_row(&order_row);

        let items = sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| ShopOrderItem::from_row(row))
            .collect();

        Ok(ShopOrderDetail { order, items })
    }

    async fn get_admin_mileage_orders(&self, page: i32, limit: i32, search: Option<String>, status: Option<u8>) -> Result<BaseDataCountResult<Row>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let parsed_search_num: Option<i32> = search.as_deref().and_then(|s| s.parse().ok());

        let results = client
            .query(
                "EXEC sp_GetAdminMileageOrders @SearchNum = @P1, @Search = @P2, @Status = @P3, @Page = @P4, @PageSize = @P5",
                &[&parsed_search_num, &search, &status, &page, &limit],
            )
            .await
            .context("Failed to execute sp_GetAdminMileageOrders")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        list_response(results)
    }
}
