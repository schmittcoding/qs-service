use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

/// TargetType: "GUILD", "PLAYER", "SCHOOL"
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AdminDeduction {
    pub deduction_id: Uuid,
    pub event_id: Uuid,
    pub target_type: String,
    pub target_id: Option<i32>,
    pub target_cha_school: Option<i32>,
    pub points: i32,
    pub reason: String,
    pub admin_user_id: Uuid,
    pub applied_at: NaiveDateTime,
    pub match_id: Option<Uuid>,
    pub is_reverted: bool,
    pub reverted_at: Option<NaiveDateTime>,
    pub reverted_by: Option<Uuid>,
}

impl AdminDeduction {
    pub fn from_row(row: &Row) -> Self {
        Self {
            deduction_id: row.get::<Uuid, _>("DeductionID").unwrap_or_else(Uuid::nil),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
            target_type: row
                .get::<&str, _>("TargetType")
                .map_or_else(String::new, |s| s.to_string()),
            target_id: row.get::<i32, _>("TargetID"),
            target_cha_school: row.get::<i32, _>("TargetChaSchool"),
            points: row.get::<i32, _>("Points").unwrap_or(0),
            reason: row
                .get::<&str, _>("Reason")
                .map_or_else(String::new, |s| s.to_string()),
            admin_user_id: row
                .get::<Uuid, _>("AdminUserID")
                .unwrap_or_else(Uuid::nil),
            applied_at: row
                .get::<NaiveDateTime, _>("AppliedAt")
                .unwrap_or_default(),
            match_id: row.get::<Uuid, _>("MatchID"),
            is_reverted: row.get::<bool, _>("IsReverted").unwrap_or(false),
            reverted_at: row.get::<NaiveDateTime, _>("RevertedAt"),
            reverted_by: row.get::<Uuid, _>("RevertedBy"),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DeductionRequest {
    pub target_type: String,
    pub target_id: Option<i32>,
    pub target_cha_school: Option<i32>,
    pub points: i32,
    pub reason: String,
    pub match_id: Option<Uuid>,
}
