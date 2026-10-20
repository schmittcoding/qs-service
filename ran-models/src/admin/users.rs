use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct AdminUserProfile {
    pub user_name: String,
    pub user_id: Uuid,
    pub user_type: i16,
    pub user_available: i16
}

impl AdminUserProfile {
    pub fn from_row(row: &Row) -> Self {
        Self {
            user_id: row
                .get::<Uuid, _>("UserID")
                .unwrap_or_else(Uuid::nil), // Default to nil UUID if conversion fails
            user_name: row
                .get::<&str, _>("UserName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            user_available: row.get::<i16, _>("UserAvailable").unwrap_or(1),
            user_type: row.get::<i16, _>("UserType").unwrap_or(1),
        }
    }
}