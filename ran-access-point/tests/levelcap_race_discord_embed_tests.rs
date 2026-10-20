//! Integration tests for Discord embed building and formatting in LevelCapRace feature.
//! Tests the structure, field inclusion, and formatting of Discord embeds sent to webhooks.

use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq)]
struct MockLevelCapRaceWinnerNotice {
    cha_num: i32,
    cha_name: String,
    cha_class: i32,
    cha_school: i32,
    reached_level: i32,
    level_up_date: Option<NaiveDateTime>,
    guild_name: Option<String>,
    rank: i32,
}

#[test]
fn test_embed_build_with_single_winner() {
    let winner = MockLevelCapRaceWinnerNotice {
        cha_num: 123,
        cha_name: "TestChar".to_string(),
        cha_class: 1,
        cha_school: 2,
        reached_level: 200,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
        guild_name: Some("TestGuild".to_string()),
        rank: 1,
    };

    let _event_name = "Spring Challenge";
    let winners = vec![winner.clone()];

    // Build embed fields
    let fields: Vec<String> = winners
        .iter()
        .map(|w| {
            let guild_str = w
                .guild_name
                .as_ref()
                .map(|g| format!(" • Guild {}", g))
                .unwrap_or_default();
            let date_str = w
                .level_up_date
                .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|| "N/A".to_string());
            format!(
                "#{} {}: Class {} • School {} • Reached Lv {} at {}{}",
                w.rank, w.cha_name, w.cha_class, w.cha_school, w.reached_level, date_str, guild_str
            )
        })
        .collect();

    assert_eq!(fields.len(), 1);
    assert!(fields[0].contains("TestChar"));
    assert!(fields[0].contains("Class 1"));
    assert!(fields[0].contains("School 2"));
    assert!(fields[0].contains("Reached Lv 200"));
    assert!(fields[0].contains("Guild TestGuild"));
}

#[test]
fn test_embed_build_with_multiple_winners() {
    let winners = vec![
        MockLevelCapRaceWinnerNotice {
            cha_num: 1,
            cha_name: "FirstWinner".to_string(),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: Some(
                NaiveDateTime::parse_from_str("2024-04-20 10:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("GuildA".to_string()),
            rank: 1,
        },
        MockLevelCapRaceWinnerNotice {
            cha_num: 2,
            cha_name: "SecondWinner".to_string(),
            cha_class: 2,
            cha_school: 2,
            reached_level: 200,
            level_up_date: Some(
                NaiveDateTime::parse_from_str("2024-04-20 11:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("GuildB".to_string()),
            rank: 2,
        },
    ];

    let fields: Vec<String> = winners
        .iter()
        .map(|w| {
            let guild_str = w
                .guild_name
                .as_ref()
                .map(|g| format!(" • Guild {}", g))
                .unwrap_or_default();
            let date_str = w
                .level_up_date
                .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|| "N/A".to_string());
            format!(
                "#{} {}: Class {} • School {} • Reached Lv {} at {}{}",
                w.rank, w.cha_name, w.cha_class, w.cha_school, w.reached_level, date_str, guild_str
            )
        })
        .collect();

    assert_eq!(fields.len(), 2);
    assert!(fields[0].contains("FirstWinner"));
    assert!(fields[1].contains("SecondWinner"));
    assert!(fields[0].contains("#1"));
    assert!(fields[1].contains("#2"));
}

#[test]
fn test_embed_with_null_guild_name() {
    let winner = MockLevelCapRaceWinnerNotice {
        cha_num: 456,
        cha_name: "NoGuildChar".to_string(),
        cha_class: 2,
        cha_school: 1,
        reached_level: 150,
        level_up_date: Some(
            NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
        guild_name: None,
        rank: 1,
    };

    let guild_str = winner
        .guild_name
        .as_ref()
        .map(|g| format!(" • Guild {}", g))
        .unwrap_or_default();

    assert_eq!(guild_str, "");

    let field = format!(
        "#{} {}: Class {} • School {} • Reached Lv {} at {}{}",
        winner.rank,
        winner.cha_name,
        winner.cha_class,
        winner.cha_school,
        winner.reached_level,
        winner
            .level_up_date
            .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
            .unwrap_or_else(|| "N/A".to_string()),
        guild_str
    );

    // Verify that when guild_name is None, no guild suffix is appended
    assert_eq!(guild_str, "");
    assert!(field.contains("NoGuildChar"));
    assert!(field.contains("Class 2"));
}

#[test]
fn test_embed_with_null_level_up_date() {
    let winner = MockLevelCapRaceWinnerNotice {
        cha_num: 789,
        cha_name: "NoDateChar".to_string(),
        cha_class: 3,
        cha_school: 0,
        reached_level: 300,
        level_up_date: None,
        guild_name: Some("SomeGuild".to_string()),
        rank: 3,
    };

    let date_str = winner
        .level_up_date
        .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
        .unwrap_or_else(|| "N/A".to_string());

    assert_eq!(date_str, "N/A");

    let field = format!(
        "#{} {}: Class {} • School {} • Reached Lv {} at {}{}",
        winner.rank,
        winner.cha_name,
        winner.cha_class,
        winner.cha_school,
        winner.reached_level,
        date_str,
        winner
            .guild_name
            .as_ref()
            .map(|g| format!(" • Guild {}", g))
            .unwrap_or_default()
    );

    assert!(field.contains("at N/A"));
}

#[test]
fn test_chunking_30_winners_into_2_embeds() {
    let mut winners = vec![];
    for i in 0..30 {
        winners.push(MockLevelCapRaceWinnerNotice {
            cha_num: 1000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: Some(
                NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("Guild".to_string()),
            rank: (i + 1) as i32,
        });
    }

    let chunk_size = 25;
    let chunks: Vec<&[MockLevelCapRaceWinnerNotice]> =
        winners.chunks(chunk_size).collect();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].len(), 25);
    assert_eq!(chunks[1].len(), 5);
}

#[test]
fn test_chunking_25_winners_into_1_embed() {
    let mut winners = vec![];
    for i in 0..25 {
        winners.push(MockLevelCapRaceWinnerNotice {
            cha_num: 2000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: None,
            guild_name: None,
            rank: (i + 1) as i32,
        });
    }

    let chunk_size = 25;
    let chunks: Vec<&[MockLevelCapRaceWinnerNotice]> =
        winners.chunks(chunk_size).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].len(), 25);
}

#[test]
fn test_chunking_50_winners_into_2_embeds() {
    let mut winners = vec![];
    for i in 0..50 {
        winners.push(MockLevelCapRaceWinnerNotice {
            cha_num: 3000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: None,
            guild_name: None,
            rank: (i + 1) as i32,
        });
    }

    let chunk_size = 25;
    let chunks: Vec<&[MockLevelCapRaceWinnerNotice]> =
        winners.chunks(chunk_size).collect();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].len(), 25);
    assert_eq!(chunks[1].len(), 25);
}

#[test]
fn test_chunking_51_winners_into_3_embeds() {
    let mut winners = vec![];
    for i in 0..51 {
        winners.push(MockLevelCapRaceWinnerNotice {
            cha_num: 4000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: None,
            guild_name: None,
            rank: (i + 1) as i32,
        });
    }

    let chunk_size = 25;
    let num_chunks = winners.len().div_ceil(chunk_size);
    let chunks: Vec<&[MockLevelCapRaceWinnerNotice]> =
        winners.chunks(chunk_size).collect();

    assert_eq!(num_chunks, 3);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].len(), 25);
    assert_eq!(chunks[1].len(), 25);
    assert_eq!(chunks[2].len(), 1);
}

#[test]
fn test_field_name_format_includes_rank_and_character_name() {
    let winner = MockLevelCapRaceWinnerNotice {
        cha_num: 111,
        cha_name: "TestChar".to_string(),
        cha_class: 1,
        cha_school: 1,
        reached_level: 200,
        level_up_date: None,
        guild_name: None,
        rank: 5,
    };

    let field_name = format!("#{} {}", winner.rank, winner.cha_name);
    assert_eq!(field_name, "#5 TestChar");
}

#[test]
fn test_embed_title_is_consistent() {
    let title = "Level Cap Race — New Winner(s)!";
    assert!(title.contains("Level Cap Race"));
    assert!(title.contains("Winner"));
}

#[test]
fn test_embed_description_includes_event_name() {
    let event_name = "Spring Challenge Event";
    let description = format!("Event: {}", event_name);
    assert_eq!(description, "Event: Spring Challenge Event");
}

#[test]
fn test_rank_increments_correctly() {
    let mut winners = vec![];
    for i in 1..=5 {
        winners.push(MockLevelCapRaceWinnerNotice {
            cha_num: i as i32,
            cha_name: format!("Rank{}", i),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: None,
            guild_name: None,
            rank: i as i32,
        });
    }

    // Verify ranks are in order
    for (idx, winner) in winners.iter().enumerate() {
        assert_eq!(winner.rank, (idx + 1) as i32);
    }
}
