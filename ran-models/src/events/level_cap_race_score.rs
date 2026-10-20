use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LevelCapRaceScore {
    pub rank: i64,
    pub event_id: Uuid,
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: Option<i32>,
    pub guild_num: Option<i32>,
    pub guild_name: Option<String>,
    pub start_level: i32,
    pub current_level: i32,
    pub levels_gained: i32,
    /// The timestamp when the character last leveled up
    pub level_up_date: Option<NaiveDateTime>,
}

impl LevelCapRaceScore {
    pub fn from_row(row: &Row) -> Self {
        Self {
            rank: row.get::<i64, _>("Rank").unwrap_or(0),
            event_id: row.get::<Uuid, _>("EventID").unwrap_or_else(Uuid::nil),
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
            level_up_date: row.get::<NaiveDateTime, _>("LevelUpDate"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_cap_race_score_serialization_with_level_up_date() {
        let score = LevelCapRaceScore {
            rank: 1,
            event_id: Uuid::nil(),
            cha_num: 12345,
            cha_name: "TestChar".to_string(),
            cha_class: 1,
            cha_school: Some(5),
            guild_num: Some(999),
            guild_name: Some("TestGuild".to_string()),
            start_level: 100,
            current_level: 120,
            levels_gained: 20,
            level_up_date: Some(NaiveDateTime::from_timestamp_opt(1609459200, 0).unwrap()),
        };

        let json = serde_json::to_value(&score).expect("should serialize");
        assert_eq!(json["rank"], 1);
        assert_eq!(json["cha_name"], "TestChar");
        assert_eq!(json["current_level"], 120);
        assert_eq!(json["levels_gained"], 20);
        // Verify level_up_date is present in JSON
        assert!(json.get("level_up_date").is_some());
        assert!(!json["level_up_date"].is_null());
    }

    #[test]
    fn test_level_cap_race_score_serialization_null_level_up_date() {
        let score = LevelCapRaceScore {
            rank: 2,
            event_id: Uuid::nil(),
            cha_num: 54321,
            cha_name: "AnotherChar".to_string(),
            cha_class: 2,
            cha_school: None,
            guild_num: None,
            guild_name: None,
            start_level: 80,
            current_level: 85,
            levels_gained: 5,
            level_up_date: None,
        };

        let json = serde_json::to_value(&score).expect("should serialize");
        assert_eq!(json["rank"], 2);
        assert_eq!(json["cha_name"], "AnotherChar");
        // Verify level_up_date is null in JSON
        assert!(json.get("level_up_date").is_some());
        assert!(json["level_up_date"].is_null());
    }

    #[test]
    fn test_level_cap_race_score_deserialization_with_level_up_date() {
        let json_str = r#"{
            "rank": 3,
            "event_id": "00000000-0000-0000-0000-000000000000",
            "cha_num": 11111,
            "cha_name": "Deserialized",
            "cha_class": 3,
            "cha_school": 10,
            "guild_num": 888,
            "guild_name": "SomeGuild",
            "start_level": 90,
            "current_level": 110,
            "levels_gained": 20,
            "level_up_date": "2021-01-01T00:00:00"
        }"#;

        let score: LevelCapRaceScore = serde_json::from_str(json_str).expect("should deserialize");
        assert_eq!(score.rank, 3);
        assert_eq!(score.cha_name, "Deserialized");
        assert_eq!(score.cha_class, 3);
        assert_eq!(score.current_level, 110);
        assert!(score.level_up_date.is_some());
        assert_eq!(
            score.level_up_date,
            NaiveDateTime::from_timestamp_opt(1609459200, 0)
        );
    }

    #[test]
    fn test_level_cap_race_score_deserialization_null_level_up_date() {
        let json_str = r#"{
            "rank": 4,
            "event_id": "00000000-0000-0000-0000-000000000000",
            "cha_num": 22222,
            "cha_name": "NoDate",
            "cha_class": 1,
            "cha_school": null,
            "guild_num": null,
            "guild_name": null,
            "start_level": 100,
            "current_level": 101,
            "levels_gained": 1,
            "level_up_date": null
        }"#;

        let score: LevelCapRaceScore = serde_json::from_str(json_str).expect("should deserialize");
        assert_eq!(score.rank, 4);
        assert_eq!(score.cha_name, "NoDate");
        assert!(score.level_up_date.is_none());
    }

    #[test]
    fn test_level_cap_race_score_serde_roundtrip() {
        let original = LevelCapRaceScore {
            rank: 5,
            event_id: Uuid::new_v4(),
            cha_num: 99999,
            cha_name: "RoundTrip".to_string(),
            cha_class: 2,
            cha_school: Some(3),
            guild_num: Some(777),
            guild_name: Some("Guild123".to_string()),
            start_level: 95,
            current_level: 115,
            levels_gained: 20,
            level_up_date: Some(NaiveDateTime::from_timestamp_opt(1577836800, 0).unwrap()),
        };

        let json = serde_json::to_string(&original).expect("serialize");
        let deserialized: LevelCapRaceScore = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(original.rank, deserialized.rank);
        assert_eq!(original.cha_num, deserialized.cha_num);
        assert_eq!(original.cha_name, deserialized.cha_name);
        assert_eq!(original.cha_class, deserialized.cha_class);
        assert_eq!(original.current_level, deserialized.current_level);
        assert_eq!(original.levels_gained, deserialized.levels_gained);
        assert_eq!(original.level_up_date, deserialized.level_up_date);
    }

    #[test]
    fn test_level_cap_race_score_with_all_optional_fields() {
        let score = LevelCapRaceScore {
            rank: 6,
            event_id: Uuid::new_v4(),
            cha_num: 55555,
            cha_name: "FullOptional".to_string(),
            cha_class: 3,
            cha_school: Some(7),
            guild_num: Some(666),
            guild_name: Some("OptionalGuild".to_string()),
            start_level: 50,
            current_level: 100,
            levels_gained: 50,
            level_up_date: Some(NaiveDateTime::from_timestamp_opt(1609459200, 0).unwrap()),
        };

        let json = serde_json::to_value(&score).expect("serialize");
        assert!(json["cha_school"].is_number());
        assert!(json["guild_num"].is_number());
        assert!(json["guild_name"].is_string());
        assert!(!json["level_up_date"].is_null());
    }

    #[test]
    fn test_level_cap_race_score_with_all_none_optional_fields() {
        let score = LevelCapRaceScore {
            rank: 7,
            event_id: Uuid::new_v4(),
            cha_num: 77777,
            cha_name: "NoneOptional".to_string(),
            cha_class: 0,
            cha_school: None,
            guild_num: None,
            guild_name: None,
            start_level: 1,
            current_level: 1,
            levels_gained: 0,
            level_up_date: None,
        };

        let json = serde_json::to_value(&score).expect("serialize");
        assert!(json["cha_school"].is_null());
        assert!(json["guild_num"].is_null());
        assert!(json["guild_name"].is_null());
        assert!(json["level_up_date"].is_null());
    }
}
