use chrono::{DateTime, Local, NaiveDateTime};
use tiberius::numeric::Numeric;
use serde::Serialize;
use tiberius::Row;

use crate::util::naive_to_local;

// Slim view returned by sp_GetUserOrders (user-facing list — no identity or point fields).
#[derive(Serialize)]
pub struct ShopOrderSummary {
    pub order_id:       i64,
    pub order_date:     DateTime<Local>,
    pub total_items:    i32,
    pub total_quantity: i32,
    pub total_amount:   f64,
    pub total_discount: f64,
    pub order_status:   u8,
    pub created_at:     DateTime<Local>,
}

impl ShopOrderSummary {
    pub fn from_row(row: &Row) -> Self {
        Self {
            order_id: row.get::<i64, _>("OrderID").unwrap_or_default(),
            order_date: row
                .get::<NaiveDateTime, _>("OrderDate")
                .map(naive_to_local)
                .unwrap_or_default(),
            total_items: row.get::<i32, _>("TotalItems").unwrap_or_default(),
            total_quantity: row.get::<i32, _>("TotalQuantity").unwrap_or_default(),
            total_amount: row
                .try_get::<Numeric, _>("TotalAmount")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            total_discount: row
                .try_get::<Numeric, _>("TotalDiscount")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            order_status: row.get::<u8, _>("OrderStatus").unwrap_or_default(),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
pub struct ShopOrder {
    pub order_id:       i64,
    pub user_num:       i32,
    pub user_uid:       String,
    pub order_date:     DateTime<Local>,
    pub total_items:    i32,
    pub total_quantity: i32,
    pub total_amount:   f64,
    pub total_discount: f64,
    pub points_before:  f64,
    pub points_after:   f64,
    pub order_status:   u8,
    pub created_at:     DateTime<Local>,
}

#[derive(Serialize)]
pub struct ShopOrderItem {
    pub order_item_id:      i64,
    pub order_id:           i64,
    pub user_num:           i32,
    pub product_num:        i32,
    pub item_name:          String,
    pub item_main:          i32,
    pub item_sub:           i32,
    pub quantity:           i32,
    pub unit_price:         f64,
    pub discount_type:      Option<u8>,
    pub discount_value:     Option<f64>,
    pub final_unit_price:   f64,
    pub line_total:         f64,
}

#[derive(Serialize)]
pub struct ShopOrderItemWithDate {
    #[serde(flatten)]
    pub item:           ShopOrderItem,
    pub order_date:     DateTime<Local>,
    pub order_status:   u8,
}

#[derive(Serialize)]
pub struct ShopOrderDetail {
    pub order: ShopOrder,
    pub items: Vec<ShopOrderItem>,
}

impl ShopOrder {
    pub fn from_row(row: &Row) -> Self {
        Self {
            order_id: row.get::<i64, _>("OrderID").unwrap_or_default(),
            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            user_uid: row
                .get::<&str, _>("UserUID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            order_date: row
                .get::<NaiveDateTime, _>("OrderDate")
                .map(naive_to_local)
                .unwrap_or_default(),
            total_items: row.get::<i32, _>("TotalItems").unwrap_or_default(),
            total_quantity: row.get::<i32, _>("TotalQuantity").unwrap_or_default(),
            total_amount: row
                .try_get::<Numeric, _>("TotalAmount")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            total_discount: row
                .try_get::<Numeric, _>("TotalDiscount")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            points_before: row
                .try_get::<Numeric, _>("PointsBefore")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            points_after: row
                .try_get::<Numeric, _>("PointsAfter")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            order_status: row.get::<u8, _>("OrderStatus").unwrap_or_default(),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
        }
    }
}

impl ShopOrderItem {
    pub fn from_row(row: &Row) -> Self {
        Self {
            order_item_id: row.get::<i64, _>("OrderItemID").unwrap_or_default(),
            order_id: row.get::<i64, _>("OrderID").unwrap_or_default(),
            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            product_num: row.get::<i32, _>("ProductNum").unwrap_or_default(),
            item_name: row
                .get::<&str, _>("ItemName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_main: row.get::<i32, _>("ItemMain").unwrap_or_default(),
            item_sub: row.get::<i32, _>("ItemSub").unwrap_or_default(),
            quantity: row.get::<i32, _>("Quantity").unwrap_or_default(),
            unit_price: row
                .try_get::<Numeric, _>("UnitPrice")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            discount_type: row.try_get::<u8, _>("DiscountType").ok().flatten(),
            discount_value: row
                .try_get::<Numeric, _>("DiscountValue")
                .ok()
                .flatten()
                .map(f64::from),
            final_unit_price: row
                .try_get::<Numeric, _>("FinalUnitPrice")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            line_total: row
                .try_get::<Numeric, _>("LineTotal")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
        }
    }
}

impl ShopOrderItemWithDate {
    pub fn from_row(row: &Row) -> Self {
        Self {
            item: ShopOrderItem::from_row(row),
            order_date: row
                .get::<NaiveDateTime, _>("OrderDate")
                .map(naive_to_local)
                .unwrap_or_default(),
            order_status: row.get::<u8, _>("OrderStatus").unwrap_or_default(),
        }
    }
}
