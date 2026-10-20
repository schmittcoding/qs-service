use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GvgConfig {
    pub min_guild_members: i32,
    pub match_duration_min: i32,
    pub tally_delay_min: i32,
    pub event_map_mid: i32,
    pub event_map_sid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KothConfig {
    pub match_duration_min: i32,
    pub tally_delay_min: i32,
    pub event_map_mid: i32,
    pub event_map_sid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GvgKothConfig {
    pub min_guild_members: i32,
    pub match_duration_min: i32,
    pub tally_delay_min: i32,
    pub event_map_mid: i32,
    pub event_map_sid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LevelCapRaceConfig {
    pub max_account_age_days: i32,
    pub target_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SpeedRunConfig {
    pub dungeon_map_mid: i32,
    pub dungeon_map_sid: i32,
    pub boss_mob_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CategoryConfig {
    Gvg(GvgConfig),
    Koth(KothConfig),
    GvgKoth(GvgKothConfig),
    LevelCapRace(LevelCapRaceConfig),
    SpeedRun(SpeedRunConfig),
}
