use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GuildEventScore {
    pub rank: i64,
    pub event_id: Uuid,
    pub guild_num: i32,
    pub guild_name: String,
    pub cha_school: i32,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub tower_points: i32,
    pub deduction: i32,
    pub computed_score: i32,
}

impl GuildEventScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            rank: row.get::<i64, _>("Rank").unwrap_or(0),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            guild_num: row.get::<i32, _>("GuildNum").unwrap_or_default(),
            guild_name: row.get::<&str, _>("GuildName").map_or_else(String::new, |s| s.to_string()),
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or_default(),
            total_kills: row.get::<i32, _>("TotalKills").unwrap_or_default(),
            total_deaths: row.get::<i32, _>("TotalDeaths").unwrap_or_default(),
            tower_points: row.get::<i32, _>("TowerPoints").unwrap_or_default(),
            deduction: row.get::<i32, _>("Deduction").unwrap_or_default(),
            computed_score: row.get::<i32, _>("ComputedScore").unwrap_or_default(),
        }
    }
}
