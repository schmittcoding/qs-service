use serde::{Deserialize, Serialize};
use tiberius::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemCatalogOption {
    pub product_num: i32,
    pub name: String,
    pub item_main: i32,
    pub item_sub: i32,
}

impl ItemCatalogOption {
    pub fn from_row(row: &Row) -> Self {
        Self {
            product_num: row.get::<i32, _>("ProductNum").unwrap_or(0),
            name: row
                .get::<&str, _>("ItemName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_main: row.get::<i32, _>("ItemMain").unwrap_or(0),
            item_sub: row.get::<i32, _>("ItemSub").unwrap_or(0),
        }
    }
}
