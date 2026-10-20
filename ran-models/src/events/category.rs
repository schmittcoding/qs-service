use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Gvg,
    Koth,
    LevelCapRace,
    SpeedRun,
    GvgKoth,
}

impl EventCategory {
    /// Parse from a DB VARCHAR column (case-insensitive).
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gvg" => Some(Self::Gvg),
            "koth" => Some(Self::Koth),
            "levelcaprace" | "level_cap_race" => Some(Self::LevelCapRace),
            "speedrun" | "speed_run" => Some(Self::SpeedRun),
            "gvgkoth" | "gvg_koth" | "gvg+koth" => Some(Self::GvgKoth),
            _ => None,
        }
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::GvgKoth)
    }

    /// Whether this category runs on a MatchSchedule (requires at least one match to activate/tally).
    pub fn is_match_based(&self) -> bool {
        matches!(self, Self::Gvg | Self::Koth | Self::GvgKoth)
    }

    pub fn selectable() -> Vec<EventCategory> {
        vec![Self::Gvg, Self::Koth, Self::LevelCapRace, Self::SpeedRun]
    }
}

impl fmt::Display for EventCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gvg => write!(f, "GvG"),
            Self::Koth => write!(f, "KOTH"),
            Self::LevelCapRace => write!(f, "LevelCapRace"),
            Self::SpeedRun => write!(f, "SpeedRun"),
            Self::GvgKoth => write!(f, "GvGKoth"),
        }
    }
}

impl FromStr for EventCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_db_str(s).ok_or_else(|| format!("Unknown event category: {}", s))
    }
}
