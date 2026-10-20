use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

use crate::events::gimmick::EventMatchGimmick;

/// TallyStatus: Pending, Waiting, Processing, Completed, Failed
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MatchSchedule {
    pub match_id: Uuid,
    pub event_id: Uuid,
    pub match_label: Option<String>,
    pub match_date: NaiveDate,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub tally_scheduled: NaiveDateTime,
    pub tally_status: String,
    pub tally_started_at: Option<NaiveDateTime>,
    pub tally_completed_at: Option<NaiveDateTime>,
    pub rows_tallied: Option<i32>,
    pub kills_counted: Option<i32>,
    pub kills_rejected: Option<i32>,
    /// Tower point values for this match (rotational per match, default 0)
    pub tower_points_th: i32,
    pub tower_points_nuc: i32,
    pub tower_points_faci: i32,
    /// Which school (ChaSchool INT) won each tower — set by admin after match ends.
    /// NULL means no school controlled that tower.
    pub tower_winner_th: Option<i32>,
    pub tower_winner_nuc: Option<i32>,
    pub tower_winner_faci: Option<i32>,
}

impl MatchSchedule {
    pub fn from_row(row: &Row) -> Self {
        Self {
            match_id: row.get::<Uuid, _>("MatchID").unwrap_or_else(Uuid::nil),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            match_label: row.get::<&str, _>("MatchLabel").map(|s| s.to_string()),
            match_date: row.get::<NaiveDate, _>("MatchDate").unwrap_or_default(),
            start_time: row.get::<NaiveDateTime, _>("StartTime").unwrap_or_default(),
            end_time: row.get::<NaiveDateTime, _>("EndTime").unwrap_or_default(),
            tally_scheduled: row.get::<NaiveDateTime, _>("TallyScheduled").unwrap_or_default(),
            tally_status: row
                .get::<&str, _>("TallyStatus")
                .map_or_else(|| "Pending".to_string(), |s| s.to_string()),
            tally_started_at: row.get::<NaiveDateTime, _>("TallyStartedAt"),
            tally_completed_at: row.get::<NaiveDateTime, _>("TallyCompletedAt"),
            rows_tallied: row.get::<i32, _>("RowsTallied"),
            kills_counted: row.get::<i32, _>("KillsCounted"),
            kills_rejected: row.get::<i32, _>("KillsRejected"),
            tower_points_th: row.get::<i32, _>("TowerPointsTH").unwrap_or(0),
            tower_points_nuc: row.get::<i32, _>("TowerPointsNuc").unwrap_or(0),
            tower_points_faci: row.get::<i32, _>("TowerPointsFaci").unwrap_or(0),
            tower_winner_th: row.get::<i32, _>("TowerWinnerTH"),
            tower_winner_nuc: row.get::<i32, _>("TowerWinnerNuc"),
            tower_winner_faci: row.get::<i32, _>("TowerWinnerFaci"),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MatchWithGimmicks {
    #[serde(flatten)]
    pub match_schedule: MatchSchedule,
    pub gimmicks: Vec<EventMatchGimmick>,
}

/// Slim summary of MatchSchedule for public/non-admin endpoints
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MatchScheduleSummary {
    pub event_date: NaiveDate,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub tower_points_th: i32,
    pub tower_points_faci: i32,
    pub tower_points_nuc: i32,
    pub tower_winner_th: Option<i32>,
    pub tower_winner_faci: Option<i32>,
    pub tower_winner_nuc: Option<i32>,
    #[serde(rename = "total_kills")]
    pub kills_counted: Option<i32>,
}

impl MatchScheduleSummary {
    pub fn from_row(row: &Row) -> Self {
        Self {
            event_date: row.get::<NaiveDate, _>("MatchDate").unwrap_or_default(),
            start_time: row.get::<NaiveDateTime, _>("StartTime").unwrap_or_default(),
            end_time: row.get::<NaiveDateTime, _>("EndTime").unwrap_or_default(),
            tower_points_th: row.get::<i32, _>("TowerPointsTH").unwrap_or(0),
            tower_points_faci: row.get::<i32, _>("TowerPointsFaci").unwrap_or(0),
            tower_points_nuc: row.get::<i32, _>("TowerPointsNuc").unwrap_or(0),
            tower_winner_th: row.get::<i32, _>("TowerWinnerTH"),
            tower_winner_faci: row.get::<i32, _>("TowerWinnerFaci"),
            tower_winner_nuc: row.get::<i32, _>("TowerWinnerNuc"),
            kills_counted: row.get::<i32, _>("KillsCounted"),
        }
    }
}

/// Display status for the frontend banner
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub enum MatchDisplayStatus {
    Idle,
    Live { match_info: MatchSchedule },
    TallyingSoon { match_info: MatchSchedule },
    TallyingNow { match_info: MatchSchedule },
}
