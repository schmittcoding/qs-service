use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tiberius::Row;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct LevelCapRaceWinnerNotice {
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub reached_level: i32,
    pub level_up_date: Option<NaiveDateTime>,
    pub guild_name: Option<String>,
    pub rank: i32,
}

impl LevelCapRaceWinnerNotice {
    pub fn from_row(row: &Row) -> Self {
        Self {
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(String::new, |s| s.to_string()),
            cha_class: row.get::<i32, _>("ChaClass").unwrap_or_default(),
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or(0),
            reached_level: row.get::<i32, _>("ReachedLevel").unwrap_or_default(),
            level_up_date: row.get::<NaiveDateTime, _>("LevelUpDate"),
            guild_name: row.get::<&str, _>("GuildName").map(|s| s.to_string()),
            rank: row.get::<i32, _>("Rank").unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_cap_race_winner_notice_serialization_roundtrip() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 123,
            cha_name: "TestChar".to_string(),
            cha_class: 1,
            cha_school: 2,
            reached_level: 200,
            level_up_date: Some(
                NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("TestGuild".to_string()),
            rank: 1,
        };

        let json = serde_json::to_string(&winner).expect("Serialization failed");
        let deserialized: LevelCapRaceWinnerNotice =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(winner, deserialized);
    }

    #[test]
    fn test_level_cap_race_winner_notice_with_nullable_fields() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 456,
            cha_name: "NoGuildChar".to_string(),
            cha_class: 2,
            cha_school: 1,
            reached_level: 150,
            level_up_date: None,
            guild_name: None,
            rank: 5,
        };

        let json = serde_json::to_string(&winner).expect("Serialization failed");
        let deserialized: LevelCapRaceWinnerNotice =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(winner, deserialized);
        assert!(deserialized.level_up_date.is_none());
        assert!(deserialized.guild_name.is_none());
    }

    #[test]
    fn test_level_cap_race_winner_notice_json_schema() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 789,
            cha_name: "SchemaTest".to_string(),
            cha_class: 3,
            cha_school: 0,
            reached_level: 300,
            level_up_date: Some(
                NaiveDateTime::parse_from_str("2024-04-21 10:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("SchemaGuild".to_string()),
            rank: 1,
        };

        let json_value = serde_json::to_value(&winner).expect("to_value failed");

        assert!(json_value["cha_num"].is_number());
        assert!(json_value["cha_name"].is_string());
        assert!(json_value["cha_class"].is_number());
        assert!(json_value["cha_school"].is_number());
        assert!(json_value["reached_level"].is_number());
        assert!(json_value["level_up_date"].is_string());
        assert!(json_value["guild_name"].is_string());
        assert!(json_value["rank"].is_number());

        assert_eq!(json_value["cha_num"].as_i64(), Some(789));
        assert_eq!(json_value["cha_name"].as_str(), Some("SchemaTest"));
        assert_eq!(json_value["rank"].as_i64(), Some(1));
    }

    #[test]
    fn test_level_cap_race_winner_notice_defaults_on_missing_fields() {
        // Simulate a scenario where optional fields are serialized without values
        let json_with_null = r#"
        {
            "cha_num": 999,
            "cha_name": "TestCharacter",
            "cha_class": 1,
            "cha_school": 2,
            "reached_level": 100,
            "level_up_date": null,
            "guild_name": null,
            "rank": 1
        }
        "#;

        let winner: LevelCapRaceWinnerNotice =
            serde_json::from_str(json_with_null).expect("Deserialization failed");

        assert_eq!(winner.cha_num, 999);
        assert_eq!(winner.cha_name, "TestCharacter");
        assert!(winner.level_up_date.is_none());
        assert!(winner.guild_name.is_none());
    }
}
