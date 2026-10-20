use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: Uuid,
    pub provider: String,
    pub link: String,
    pub status: u8,
    pub r#type: i16,
    pub version: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DownloadItemPayload {
    pub provider: String,
    pub link: String,
    pub status: u8,
    pub r#type: i16,
    pub version: String,
}

impl DownloadItem {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get::<Uuid, _>("WebDownloadID")
                .unwrap_or_else(Uuid::nil), // Default to nil UUID if conversion fails
            provider: row
                .get::<&str, _>("WebDownloadProvider")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            link: row
                .get::<&str, _>("WebDownloadLink")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            version: row
                .get::<&str, _>("WebDownloadVersion")
                .map(|s| s.to_string()).unwrap_or_default(),
            status: row
                .try_get::<bool, _>("WebDownloadStatus")
                .ok()
                .flatten()
                .unwrap_or(false) as u8,
            r#type: row.get::<i16, _>("WebDownloadType").unwrap_or(0),
        }
    }
}