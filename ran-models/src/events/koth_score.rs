use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KOTHScore {
    pub rank: i64,
    pub event_id: Uuid,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: Option<i32>,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub deduction: i32,
}

impl KOTHScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            rank: row.get::<i64, _>("Rank").unwrap_or(0),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row.get::<&str, _>("ChaName").map_or_else(String::new, |s| s.to_string()),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_school: row.get::<i32, _>("ChaSchool"),
            guild_num: row.get::<i32, _>("GuildNum"),
            guild_name: row.get::<&str, _>("GuildName").map(|s| s.to_string()),
            total_kills: row.get::<i32, _>("TotalKills").unwrap_or_default(),
            total_deaths: row.get::<i32, _>("TotalDeaths").unwrap_or_default(),
            deduction: row.get::<i32, _>("Deduction").unwrap_or_default(),
        }
    }
}
