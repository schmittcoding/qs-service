use std::{num::NonZeroU32, sync::Arc, time::Duration as StdDuration};

use chrono::{NaiveDate, Utc};
use futures_util::stream::{self, StreamExt};
use governor::{Quota, RateLimiter};
use ran_db_core::webmanager::{email::campaigns::WebEmailCampaignTrait, WebDB};
use ran_models::email::CampaignProgressEvent;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::{services::smtp, states::AppState};

const BATCH_SIZE: i32 = 500;
const MAX_ATTEMPTS: i16 = 3;
const FAILURE_THRESHOLD_COUNT: i32 = 200;
const POLL_INTERVAL_SECS: u64 = 15;
const SMTP_CONCURRENCY: usize = 5;
const SMTP_RATE_PER_SEC: u32 = 5;
const DAILY_SMTP_QUOTA: i32 = 10_000;

struct DailySendTracker {
    date: NaiveDate,
    count: i32,
}

impl DailySendTracker {
    fn new() -> Self {
        Self {
            date: Utc::now().date_naive(),
            count: 0,
        }
    }

    fn remaining(&mut self) -> i32 {
        let today = Utc::now().date_naive();
        if today != self.date {
            self.date = today;
            self.count = 0;
        }
        DAILY_SMTP_QUOTA - self.count
    }

    fn add(&mut self, n: i32) {
        self.count += n;
    }
}

pub fn spawn_campaign_worker(web_db: Arc<WebDB>, app_state: Arc<AppState>) {
    tokio::spawn(async move {
        info!("Campaign email worker started — polling every {}s", POLL_INTERVAL_SECS);
        let mut daily_tracker = DailySendTracker::new();

        loop {
            sleep(StdDuration::from_secs(POLL_INTERVAL_SECS)).await;

            if let Err(e) = promote_scheduled_campaigns(&web_db).await {
                error!("Campaign scheduler promotion error: {:?}", e);
            }

            if let Err(e) = process_active_campaign(&web_db, &app_state, &mut daily_tracker).await {
                error!("Campaign worker processing error: {:?}", e);
            }
        }
    });
}

async fn promote_scheduled_campaigns(web_db: &WebDB) -> anyhow::Result<()> {
    let due = web_db.get_scheduled_campaigns().await?;

    for campaign in due {
        info!(
            "Promoting scheduled campaign '{}' ({}) to Queued",
            campaign.campaign_name, campaign.campaign_id
        );

        let count = web_db
            .populate_campaign_recipients(
                campaign.campaign_id,
                campaign.segment_type,
                campaign.segment_param,
            )
            .await?;

        info!(
            "Campaign '{}': {} recipients queued",
            campaign.campaign_name, count
        );

        web_db
            .update_campaign_status(campaign.campaign_id, 2, Some(count), None, None)
            .await?;
    }

    Ok(())
}

async fn process_active_campaign(web_db: &WebDB, app_state: &AppState, daily_tracker: &mut DailySendTracker) -> anyhow::Result<()> {
    let campaign = match web_db.get_active_campaign().await? {
        Some(c) => c,
        None => return Ok(()),
    };

    let campaign_id = campaign.campaign_id;

    if campaign.status == 2 {
        web_db.update_campaign_status(campaign_id, 3, None, None, None).await?;
        info!(
            "Campaign '{}' ({}) transitioned Queued → Sending",
            campaign.campaign_name, campaign_id
        );
    }

    let html_template = campaign.html_body.unwrap_or_default();
    let subject_template = campaign.subject_line.clone();

    let mut total_sent = campaign.total_sent;
    let mut total_failed = campaign.total_failed;
    let mut perm_failed_this_run: i32 = 0;
    let mut batch_counter = 0u32;

    let rate_limiter = RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(SMTP_RATE_PER_SEC).unwrap()),
    );

    loop {
        let remaining_quota = daily_tracker.remaining();
        if remaining_quota <= 0 {
            info!(
                "Campaign '{}': daily SMTP quota ({}) reached — pausing until tomorrow",
                campaign.campaign_name, DAILY_SMTP_QUOTA
            );

            let pending = campaign.total_recipients - total_sent - total_failed;
            broadcast_progress(
                app_state,
                campaign_id,
                3,
                campaign.total_recipients,
                total_sent,
                total_failed,
                pending.max(0),
            );
            break;
        }

        let fetch_size = BATCH_SIZE.min(remaining_quota);

        let batch = web_db
            .get_pending_recipient_batch(campaign_id, fetch_size)
            .await?;

        if batch.is_empty() {
            web_db
                .update_campaign_status(campaign_id, 4, None, Some(total_sent), Some(total_failed))
                .await?;

            info!(
                "Campaign '{}' completed: {} sent, {} failed",
                campaign.campaign_name, total_sent, total_failed
            );

            broadcast_progress(
                app_state,
                campaign_id,
                4,
                campaign.total_recipients,
                total_sent,
                total_failed,
                0,
            );
            break;
        }

        // Separate recipients that have exhausted retries from those to send
        let mut to_send = Vec::new();
        for recipient in &batch {
            if recipient.attempt_count >= MAX_ATTEMPTS {
                if let Err(e) = web_db
                    .update_recipient_status(
                        campaign_id,
                        recipient.user_num,
                        2,
                        Some("Max retry attempts reached".to_string()),
                    )
                    .await
                {
                    warn!("Failed to mark recipient {} as failed: {:?}", recipient.user_num, e);
                }
                total_failed += 1;
                perm_failed_this_run += 1;
            } else {
                to_send.push(recipient);
            }
        }

        // Filter out suppressed recipients
        let suppression_cache = app_state.suppression_cache.read().await;
        let mut sendable = Vec::new();
        for recipient in to_send {
            if suppression_cache.contains(&recipient.user_email.to_lowercase()) {
                if let Err(e) = web_db
                    .update_recipient_status(
                        campaign_id,
                        recipient.user_num,
                        2,
                        Some("Email suppressed (bounce/complaint)".to_string()),
                    )
                    .await
                {
                    warn!("Failed to mark suppressed recipient {} as failed: {:?}", recipient.user_num, e);
                }
                total_failed += 1;
            } else {
                sendable.push(recipient);
            }
        }
        drop(suppression_cache);

        // Collect send items with owned data to avoid lifetime issues
        let send_items: Vec<_> = sendable
            .iter()
            .map(|r| (r.user_num, r.user_name.clone(), r.user_email.clone()))
            .collect();

        // Concurrent SMTP sending with rate limiting
        let results: Vec<_> = stream::iter(send_items.into_iter().map(|(user_num, user_name, user_email)| {
            let rate_limiter = &rate_limiter;
            let mailer = &app_state.smtp_transport;
            let from_email = &app_state.smtp_from_email;
            let html_template = &html_template;
            let subject_template = &subject_template;

            async move {
                let mut vars = smtp::default_preview_variables();
                vars.insert("username".to_string(), user_name);
                vars.insert("email".to_string(), user_email.clone());
                vars.insert("user_num".to_string(), user_num.to_string());

                let (rendered_subject, rendered_html) =
                    smtp::render_template(html_template, subject_template, &vars);

                rate_limiter.until_ready().await;

                let result = smtp::send_campaign_email(
                    mailer,
                    from_email,
                    &user_email,
                    &rendered_subject,
                    &rendered_html,
                )
                .await;

                (user_num, user_email, result)
            }
        }))
        .buffer_unordered(SMTP_CONCURRENCY)
        .collect()
        .await;

        // Process results sequentially for DB updates
        for (user_num, _email, result) in &results {
            match result {
                Ok(()) => {
                    if let Err(e) = web_db
                        .update_recipient_status(campaign_id, *user_num, 1, None)
                        .await
                    {
                        warn!("Failed to mark recipient {} as sent: {:?}", user_num, e);
                    }
                    total_sent += 1;
                }
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    warn!("SMTP send failed for recipient {} (campaign {}): {}", user_num, campaign_id, &err_msg);
                    if let Err(update_err) = web_db
                        .update_recipient_status(
                            campaign_id,
                            *user_num,
                            0,
                            Some(err_msg.chars().take(490).collect()),
                        )
                        .await
                    {
                        warn!(
                            "Failed to update recipient {} after send error: {:?}",
                            user_num, update_err
                        );
                    }
                }
            }
        }

        let sent_this_batch = results.iter().filter(|(_, _, r)| r.is_ok()).count() as i32;
        daily_tracker.add(sent_this_batch);

        batch_counter += 1;

        if batch_counter % 2 == 0 {
            if let Err(e) = web_db
                .update_campaign_status(campaign_id, 3, None, Some(total_sent), Some(total_failed))
                .await
            {
                warn!("Failed to update campaign progress counters: {:?}", e);
            }

            let pending = campaign.total_recipients - total_sent - total_failed;
            let pending = pending.max(0);
            broadcast_progress(
                app_state,
                campaign_id,
                3,
                campaign.total_recipients,
                total_sent,
                total_failed,
                pending,
            );
        }

        let total_resolved = total_sent + perm_failed_this_run;
        if total_resolved >= FAILURE_THRESHOLD_COUNT
            && perm_failed_this_run as f32 / total_resolved as f32 > 0.5
        {
            error!(
                "Campaign '{}' failure rate exceeded threshold ({}/{} permanently failed) — marking Failed",
                campaign.campaign_name, perm_failed_this_run, total_resolved
            );
            web_db
                .update_campaign_status(campaign_id, 5, None, Some(total_sent), Some(total_failed))
                .await?;

            broadcast_progress(
                app_state,
                campaign_id,
                5,
                campaign.total_recipients,
                total_sent,
                total_failed,
                campaign.total_recipients - total_sent - total_failed,
            );
            break;
        }
    }

    Ok(())
}

fn broadcast_progress(
    app_state: &AppState,
    campaign_id: uuid::Uuid,
    status: i16,
    total_recipients: i32,
    total_sent: i32,
    total_failed: i32,
    total_pending: i32,
) {
    let event = CampaignProgressEvent {
        campaign_id,
        status,
        total_recipients,
        total_sent,
        total_failed,
        total_pending,
    };
    let _ = app_state.campaign_progress_tx.send(event);
}
