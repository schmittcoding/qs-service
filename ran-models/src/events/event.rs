use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

use super::category::EventCategory;
use super::category_config::CategoryConfig;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventDefinition {
    pub event_id: Uuid,
    pub event_name: String,
    pub event_slug: String,
    pub season: i32,
    pub registration_open: NaiveDateTime,
    pub registration_close: NaiveDateTime,
    pub event_start: NaiveDateTime,
    pub event_end: NaiveDateTime,
    pub min_level: i32,
    pub is_active: bool,
    pub event_category: EventCategory,
    pub category_config: Option<CategoryConfig>,
    pub created_at: NaiveDateTime,
}

impl EventDefinition {
    pub fn from_row(row: &Row) -> Self {
        let event_category = row
            .try_get::<&str, _>("EventCategory")
            .ok()
            .flatten()
            .and_then(EventCategory::from_db_str)
            .unwrap_or(EventCategory::Gvg);

        // Read flat columns from DB
        let match_duration_min = row.get::<i32, _>("MatchDurationMin").unwrap_or(45);
        let tally_delay_min = row.get::<i32, _>("TallyDelayMin").unwrap_or(30);
        let min_guild_members = row.get::<i32, _>("MinGuildMembers").unwrap_or(0);
        let event_map_mid = row.get::<i32, _>("EventMapMID").unwrap_or(0);
        let event_map_sid = row.get::<i32, _>("EventMapSID").unwrap_or(0);

        // Parse optional JSON for variant-specific fields
        let json_config = row
            .try_get::<&str, _>("CategoryConfig")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());

        // Build typed CategoryConfig variant
        let category_config = match event_category {
            EventCategory::Gvg => Some(CategoryConfig::Gvg(
                super::category_config::GvgConfig {
                    min_guild_members,
                    match_duration_min,
                    tally_delay_min,
                    event_map_mid,
                    event_map_sid,
                },
            )),
            EventCategory::Koth => Some(CategoryConfig::Koth(
                super::category_config::KothConfig {
                    match_duration_min,
                    tally_delay_min,
                    event_map_mid,
                    event_map_sid,
                },
            )),
            EventCategory::GvgKoth => Some(CategoryConfig::GvgKoth(
                super::category_config::GvgKothConfig {
                    min_guild_members,
                    match_duration_min,
                    tally_delay_min,
                    event_map_mid,
                    event_map_sid,
                },
            )),
            EventCategory::LevelCapRace => {
                let max_account_age_days = json_config
                    .as_ref()
                    .and_then(|v| v.get("max_account_age_days"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(0);
                let target_level = json_config
                    .as_ref()
                    .and_then(|v| v.get("target_level"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(0);
                Some(CategoryConfig::LevelCapRace(
                    super::category_config::LevelCapRaceConfig {
                        max_account_age_days,
                        target_level,
                    },
                ))
            }
            EventCategory::SpeedRun => {
                let dungeon_map_mid = json_config
                    .as_ref()
                    .and_then(|v| v.get("dungeon_map_mid"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(0);
                let dungeon_map_sid = json_config
                    .as_ref()
                    .and_then(|v| v.get("dungeon_map_sid"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(0);
                let boss_mob_id = json_config
                    .as_ref()
                    .and_then(|v| v.get("boss_mob_id"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                Some(CategoryConfig::SpeedRun(
                    super::category_config::SpeedRunConfig {
                        dungeon_map_mid,
                        dungeon_map_sid,
                        boss_mob_id,
                    },
                ))
            }
        };

        Self {
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            event_name: row
                .get::<&str, _>("EventName")
                .map_or_else(String::new, |s| s.to_string()),
            event_slug: row
                .get::<&str, _>("EventSlug")
                .map_or_else(String::new, |s| s.to_string()),
            season: row.get::<i32, _>("Season").unwrap_or_default(),
            registration_open: row
                .get::<NaiveDateTime, _>("RegistrationOpen")
                .unwrap_or_default(),
            registration_close: row
                .get::<NaiveDateTime, _>("RegistrationClose")
                .unwrap_or_default(),
            event_start: row
                .get::<NaiveDateTime, _>("EventStart")
                .unwrap_or_default(),
            event_end: row.get::<NaiveDateTime, _>("EventEnd").unwrap_or_default(),
            min_level: row.get::<i32, _>("MinLevel").unwrap_or(200),
            is_active: row
                .try_get::<bool, _>("IsActive")
                .ok()
                .flatten()
                .unwrap_or(true),
            event_category,
            category_config,
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .unwrap_or_default(),
        }
    }

    pub fn is_registration_open(&self, now: NaiveDateTime) -> bool {
        now >= self.registration_open && now <= self.registration_close
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventDefinitionAdmin {
    #[serde(flatten)]
    pub base: EventDefinition,
    pub registration_count: i32,
    pub match_count: i32
}

impl EventDefinitionAdmin {
    pub fn from_row(row: &Row) -> Self {
        Self {
            base: EventDefinition::from_row(row),
            registration_count: row.get::<i32, _>("RegistrationCount").unwrap_or(0),
            match_count: row.get::<i32, _>("MatchCount").unwrap_or(0),
        }
    }
}

// ──────────────────────── Request DTOs ────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEventRequest {
    pub event_name: String,
    pub event_slug: String,
    pub season: i32,
    pub registration_open: NaiveDateTime,
    pub registration_close: NaiveDateTime,
    pub event_start: NaiveDateTime,
    pub event_end: NaiveDateTime,
    pub min_level: i32,
    pub event_category: Option<EventCategory>,
    pub category_config: Option<CategoryConfig>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEventRequest {
    pub event_name: Option<String>,
    pub event_slug: Option<String>,
    pub registration_open: Option<NaiveDateTime>,
    pub registration_close: Option<NaiveDateTime>,
    pub event_start: Option<NaiveDateTime>,
    pub event_end: Option<NaiveDateTime>,
    pub min_level: Option<i32>,
    pub event_category: Option<EventCategory>,
    pub category_config: Option<CategoryConfig>,
}

// ──────────────────────── Match DTOs ────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateMatchRequest {
    pub match_label: Option<String>,
    pub match_date: NaiveDate,
    pub start_time: NaiveDateTime,
    /// Tower point values for this match (rotational). Defaults to 0.
    #[serde(default)]
    pub tower_points_th: i32,
    #[serde(default)]
    pub tower_points_nuc: i32,
    #[serde(default)]
    pub tower_points_faci: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateMatchesRequest {
    pub matches: Vec<CreateMatchRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateMatchesRequest {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days_of_week: Vec<String>,
    pub start_hour: u32,
    pub start_minute: u32,
    #[serde(default = "default_label_prefix")]
    pub label_prefix: String,
}

fn default_label_prefix() -> String { "Week".to_string() }

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMatchRequest {
    pub match_label: Option<String>,
    pub start_time: Option<NaiveDateTime>,
    /// Tower point values (rotational per match)
    pub tower_points_th: Option<i32>,
    pub tower_points_nuc: Option<i32>,
    pub tower_points_faci: Option<i32>,
}

/// Admin sets tower winners after a match ends, triggering tally.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetTowerResultsRequest {
    /// ChaSchool INT of the school that won Town Hall tower. NULL = no winner.
    pub tower_winner_th: Option<i32>,
    /// ChaSchool INT of the school that won Nuclear tower. NULL = no winner.
    pub tower_winner_nuc: Option<i32>,
    /// ChaSchool INT of the school that won Facility tower. NULL = no winner.
    pub tower_winner_faci: Option<i32>,
}

// ──────────────────────── Dashboard Response ────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EventDashboard {
    pub event: EventDefinition,
    pub registration_stats: RegistrationStats,
    pub match_stats: MatchStats,
    pub deduction_stats: DeductionStats,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegistrationStats {
    pub total_registered: i32,
    pub active: i32,
    pub withdrawn: i32,
    pub disqualified: i32,
    pub guild_count: i32,
    pub school_breakdown: Vec<SchoolBreakdown>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SchoolBreakdown {
    pub cha_school: i32,
    pub count: i32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MatchStats {
    pub total_matches: i32,
    pub completed: i32,
    pub pending: i32,
    pub next_match: Option<NextMatch>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NextMatch {
    pub match_id: Uuid,
    pub label: Option<String>,
    pub start_time: NaiveDateTime,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DeductionStats {
    pub total_deductions: i32,
    pub total_points_deducted: i32,
    pub reverted: i32,
}
