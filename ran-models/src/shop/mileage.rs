use chrono::{DateTime, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use tiberius::numeric::Numeric;

use crate::util::naive_to_local;

#[derive(Serialize)]
pub struct MileageConfig {
    pub config_id:      i32,
    pub mileage_rate:   f64,
    pub is_active:      bool,
    pub effective_from: DateTime<Local>,
    pub effective_to:   Option<DateTime<Local>>,
    pub description:    Option<String>,
    pub updated_by:     String,
    pub created_at:     DateTime<Local>,
    pub updated_at:     DateTime<Local>,
}

impl MileageConfig {
    pub fn from_row(row: &Row) -> Self {
        Self {
            config_id: row.get::<i32, _>("ConfigID").unwrap_or_default(),
            mileage_rate: row
                .try_get::<Numeric, _>("MileageRate")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            is_active: row.get::<bool, _>("IsActive").unwrap_or_default(),
            effective_from: row
                .get::<NaiveDateTime, _>("EffectiveFrom")
                .map(naive_to_local)
                .unwrap_or_default(),
            effective_to: row
                .try_get::<NaiveDateTime, _>("EffectiveTo")
                .ok()
                .flatten()
                .map(naive_to_local),
            description: row
                .try_get::<&str, _>("Description")
                .ok()
                .flatten()
                .map(|s| s.to_string()),
            updated_by: row
                .get::<&str, _>("UpdatedBy")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            created_at: row
                .get::<NaiveDateTime, _>("CreatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
            updated_at: row
                .get::<NaiveDateTime, _>("UpdatedAt")
                .map(naive_to_local)
                .unwrap_or_default(),
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateMileageConfigRequest {
    pub mileage_rate:   f64,
    pub effective_from: Option<String>,
    pub effective_to:   Option<String>,
    pub description:    Option<String>,
}
