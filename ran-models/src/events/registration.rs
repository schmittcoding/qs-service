use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

/// Status values: 1=Active, 2=Withdrawn, 3=Disqualified
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventRegistration {
    pub registration_id: Uuid,
    pub event_id: Uuid,
    pub cha_num: i32,
    pub user_num: i32,
    pub guild_num: Option<i32>,
    pub cha_school: Option<i32>,
    pub cha_class: i32,
    pub cha_level: i32,
    pub status: i16,
    pub registered_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub ip_address: Option<String>,
}

impl EventRegistration {
    pub fn from_row(row: &Row) -> Self {
        Self {
            registration_id: row
                .get::<Uuid, _>("RegistrationID")
                .unwrap_or_else(Uuid::nil),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            guild_num: row.get::<i32, _>("GuildNum"),
            cha_school: row.get::<i32, _>("ChaSchool"),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_level: row.get::<i32, _>("ChaLevel").unwrap_or_default(),
            status: row
                .get::<u8, _>("Status")
                .map(|v| v as i16)
                .unwrap_or(1),
            registered_at: row
                .get::<NaiveDateTime, _>("RegisteredAt")
                .unwrap_or_default(),
            updated_at: row.get::<NaiveDateTime, _>("UpdatedAt"),
            ip_address: row
                .get::<&str, _>("IPAddress")
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub cha_num: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GvgRegisterRequest {
    pub guild_num: i32,
    pub member_cha_nums: Vec<i32>,
}
