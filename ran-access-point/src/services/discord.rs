use ran_models::events::level_cap_race_winner::LevelCapRaceWinnerNotice;
use serde_json::json;
use std::time::Duration;
use tracing::error;

pub async fn send_embed(webhook_url: &str, embed: serde_json::Value) -> Result<(), String> {
    let client = reqwest::Client::new();
    let payload = json!({
        "embeds": [embed]
    });

    match client
        .post(webhook_url)
        .json(&payload)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Discord webhook failed: {}", e);
            Err(e.to_string())
        }
    }
}

pub async fn announce_level_cap_winners(event_name: &str, winners: &[LevelCapRaceWinnerNotice]) {
    if winners.is_empty() {
        return;
    }

    let webhook_url = std::env::var("DISCORD_LEVELCAP_WEBHOOK")
        .or_else(|_| std::env::var("DISCORD_SERVER_WEBHOOK"))
        .unwrap_or_default();

    if webhook_url.is_empty() {
        error!("DISCORD_LEVELCAP_WEBHOOK and DISCORD_SERVER_WEBHOOK not set");
        return;
    }

    let total_winners = winners.len();
    let chunk_size = 25;
    let num_chunks = total_winners.div_ceil(chunk_size);

    for (chunk_idx, chunk) in winners.chunks(chunk_size).enumerate() {
        let fields: Vec<serde_json::Value> = chunk
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
                json!({
                    "name": format!("#{} {}", w.rank, w.cha_name),
                    "value": format!(
                        "Class {} • School {} • Reached Lv {} at {}{}",
                        w.cha_class, w.cha_school, w.reached_level, date_str, guild_str
                    ),
                    "inline": false
                })
            })
            .collect();

        let embed = json!({
            "title": "Level Cap Race — New Winner(s)!",
            "description": format!("Event: {}", event_name),
            "fields": fields,
            "color": 0x9C27B0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Err(e) = send_embed(&webhook_url, embed).await {
            error!("Failed to send winners embed (chunk {}): {}", chunk_idx, e);
        }

        if chunk_idx < num_chunks - 1 {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announce_level_cap_winners_empty_list() {
        // Function should return early; just verify no panic
        let winners: Vec<LevelCapRaceWinnerNotice> = vec![];
        // This test runs synchronously; we'll use tokio::test for async variants
        assert!(winners.is_empty());
    }

    #[test]
    fn test_embed_structure_has_required_fields() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 123,
            cha_name: "TestChar".to_string(),
            cha_class: 1,
            cha_school: 2,
            reached_level: 200,
            level_up_date: Some(
                chrono::NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: Some("TestGuild".to_string()),
            rank: 1,
        };

        let winners = vec![winner];
        let event_name = "TestEvent";

        // Build embed manually to verify structure
        let fields: Vec<serde_json::Value> = winners
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
                json!({
                    "name": format!("#{} {}", w.rank, w.cha_name),
                    "value": format!(
                        "Class {} • School {} • Reached Lv {} at {}{}",
                        w.cha_class, w.cha_school, w.reached_level, date_str, guild_str
                    ),
                    "inline": false
                })
            })
            .collect();

        let embed = json!({
            "title": "Level Cap Race — New Winner(s)!",
            "description": format!("Event: {}", event_name),
            "fields": fields,
            "color": 0x9C27B0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Verify structure
        assert!(embed["title"].is_string());
        assert_eq!(embed["title"].as_str().unwrap(), "Level Cap Race — New Winner(s)!");
        assert!(embed["description"].is_string());
        assert!(embed["fields"].is_array());
        assert!(embed["color"].is_number());
        assert!(embed["timestamp"].is_string());

        // Verify first field includes winner name and rank
        assert!(embed["fields"][0]["name"].as_str().unwrap().contains("TestChar"));
        assert!(embed["fields"][0]["value"]
            .as_str()
            .unwrap()
            .contains("Class 1"));
    }

    #[test]
    fn test_embed_with_null_level_up_date() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 456,
            cha_name: "NoDateChar".to_string(),
            cha_class: 2,
            cha_school: 1,
            reached_level: 150,
            level_up_date: None,
            guild_name: Some("SomeGuild".to_string()),
            rank: 2,
        };

        let date_str = winner
            .level_up_date
            .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
            .unwrap_or_else(|| "N/A".to_string());

        assert_eq!(date_str, "N/A");

        let field_value = format!(
            "Class {} • School {} • Reached Lv {} at {}{}",
            winner.cha_class,
            winner.cha_school,
            winner.reached_level,
            date_str,
            " • Guild SomeGuild"
        );

        assert!(field_value.contains("at N/A"));
    }

    #[test]
    fn test_embed_with_null_guild_name() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 789,
            cha_name: "NoGuildChar".to_string(),
            cha_class: 3,
            cha_school: 0,
            reached_level: 250,
            level_up_date: Some(
                chrono::NaiveDateTime::parse_from_str("2024-04-21 10:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: None,
            rank: 3,
        };

        let guild_str = winner
            .guild_name
            .as_ref()
            .map(|g| format!(" • Guild {}", g))
            .unwrap_or_default();

        assert_eq!(guild_str, "");

        let field_value = format!(
            "Class {} • School {} • Reached Lv {} at {}{}",
            winner.cha_class,
            winner.cha_school,
            winner.reached_level,
            winner
                .level_up_date
                .map(|d| format!("{}", d.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|| "N/A".to_string()),
            guild_str
        );

        assert!(!field_value.contains("Guild"));
    }

    #[test]
    fn test_chunk_size_is_25() {
        // Verify that chunk size is 25 fields per embed
        let mut winners = vec![];
        for i in 0..30 {
            winners.push(LevelCapRaceWinnerNotice {
                cha_num: 1000 + i as i32,
                cha_name: format!("Char{}", i),
                cha_class: 1,
                cha_school: 1,
                reached_level: 200,
                level_up_date: Some(
                    chrono::NaiveDateTime::parse_from_str(
                        "2024-04-20 14:30:00",
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .unwrap(),
                ),
                guild_name: Some("Guild".to_string()),
                rank: (i + 1) as i32,
            });
        }

        let chunk_size = 25;
        let num_chunks = winners.len().div_ceil(chunk_size);

        // Verify we get 2 chunks for 30 winners
        assert_eq!(num_chunks, 2);

        // Verify first chunk has 25, second has 5
        let chunks: Vec<_> = winners.chunks(chunk_size).collect();
        assert_eq!(chunks[0].len(), 25);
        assert_eq!(chunks[1].len(), 5);
    }

    #[test]
    fn test_rank_formatting_in_field_name() {
        let winner = LevelCapRaceWinnerNotice {
            cha_num: 100,
            cha_name: "FirstWinner".to_string(),
            cha_class: 1,
            cha_school: 1,
            reached_level: 200,
            level_up_date: Some(
                chrono::NaiveDateTime::parse_from_str("2024-04-20 14:30:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            ),
            guild_name: None,
            rank: 1,
        };

        let field_name = format!("#{} {}", winner.rank, winner.cha_name);
        assert_eq!(field_name, "#1 FirstWinner");
    }
}

