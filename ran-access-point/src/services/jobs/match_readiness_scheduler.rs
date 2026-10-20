use chrono::{Local, NaiveTime};
use ran_db_core::webmanager::WebDB;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Fixed schedule times when the readiness check runs (HH:MM).
/// These times are chosen to run shortly after matches typically end,
/// transitioning ended matches from 'Pending' → 'Waiting' so the
/// tally scheduler can pick them up.
const SCHEDULE_TIMES: [(u32, u32); 5] = [
    (14, 46),
    (17, 46),
    (20, 46),
    (23, 46),
    (23, 56),
];

/// Spawns a background task that runs at fixed times (14:46, 17:46, 20:46, 23:46)
/// to transition matches whose EndTime has passed from 'Pending' to 'Waiting'.
pub fn spawn_match_readiness_scheduler(web_db: Arc<WebDB>) {
    tokio::spawn(async move {
        info!(
            "Match readiness scheduler started — runs at {:?}",
            SCHEDULE_TIMES
                .iter()
                .map(|(h, m)| format!("{:02}:{:02}", h, m))
                .collect::<Vec<_>>()
        );

        loop {
            let sleep_duration = duration_until_next_run();
            info!(
                "Match readiness scheduler sleeping for {:.0}s (next run at {})",
                sleep_duration.as_secs_f64(),
                next_run_display()
            );

            tokio::time::sleep(sleep_duration).await;

            match web_db.transition_ended_matches_to_waiting().await {
                Ok(count) if count > 0 => {
                    info!(
                        "Match readiness scheduler: transitioned {} match(es) from Pending → Waiting",
                        count
                    );
                }
                Ok(_) => {
                    info!("Match readiness scheduler: no matches needed transitioning");
                }
                Err(e) => {
                    error!("Match readiness scheduler error: {:?}", e);
                }
            }
        }
    });
}

/// Calculate duration until the next scheduled run time.
fn duration_until_next_run() -> Duration {
    let now = Local::now();
    let current_time = now.time();

    // Find the next schedule time today or tomorrow
    for &(hour, minute) in &SCHEDULE_TIMES {
        let target = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
        if target > current_time {
            let diff = target - current_time;
            return diff.to_std().unwrap_or(Duration::from_secs(60));
        }
    }

    // All today's times have passed — next run is tomorrow's first schedule
    let (first_hour, first_minute) = SCHEDULE_TIMES[0];
    let tomorrow_target = NaiveTime::from_hms_opt(first_hour, first_minute, 0).unwrap();
    let remaining_today = NaiveTime::from_hms_opt(23, 59, 59).unwrap() - current_time;
    let tomorrow_offset = tomorrow_target - NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    let total_secs = remaining_today.num_seconds() + 1 + tomorrow_offset.num_seconds();
    Duration::from_secs(total_secs.max(1) as u64)
}

/// Get a display string for the next run time.
fn next_run_display() -> String {
    let now = Local::now();
    let current_time = now.time();

    for &(hour, minute) in &SCHEDULE_TIMES {
        let target = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
        if target > current_time {
            return format!("{:02}:{:02} today", hour, minute);
        }
    }

    let (h, m) = SCHEDULE_TIMES[0];
    format!("{:02}:{:02} tomorrow", h, m)
}
