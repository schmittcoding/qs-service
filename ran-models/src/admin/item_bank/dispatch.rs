use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

use crate::util::naive_to_local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchItem {
    pub id: Uuid,
    pub dispatch_id: Uuid,
    pub product_num: i32,
    pub item_main: i32,
    pub item_sub: i32,
    pub item_name: String,
    pub created_at: DateTime<Local>,
}

impl ItemDispatchItem {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            id: row
                .get::<Uuid, _>("DispatchItemID")
                .unwrap_or_else(Uuid::nil),
            dispatch_id: row
                .get::<Uuid, _>("DispatchID")
                .unwrap_or_else(Uuid::nil),
            item_name: row
                .get::<&str, _>("DispatchItemName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_main: row.get::<i32, _>("DispatchItemMain").unwrap_or(0),
            item_sub: row.get::<i32, _>("DispatchItemSub").unwrap_or(0),
            product_num: row.get::<i32, _>("DispatchProductNum").unwrap_or(0),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchCharacter {
    pub id: Uuid,
    pub dispatch_id: Uuid,
    pub user_num: i64,
    pub user_name: String,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_level: i32,
    pub created_at: DateTime<Local>,
}

impl ItemDispatchCharacter {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get::<Uuid, _>("DispatchChaID")
                .unwrap_or_else(Uuid::nil),
            dispatch_id: row
                .get::<Uuid, _>("DispatchID")
                .unwrap_or_else(Uuid::nil),
            cha_name: row
                .get::<&str, _>("DispatchChaName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            cha_num: row.get::<i32, _>("DispatchChaNum").unwrap_or(0),
            cha_level: row.get::<i32, _>("DispatchChaLevel").unwrap_or(0),
            user_num: row.get::<i64, _>("DispatchUserNum").unwrap_or(0),
            user_name: row
                .get::<&str, _>("DispatchUserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatch {
    pub id: Uuid,
    pub notes: String,
    pub created_at: DateTime<Local>,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub items: Vec<ItemDispatchItem>,
    pub users: Vec<ItemDispatchCharacter>,
}

impl ItemDispatch {
    pub fn from_row(row: &Row, items: Vec<Row>, users: Vec<Row>) -> Self {
        Self {
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
            id: row
                .get::<Uuid, _>("DispatchID")
                .unwrap_or_else(Uuid::nil),
            admin_id: row
                .get::<Uuid, _>("UserID")
                .unwrap_or_else(Uuid::nil),
            admin_name: row
                .get::<&str, _>("UserName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            notes: row
                .get::<&str, _>("DispatchNotes")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            items: items
                .iter()
                .map(|row| ItemDispatchItem::from_row(row))
                .collect(),
            users: users
                .iter()
                .map(|row| ItemDispatchCharacter::from_row(row))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchList {
    pub id: Uuid,
    pub item_count: i32,
    pub user_count: i32,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub notes: String,
    pub created_at: DateTime<Local>,
}

impl ItemDispatchList {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get::<Uuid, _>("DispatchID")
                .unwrap_or_else(Uuid::nil),
            admin_id: row
                .get::<Uuid, _>("AdminID")
                .unwrap_or_else(Uuid::nil),
            admin_name: row
                .get::<&str, _>("AdminName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            notes: row
                .get::<&str, _>("DispatchNotes")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_count: row.get::<i32, _>("ItemCount").unwrap_or(0),
            user_count: row.get::<i32, _>("UserCount").unwrap_or(0),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchDetailItem {
    pub product_num: i32,
    pub quantity: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchDetailCharacter {
    pub cha_num: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDispatchDetailRequestPayload {
    pub characters: Vec<ItemDispatchDetailCharacter>,
    pub items: Vec<ItemDispatchDetailItem>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DispatchCharacterFilterRequest {
    pub search: Option<String>,
    pub min_level: Option<i32>,
    pub max_level: Option<i32>,
    pub login_from: Option<String>,
    pub login_to: Option<String>,
    pub page: Option<i16>,
    pub limit: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DispatchCharacterFilterResult {
    pub user_num: i32,
    pub user_id: String,
    pub cha_name: String,
    pub cha_num: i32,
    pub cha_level: i32,
    pub cha_class: i32,
    pub cha_school: i32,
}

impl DispatchCharacterFilterResult {
    pub fn from_row(row: &Row) -> Self {
        Self {
            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            user_id: row
                .get::<&str, _>("UserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_level: row.get::<i32, _>("ChaLevel").unwrap_or_default(),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or_default(),
        }
    }
}
