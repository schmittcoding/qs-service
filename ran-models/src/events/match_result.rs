use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MatchGuildResult {
    pub match_id: Uuid,
    pub guild_num: i32,
    pub guild_name: String,
    pub cha_school: i32,
    pub match_kills: i32,
    pub match_deaths: i32,
    pub match_tower: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MatchCharResult {
    pub match_id: Uuid,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub match_kills: i32,
    pub match_deaths: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchGuildResultExport {
    pub match_id: Uuid,
    pub match_label: Option<String>,
    pub match_date: NaiveDate,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub tally_status: String,
    pub rows_tallied: Option<i32>,
    pub kills_counted: Option<i32>,
    pub kills_rejected: Option<i32>,
    pub guild_num: i32,
    pub guild_name: String,
    pub cha_school: i32,
    pub match_kills: i32,
    pub match_deaths: i32,
    pub match_tower: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchKothResultExport {
    pub match_id: Uuid,
    pub match_label: Option<String>,
    pub match_date: NaiveDate,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub tally_status: String,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub match_kills: i32,
    pub match_deaths: i32,
}
