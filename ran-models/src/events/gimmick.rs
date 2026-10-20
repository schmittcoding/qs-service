use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum GimmickType {
    DoublePoints,
    NullDeaths,
}

impl GimmickType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            GimmickType::DoublePoints => "DoublePoints",
            GimmickType::NullDeaths => "NullDeaths",
        }
    }
}

impl TryFrom<&str> for GimmickType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "DoublePoints" => Ok(GimmickType::DoublePoints),
            "NullDeaths" => Ok(GimmickType::NullDeaths),
            _ => Err(format!("Invalid GimmickType: {}", s)),
        }
    }
}

impl FromStr for GimmickType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventMatchGimmick {
    pub gimmick_id: Uuid,
    pub match_id: Uuid,
    pub gimmick_type: GimmickType,
    pub gimmick_start_time: NaiveDateTime,
    pub gimmick_end_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl EventMatchGimmick {
    pub fn from_row(row: &Row) -> Self {
        let gimmick_type_str = row
            .get::<&str, _>("GimmickType")
            .unwrap_or("DoublePoints");
        let gimmick_type = GimmickType::try_from(gimmick_type_str)
            .unwrap_or(GimmickType::DoublePoints);

        Self {
            gimmick_id: row.get::<Uuid, _>("GimmickID").unwrap_or_else(Uuid::nil),
            match_id: row.get::<Uuid, _>("MatchID").unwrap_or_else(Uuid::nil),
            gimmick_type,
            gimmick_start_time: row
                .get::<NaiveDateTime, _>("GimmickStartTime")
                .unwrap_or_default(),
            gimmick_end_time: row
                .get::<NaiveDateTime, _>("GimmickEndTime")
                .unwrap_or_default(),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateGimmickRequest {
    pub gimmick_type: GimmickType,
    pub gimmick_start_time: NaiveDateTime,
    pub gimmick_end_time: NaiveDateTime,
}
