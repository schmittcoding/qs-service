use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

use super::match_schedule::MatchScheduleSummary;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventSnapshot {
    pub snapshot_id: Uuid,
    pub event_id: Uuid,
    pub match_id: Option<Uuid>,
    pub snapshot_time: NaiveDateTime,
    pub label: Option<String>,
    pub created_at: NaiveDateTime,
}

impl EventSnapshot {
    pub fn from_row(row: &Row) -> Self {
        Self {
            snapshot_id: row.get::<Uuid, _>("SnapshotID").unwrap_or_else(Uuid::nil),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            match_id: row.get::<Uuid, _>("MatchID"),
            snapshot_time: row.get::<NaiveDateTime, _>("SnapshotTime").unwrap_or_default(),
            label: row.get::<&str, _>("Label").map(|s| s.to_string()),
            created_at: row.get::<NaiveDateTime, _>("CreatedAt").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventSnapshotWithSchedule {
    #[serde(flatten)]
    pub snapshot: EventSnapshot,
    pub match_schedule: Option<MatchScheduleSummary>,
}

impl EventSnapshotWithSchedule {
    /// Create from a row that includes EventSnapshot + MatchSchedule columns joined together.
    /// The LEFT JOIN is detected by checking if MatchScheduleID is NULL.
    pub fn from_joined_row(row: &Row) -> Self {
        let snapshot = EventSnapshot::from_row(row);

        // Detect if the LEFT JOIN hit by checking if MatchScheduleID is NOT NULL
        let match_schedule_id: Option<Uuid> = row.get::<Uuid, _>("MatchScheduleID");

        let match_schedule = if match_schedule_id.is_some() {
            // LEFT JOIN hit — build MatchScheduleSummary from aliased columns
            Some(MatchScheduleSummary {
                event_date: row.get::<chrono::NaiveDate, _>("MatchDate").unwrap_or_default(),
                start_time: row.get::<NaiveDateTime, _>("StartTime").unwrap_or_default(),
                end_time: row.get::<NaiveDateTime, _>("EndTime").unwrap_or_default(),
                tower_points_th: row.get::<i32, _>("TowerPointsTH").unwrap_or(0),
                tower_points_faci: row.get::<i32, _>("TowerPointsFaci").unwrap_or(0),
                tower_points_nuc: row.get::<i32, _>("TowerPointsNuc").unwrap_or(0),
                tower_winner_th: row.get::<i32, _>("TowerWinnerTH"),
                tower_winner_faci: row.get::<i32, _>("TowerWinnerFaci"),
                tower_winner_nuc: row.get::<i32, _>("TowerWinnerNuc"),
                kills_counted: row.get::<i32, _>("KillsCounted"),
            })
        } else {
            // LEFT JOIN miss — set to None
            None
        };

        Self {
            snapshot,
            match_schedule,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SnapshotGuildScore {
    pub snapshot_id: Uuid,
    pub guild_num: i32,
    pub guild_name: String,
    pub cha_school: i32,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub tower_points: i32,
    pub deduction: i32,
    pub computed_score: i32,
    pub rank: i32,
    /// Positive = moved up, negative = moved down, None = new entry (no previous snapshot)
    pub rank_change: Option<i32>,
    /// Total registered GvG members in this guild at snapshot time
    pub member_count: i32,
}

impl SnapshotGuildScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            snapshot_id: row.get::<Uuid, _>("SnapshotID").unwrap_or_else(Uuid::nil),
            guild_num: row.get::<i32, _>("GuildNum").unwrap_or_default(),
            guild_name: row.get::<&str, _>("GuildName").map_or_else(String::new, |s| s.to_string()),
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or_default(),
            total_kills: row.get::<i32, _>("TotalKills").unwrap_or_default(),
            total_deaths: row.get::<i32, _>("TotalDeaths").unwrap_or_default(),
            tower_points: row.get::<i32, _>("TowerPoints").unwrap_or_default(),
            deduction: row.get::<i32, _>("Deduction").unwrap_or_default(),
            computed_score: row.get::<i32, _>("ComputedScore").unwrap_or_default(),
            rank: row.get::<i32, _>("Rank").unwrap_or_default(),
            rank_change: row.get::<i32, _>("RankChange"),
            member_count: row.get::<i32, _>("MemberCount").unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SnapshotKOTHScore {
    pub snapshot_id: Uuid,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: Option<i32>,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub deduction: i32,
    pub rank: i32,
    /// Positive = moved up, negative = moved down, None = new entry (no previous snapshot)
    pub rank_change: Option<i32>,
}

impl SnapshotKOTHScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            snapshot_id: row.get::<Uuid, _>("SnapshotID").unwrap_or_else(Uuid::nil),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row.get::<&str, _>("ChaName").map_or_else(String::new, |s| s.to_string()),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_school: row.get::<i32, _>("ChaSchool"),
            guild_num: row.get::<i32, _>("GuildNum"),
            guild_name: row.get::<&str, _>("GuildName").map(|s| s.to_string()),
            total_kills: row.get::<i32, _>("TotalKills").unwrap_or_default(),
            total_deaths: row.get::<i32, _>("TotalDeaths").unwrap_or_default(),
            deduction: row.get::<i32, _>("Deduction").unwrap_or_default(),
            rank: row.get::<i32, _>("Rank").unwrap_or_default(),
            rank_change: row.get::<i32, _>("RankChange"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SnapshotSchoolScore {
    pub snapshot_id: Uuid,
    pub cha_school: i32,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub total_tower: i32,
    pub deduction: i32,
    pub computed_score: i32,
    pub rank: i32,
    /// Positive = moved up, negative = moved down, None = new entry (no previous snapshot)
    pub rank_change: Option<i32>,
}

impl SnapshotSchoolScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            snapshot_id: row.get::<Uuid, _>("SnapshotID").unwrap_or_else(Uuid::nil),
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or_default(),
            total_kills: row.get::<i32, _>("TotalKills").unwrap_or_default(),
            total_deaths: row.get::<i32, _>("TotalDeaths").unwrap_or_default(),
            total_tower: row.get::<i32, _>("TotalTower").unwrap_or_default(),
            deduction: row.get::<i32, _>("Deduction").unwrap_or_default(),
            computed_score: row.get::<i32, _>("ComputedScore").unwrap_or_default(),
            rank: row.get::<i32, _>("Rank").unwrap_or_default(),
            rank_change: row.get::<i32, _>("RankChange"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SnapshotLevelCapRaceScore {
    pub snapshot_id: Uuid,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: Option<i32>,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub start_level: i32,
    pub current_level: i32,
    pub levels_gained: i32,
    pub rank: i32,
    /// Positive = moved up, negative = moved down, None = new entry (no previous snapshot)
    pub rank_change: Option<i32>,
}

impl SnapshotLevelCapRaceScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            snapshot_id: row.get::<Uuid, _>("SnapshotID").unwrap_or_else(Uuid::nil),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(String::new, |s| s.to_string()),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_school: row.get::<i32, _>("ChaSchool"),
            guild_num: row.get::<i32, _>("GuildNum"),
            guild_name: row.get::<&str, _>("GuildName").map(|s| s.to_string()),
            start_level: row.get::<i32, _>("StartLevel").unwrap_or_default(),
            current_level: row.get::<i32, _>("CurrentLevel").unwrap_or_default(),
            levels_gained: row.get::<i32, _>("LevelsGained").unwrap_or_default(),
            rank: row.get::<i32, _>("Rank").unwrap_or_default(),
            rank_change: row.get::<i32, _>("RankChange"),
        }
    }
}
