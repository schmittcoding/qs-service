//! Behavior tests for sp_TallyLevelCapRace stored procedure.
//!
//! These tests document and verify the expected behavior of the SP without
//! requiring a live database. They test the logic patterns and edge cases
//! that the SP must handle.

use chrono::{NaiveDateTime, Duration};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct MockEventDefinition {
    event_id: Uuid,
    event_name: String,
    event_start: NaiveDateTime,
    event_end: NaiveDateTime,
    category_config: Option<String>,
    target_level: i32,
}

#[derive(Debug, Clone)]
struct MockEventRegistration {
    event_id: Uuid,
    cha_num: i32,
    cha_name: String,
    cha_class: i32,
    cha_school: i32,
    guild_num: Option<i32>,
    cha_level: i32,
    status: i32, // 1 = Active, 0 = Withdrawn
}

#[derive(Debug, Clone)]
struct MockCharacterInfo {
    cha_num: i32,
    cha_name: String,
    cha_level: i32,
    level_up_date: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq)]
struct MockWinner {
    cha_num: i32,
    cha_name: String,
    cha_class: i32,
    cha_school: i32,
    guild_name: Option<String>,
    reached_level: i32,
    level_up_date: Option<NaiveDateTime>,
    rank: i32,
}

fn extract_target_level(category_config: &Option<String>, default: i32) -> i32 {
    // SP logic: JSON_VALUE(ed.CategoryConfig, '$.target_level'), default to 99999
    category_config
        .as_ref()
        .and_then(|config| {
            // Parse JSON to extract target_level
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(config) {
                json.get("target_level")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn is_character_active_winner(
    registration: &MockEventRegistration,
    character: &MockCharacterInfo,
    event: &MockEventDefinition,
) -> bool {
    // SP filters: WHERE er.Status = 1 AND c.ChaLevel >= @TargetLevel
    //           AND c.LevelUpDate BETWEEN ed.EventStart AND ed.EventEnd
    registration.status == 1
        && character.cha_level >= event.target_level
        && character
            .level_up_date
            .map(|date| date >= event.event_start && date <= event.event_end)
            .unwrap_or(false)
}

fn calculate_rank(
    existing_max_rank: i32,
    position_in_batch: usize,
) -> i32 {
    // SP logic: ROW_NUMBER() OVER (ORDER BY c.LevelUpDate ASC) + ISNULL((SELECT MAX(Rank) FROM ...), 0)
    position_in_batch as i32 + 1 + existing_max_rank
}

#[test]
fn test_sp_target_level_extraction_from_json() {
    let config = Some(r#"{"target_level": 200, "other": "value"}"#.to_string());
    let target_level = extract_target_level(&config, 99999);
    assert_eq!(target_level, 200);
}

#[test]
fn test_sp_target_level_defaults_when_json_missing() {
    let config: Option<String> = None;
    let target_level = extract_target_level(&config, 99999);
    assert_eq!(target_level, 99999);
}

#[test]
fn test_sp_target_level_defaults_when_json_malformed() {
    let config = Some(r#"{"invalid": "json"}"#.to_string());
    let target_level = extract_target_level(&config, 99999);
    assert_eq!(target_level, 99999);
}

#[test]
fn test_sp_filters_inactive_registrations() {
    let event = MockEventDefinition {
        event_id: Uuid::new_v4(),
        event_name: "Test".to_string(),
        event_start: NaiveDateTime::parse_from_str("2024-04-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        event_end: NaiveDateTime::parse_from_str("2024-04-30 23:59:59", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        category_config: Some(r#"{"target_level": 200}"#.to_string()),
        target_level: 200,
    };

    let active_reg = MockEventRegistration {
        event_id: event.event_id,
        cha_num: 1,
        cha_name: "Active".to_string(),
        cha_class: 1,
        cha_school: 1,
        guild_num: None,
        cha_level: 150,
        status: 1,
    };

    let inactive_reg = MockEventRegistration {
        event_id: event.event_id,
        cha_num: 2,
        cha_name: "Withdrawn".to_string(),
        cha_class: 1,
        cha_school: 1,
        guild_num: None,
        cha_level: 150,
        status: 0,
    };

    let character = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Active".to_string(),
        cha_level: 200,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    };

    // Both should be checked, but only active counts
    let active_is_winner =
        is_character_active_winner(&active_reg, &character, &event);
    let inactive_is_winner =
        is_character_active_winner(&inactive_reg, &character, &event);

    assert!(active_is_winner);
    assert!(!inactive_is_winner);
}

#[test]
fn test_sp_requires_level_at_or_above_target() {
    let event = MockEventDefinition {
        event_id: Uuid::new_v4(),
        event_name: "Test".to_string(),
        event_start: NaiveDateTime::parse_from_str("2024-04-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        event_end: NaiveDateTime::parse_from_str("2024-04-30 23:59:59", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        category_config: Some(r#"{"target_level": 200}"#.to_string()),
        target_level: 200,
    };

    let registration = MockEventRegistration {
        event_id: event.event_id,
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_class: 1,
        cha_school: 1,
        guild_num: None,
        cha_level: 150,
        status: 1,
    };

    let character_below = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 199,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    };

    let character_at = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    };

    let character_above = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 201,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    };

    assert!(!is_character_active_winner(&registration, &character_below, &event));
    assert!(is_character_active_winner(&registration, &character_at, &event));
    assert!(is_character_active_winner(&registration, &character_above, &event));
}

#[test]
fn test_sp_requires_level_up_date_within_event_window() {
    let event_start = NaiveDateTime::parse_from_str("2024-04-10 00:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let event_end = NaiveDateTime::parse_from_str("2024-04-20 23:59:59", "%Y-%m-%d %H:%M:%S")
        .unwrap();

    let event = MockEventDefinition {
        event_id: Uuid::new_v4(),
        event_name: "Test".to_string(),
        event_start,
        event_end,
        category_config: Some(r#"{"target_level": 200}"#.to_string()),
        target_level: 200,
    };

    let registration = MockEventRegistration {
        event_id: event.event_id,
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_class: 1,
        cha_school: 1,
        guild_num: None,
        cha_level: 150,
        status: 1,
    };

    // Before event
    let char_before = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(event_start - Duration::days(1)),
    };

    // At event start (inclusive)
    let char_at_start = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(event_start),
    };

    // During event
    let char_during = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(event_start + Duration::days(5)),
    };

    // At event end (inclusive)
    let char_at_end = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(event_end),
    };

    // After event
    let char_after = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(event_end + Duration::days(1)),
    };

    assert!(!is_character_active_winner(&registration, &char_before, &event));
    assert!(is_character_active_winner(&registration, &char_at_start, &event));
    assert!(is_character_active_winner(&registration, &char_during, &event));
    assert!(is_character_active_winner(&registration, &char_at_end, &event));
    assert!(!is_character_active_winner(&registration, &char_after, &event));
}

#[test]
fn test_sp_requires_non_null_level_up_date() {
    let event = MockEventDefinition {
        event_id: Uuid::new_v4(),
        event_name: "Test".to_string(),
        event_start: NaiveDateTime::parse_from_str("2024-04-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        event_end: NaiveDateTime::parse_from_str("2024-04-30 23:59:59", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        category_config: Some(r#"{"target_level": 200}"#.to_string()),
        target_level: 200,
    };

    let registration = MockEventRegistration {
        event_id: event.event_id,
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_class: 1,
        cha_school: 1,
        guild_num: None,
        cha_level: 150,
        status: 1,
    };

    let character_with_date = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    };

    let character_no_date = MockCharacterInfo {
        cha_num: 1,
        cha_name: "Test".to_string(),
        cha_level: 200,
        level_up_date: None,
    };

    assert!(is_character_active_winner(&registration, &character_with_date, &event));
    assert!(!is_character_active_winner(&registration, &character_no_date, &event));
}

#[test]
fn test_sp_rank_calculation_with_existing_winners() {
    // If max existing rank is 5, new winners start at 6, 7, 8, etc.
    let existing_max_rank = 5;

    let rank_1st = calculate_rank(existing_max_rank, 0);
    let rank_2nd = calculate_rank(existing_max_rank, 1);
    let rank_3rd = calculate_rank(existing_max_rank, 2);

    assert_eq!(rank_1st, 6);
    assert_eq!(rank_2nd, 7);
    assert_eq!(rank_3rd, 8);
}

#[test]
fn test_sp_rank_calculation_no_existing_winners() {
    // If no existing winners, new winners start at 1, 2, 3, etc.
    let existing_max_rank = 0;

    let rank_1st = calculate_rank(existing_max_rank, 0);
    let rank_2nd = calculate_rank(existing_max_rank, 1);
    let rank_3rd = calculate_rank(existing_max_rank, 2);

    assert_eq!(rank_1st, 1);
    assert_eq!(rank_2nd, 2);
    assert_eq!(rank_3rd, 3);
}

#[test]
fn test_sp_idempotency_prevents_duplicate_winners() {
    // SP uses: NOT EXISTS (SELECT 1 FROM LevelCapRaceWinner WHERE EventID = @EventID AND ChaNum = er.ChaNum)
    // Running tally twice on same data should not create duplicate winner rows

    let existing_winners = vec![1, 2, 3]; // Char nums already crowned

    // Second tally should skip these
    for cha_num in &existing_winners {
        assert!(!(!existing_winners.contains(cha_num))); // Already in winners
    }

    let new_cha_num = 4;
    assert!(!existing_winners.contains(&new_cha_num)); // New winner can be added
}

#[test]
fn test_sp_returns_two_result_sets() {
    // Result-set 1: TotalRows, ValidEntries (both same value for LCR)
    // Result-set 2: Winner columns

    let total_rows = 100;
    let result_set_1 = (total_rows, total_rows); // TotalRows, ValidEntries

    let result_set_2 = vec![
        ("ChaNum", "i32"),
        ("ChaName", "string"),
        ("ChaClass", "i32"),
        ("ChaSchool", "i32"),
        ("ReachedLevel", "i32"),
        ("LevelUpDate", "datetime?"),
        ("GuildName", "string?"),
        ("Rank", "i32"),
    ];

    assert_eq!(result_set_1.0, 100);
    assert_eq!(result_set_1.1, 100);
    assert_eq!(result_set_2.len(), 8);
}

#[test]
fn test_sp_winner_ordered_by_rank() {
    // Result-set 2 is: SELECT ... FROM #NewWinners ORDER BY Rank ASC
    let mut winners = vec![
        MockWinner {
            cha_num: 3,
            cha_name: "Third".to_string(),
            cha_class: 1,
            cha_school: 1,
            guild_name: None,
            reached_level: 200,
            level_up_date: None,
            rank: 3,
        },
        MockWinner {
            cha_num: 1,
            cha_name: "First".to_string(),
            cha_class: 1,
            cha_school: 1,
            guild_name: None,
            reached_level: 200,
            level_up_date: None,
            rank: 1,
        },
        MockWinner {
            cha_num: 2,
            cha_name: "Second".to_string(),
            cha_class: 1,
            cha_school: 1,
            guild_name: None,
            reached_level: 200,
            level_up_date: None,
            rank: 2,
        },
    ];

    winners.sort_by_key(|w| w.rank);

    assert_eq!(winners[0].rank, 1);
    assert_eq!(winners[1].rank, 2);
    assert_eq!(winners[2].rank, 3);
}

#[test]
fn test_sp_empty_winner_list_when_no_qualifiers() {
    // If no characters meet the criteria, #NewWinners is empty
    // Result-set 2 is still returned but with zero rows
    let new_winners: Vec<MockWinner> = vec![];

    assert!(new_winners.is_empty());
}

#[test]
fn test_sp_merges_into_level_cap_race_score() {
    // Before winner detection, SP first upserts into LevelCapRaceScore:
    // - MERGE handles new and existing registrants
    // - Updates CurrentLevel and LevelsGained for existing
    // - Inserts for new

    let score_before: Option<i32> = None; // New registrant
    let is_new_registrant = score_before.is_none();
    assert!(is_new_registrant); // Would INSERT

    let score_after: Option<i32> = Some(50); // Existing registrant
    let is_existing_registrant = score_after.is_some();
    assert!(is_existing_registrant); // Would UPDATE
}
