use chrono::{DateTime, Local, NaiveDateTime};
use serde::Serialize;
use tiberius::Row;

use crate::util::naive_to_local;

#[derive(Serialize)]
pub struct ItemBankEntry {
    pub purchase_id:        i32,
    pub user_num:           i32,
    pub user_name:          String,
    pub product_num:        i32,
    pub purchase_date:      DateTime<Local>,
    pub last_change_date:   DateTime<Local>,
    pub purchase_status:    i32,
    pub item_name:          String,
    pub item_main:          i32,
    pub item_sub:           i32,
}

impl ItemBankEntry {
    pub fn from_row(row: &Row) -> Self {
        Self {
            purchase_id: row.get::<i32, _>("PurchaseID").unwrap_or(0),
            user_num: row.get::<i32, _>("UserNum").unwrap_or(0),
            user_name: row
                .get::<&str, _>("UserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            product_num: row.get::<i32, _>("ProductNum").unwrap_or(0),
            purchase_date: row
                .get::<NaiveDateTime, _>("PurchaseDate")
                .map(naive_to_local)
                .unwrap_or_default(),
            last_change_date: row
                .get::<NaiveDateTime, _>("LastChangeDate")
                .map(naive_to_local)
                .unwrap_or_default(),
            purchase_status: row.get::<i32, _>("PurchaseStatus").unwrap_or(0),
            item_name: row
                .get::<&str, _>("ItemName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_main: row.get::<i32, _>("ItemMain").unwrap_or(0),
            item_sub: row.get::<i32, _>("ItemSub").unwrap_or(0),
        }
    }
}
