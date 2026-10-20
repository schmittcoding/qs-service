use chrono::Duration;
use ran_db_core::webmanager::WebDB;
use ran_models::events::category::EventCategory;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tracing::{error, info, warn};

use crate::states::AppState;

/// Spawns background tasks:
/// 1. Match-based tally every 30s (GvG, KOTH)
/// 2. Level cap race tally every 30 minutes
pub fn spawn_tally_scheduler(web_db: Arc<WebDB>, app_state: Arc<AppState>) {
    let web_db_matches = web_db.clone();
    tokio::spawn(async move {
        info!("Tally scheduler started — polling matches every 30s");

        let poll_interval = StdDuration::from_secs(30);

        loop {
            tokio::time::sleep(poll_interval).await;

            match run_pending_tally(&web_db_matches).await {
                Ok(true) => info!("Tally completed successfully"),
                Ok(false) => {} // No pending matches — normal idle
                Err(e) => error!("Tally scheduler error: {:?}", e),
            }
        }
    });

    let web_db_lcr = web_db.clone();
    let app_state_lcr = app_state.clone();
    tokio::spawn(async move {
        info!("Level cap race tally scheduler started — running every 30 minutes");

        let poll_interval = StdDuration::from_secs(1800); // 30 minutes

        loop {
            tokio::time::sleep(poll_interval).await;

            match run_level_cap_race_tally(&web_db_lcr, &app_state_lcr).await {
                Ok(count) if count > 0 => info!("Tallied {} level cap race events", count),
                Ok(_) => {} // No active LCR events — normal idle
                Err(e) => error!("Level cap race tally error: {:?}", e),
            }
        }
    });
}

/// Check for a pending match and run the tally if one is found.
/// Returns Ok(true) if a tally was executed, Ok(false) if nothing to do.
async fn run_pending_tally(web_db: &WebDB) -> anyhow::Result<bool> {
    let match_info = match web_db.get_next_tally_match().await? {
        Some(m) => m,
        None => return Ok(false),
    };

    let match_id = match_info.match_id;
    let event_id = match_info.event_id;
    let match_label = match_info.match_label.clone();

    info!(
        "Found match ready for tally: {} ({})",
        match_id,
        match_label.as_deref().unwrap_or("no label")
    );

    // Mark as Processing
    web_db.set_match_processing(match_id).await?;

    // Load event to get category for dispatch
    let event = match web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            error!("Event {} not found for match {}", event_id, match_id);
            if let Err(fail_err) = web_db.set_match_failed(match_id).await {
                error!("Failed to mark match as failed: {:?}", fail_err);
            }
            anyhow::bail!("Event not found for tally");
        }
        Err(e) => {
            error!("Failed to load event for match {}: {:?}", match_id, e);
            if let Err(fail_err) = web_db.set_match_failed(match_id).await {
                error!("Failed to mark match as failed: {:?}", fail_err);
            }
            return Err(e);
        }
    };

    info!("Tally category: {} for event {}", event.event_category, event_id);

    // Execute the category-dispatched tally
    let tally_result = match web_db
        .execute_category_tally(
            match_id,
            event_id,
            match_info.start_time,
            match_info.end_time,
            &event.event_category,
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            error!("Tally failed for match {}: {:?}", match_id, e);
            if let Err(fail_err) = web_db.set_match_failed(match_id).await {
                error!("Failed to mark match as failed: {:?}", fail_err);
            }
            return Err(e);
        }
    };

    info!(
        "Tally stats — rows: {}, valid kills: {}, rejected: {}",
        tally_result.rows_tallied, tally_result.kills_counted, tally_result.kills_rejected
    );

    // Mark as Completed
    web_db
        .set_match_completed(
            match_id,
            tally_result.rows_tallied,
            tally_result.kills_counted,
            tally_result.kills_rejected,
        )
        .await?;

    // Create snapshot immediately after tally
    match web_db
        .create_snapshot(event_id, match_id, match_label.as_deref())
        .await
    {
        Ok(snapshot_id) => info!("Snapshot created: {}", snapshot_id),
        Err(e) => warn!("Failed to create snapshot (tally still succeeded): {:?}", e),
    }

    // Announce LevelCapRace winners to Discord (fire-and-forget)
    if event.event_category == EventCategory::LevelCapRace {
        let started_at = chrono::Local::now().naive_local() - Duration::seconds(60);
        let winners = web_db
            .take_lcr_winners_since(event_id, started_at)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to fetch LCR winners: {}", e);
                Vec::new()
            });

        if !winners.is_empty() {
            let event_name = event.event_name.clone();
            tokio::spawn(async move {
                crate::services::discord::announce_level_cap_winners(&event_name, &winners).await;
            });
        }
    }

    Ok(true)
}

/// Run tally for all active level cap race events.
/// Returns the count of events tallied.
async fn run_level_cap_race_tally(web_db: &WebDB, app_state: &AppState) -> anyhow::Result<usize> {
    let events = web_db.list_events().await?;

    let mut count = 0;

    for event in events {
        if event.base.event_category != EventCategory::LevelCapRace || !event.base.is_active {
            continue;
        }

        let event_id = event.base.event_id;

        info!("Running level cap race tally for event {}", event_id);

        match web_db.execute_level_cap_race_tally(event_id).await {
            Ok(result) => {
                info!(
                    "Level cap race tally completed for event {} — rows: {}",
                    event_id, result.rows_tallied
                );

                let match_id = uuid::Uuid::new_v4();

                if let Err(e) = web_db
                    .create_snapshot(event_id, match_id, Some("periodic_lcr_tally"))
                    .await
                {
                    warn!(
                        "Failed to create snapshot for level cap race event {}: {:?}",
                        event_id, e
                    );
                }

                // Invalidate the leaderboard cache for this event
                let mut cache = app_state.level_cap_race_leaderboard_cache.lock().await;
                cache.remove(&event_id);
                drop(cache);
                info!("Invalidated LevelCapRace leaderboard cache for event {}", event_id);

                count += 1;
            }
            Err(e) => {
                error!(
                    "Failed to tally level cap race event {}: {:?}",
                    event_id, e
                );
            }
        }
    }

    Ok(count)
}
