use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime, TimeZone};
use ran_models::{
    email::{
        CampaignProgress, CampaignRecipient, CreateEmailCampaignRequest, EmailCampaign,
    },
    BaseDataCountResult,
};
use uuid::Uuid;

use crate::webmanager::WebDB;

fn opt_naive_to_local(naive: Option<NaiveDateTime>) -> Option<chrono::DateTime<Local>> {
    naive.and_then(|n| Local.from_local_datetime(&n).single())
}

fn naive_to_local(naive: NaiveDateTime) -> chrono::DateTime<Local> {
    Local
        .from_local_datetime(&naive)
        .single()
        .unwrap_or_else(|| Local::now())
}

fn row_to_campaign(row: &tiberius::Row) -> EmailCampaign {
    EmailCampaign {
        campaign_id: row
            .try_get::<Uuid, _>("CampaignID")
            .ok()
            .flatten()
            .unwrap_or_default(),
        campaign_name: row
            .try_get::<&str, _>("CampaignName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        template_id: row
            .try_get::<Uuid, _>("TemplateID")
            .ok()
            .flatten()
            .unwrap_or_default(),
        template_name: row
            .try_get::<&str, _>("TemplateName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        subject_line: row
            .try_get::<&str, _>("SubjectLine")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        html_body: row
            .try_get::<&str, _>("HtmlBody")
            .ok()
            .flatten()
            .map(|s| s.to_string()),
        segment_type: row
            .try_get::<u8, _>("SegmentType")
            .ok()
            .flatten()
            .unwrap_or(0) as i16,
        segment_param: row.try_get::<i32, _>("SegmentParam").ok().flatten(),
        status: row
            .try_get::<u8, _>("Status")
            .ok()
            .flatten()
            .unwrap_or(0) as i16,
        scheduled_at: opt_naive_to_local(
            row.try_get::<NaiveDateTime, _>("ScheduledAt").ok().flatten(),
        ),
        started_at: opt_naive_to_local(
            row.try_get::<NaiveDateTime, _>("StartedAt").ok().flatten(),
        ),
        completed_at: opt_naive_to_local(
            row.try_get::<NaiveDateTime, _>("CompletedAt").ok().flatten(),
        ),
        total_recipients: row
            .try_get::<i32, _>("TotalRecipients")
            .ok()
            .flatten()
            .unwrap_or(0),
        total_sent: row
            .try_get::<i32, _>("TotalSent")
            .ok()
            .flatten()
            .unwrap_or(0),
        total_failed: row
            .try_get::<i32, _>("TotalFailed")
            .ok()
            .flatten()
            .unwrap_or(0),
        created_by: row
            .try_get::<&str, _>("CreatedBy")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        created_at: naive_to_local(
            row.try_get::<NaiveDateTime, _>("CreatedAt")
                .ok()
                .flatten()
                .unwrap_or_default(),
        ),
        updated_at: naive_to_local(
            row.try_get::<NaiveDateTime, _>("UpdatedAt")
                .ok()
                .flatten()
                .unwrap_or_default(),
        ),
    }
}

fn row_to_recipient(row: &tiberius::Row) -> CampaignRecipient {
    CampaignRecipient {
        campaign_id: row
            .try_get::<Uuid, _>("CampaignID")
            .ok()
            .flatten()
            .unwrap_or_default(),
        user_num: row
            .try_get::<i64, _>("UserNum")
            .ok()
            .flatten()
            .unwrap_or(0),
        user_email: row
            .try_get::<&str, _>("UserEmail")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        user_name: row
            .try_get::<&str, _>("UserName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        send_status: row
            .try_get::<u8, _>("SendStatus")
            .ok()
            .flatten()
            .unwrap_or(0) as i16,
        attempt_count: row
            .try_get::<u8, _>("AttemptCount")
            .ok()
            .flatten()
            .unwrap_or(0) as i16,
        sent_at: opt_naive_to_local(row.try_get::<NaiveDateTime, _>("SentAt").ok().flatten()),
        error_message: row
            .try_get::<&str, _>("ErrorMessage")
            .ok()
            .flatten()
            .map(|s| s.to_string()),
    }
}

#[async_trait::async_trait]
pub trait WebEmailCampaignTrait {
    async fn create_email_campaign(
        &self,
        req: CreateEmailCampaignRequest,
        admin_username: &str,
    ) -> Result<EmailCampaign>;

    async fn get_email_campaigns(
        &self,
        status: Option<i16>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult<EmailCampaign>>;

    async fn get_email_campaign_by_id(
        &self,
        campaign_id: Uuid,
    ) -> Result<Option<EmailCampaign>>;

    async fn update_campaign_status(
        &self,
        campaign_id: Uuid,
        new_status: i16,
        total_recipients: Option<i32>,
        total_sent: Option<i32>,
        total_failed: Option<i32>,
    ) -> Result<()>;

    async fn populate_campaign_recipients(
        &self,
        campaign_id: Uuid,
        segment_type: i16,
        segment_param: Option<i32>,
    ) -> Result<i32>;

    async fn get_pending_recipient_batch(
        &self,
        campaign_id: Uuid,
        batch_size: i32,
    ) -> Result<Vec<CampaignRecipient>>;

    async fn update_recipient_status(
        &self,
        campaign_id: Uuid,
        user_num: i64,
        send_status: i16,
        error_message: Option<String>,
    ) -> Result<()>;

    async fn get_scheduled_campaigns(&self) -> Result<Vec<EmailCampaign>>;

    async fn get_active_campaign(&self) -> Result<Option<EmailCampaign>>;

    async fn get_campaign_progress(&self, campaign_id: Uuid) -> Result<Option<CampaignProgress>>;

    async fn retry_campaign(&self, campaign_id: Uuid) -> Result<i32>;
}

#[async_trait::async_trait]
impl WebEmailCampaignTrait for WebDB {
    async fn create_email_campaign(
        &self,
        req: CreateEmailCampaignRequest,
        admin_username: &str,
    ) -> Result<EmailCampaign> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let segment_type = req.segment_type as u8;
        let scheduled_naive = req
            .scheduled_at
            .map(|dt| dt.naive_local());

        let sp_query = "EXEC sp_CreateEmailCampaign
            @CampaignName = @P1,
            @TemplateID = @P2,
            @SegmentType = @P3,
            @SegmentParam = @P4,
            @ScheduledAt = @P5,
            @CreatedBy = @P6";

        let row = client
            .query(
                sp_query,
                &[
                    &req.campaign_name.as_str(),
                    &req.template_id,
                    &segment_type,
                    &req.segment_param,
                    &scheduled_naive,
                    &admin_username,
                ],
            )
            .await
            .context("Failed to execute sp_CreateEmailCampaign")?
            .into_row()
            .await
            .context("Failed to fetch row")?
            .context("No row returned from sp_CreateEmailCampaign")?;

        Ok(row_to_campaign(&row))
    }

    async fn get_email_campaigns(
        &self,
        status: Option<i16>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult<EmailCampaign>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let status_param: Option<u8> = status.map(|s| s as u8);

        let sp_query = "EXEC sp_GetEmailCampaigns @Status = @P1, @Page = @P2, @PageSize = @P3";

        let results = client
            .query(sp_query, &[&status_param, &page, &page_size])
            .await
            .context("Failed to execute sp_GetEmailCampaigns")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        let mut result_sets = results.into_iter();

        let total_count = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let data: Vec<EmailCampaign> = result_sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| row_to_campaign(row))
            .collect();

        Ok(BaseDataCountResult {
            data,
            total_records: total_count,
        })
    }

    async fn get_email_campaign_by_id(
        &self,
        campaign_id: Uuid,
    ) -> Result<Option<EmailCampaign>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetEmailCampaignById @CampaignID = @P1";

        let row = client
            .query(sp_query, &[&campaign_id])
            .await
            .context("Failed to execute sp_GetEmailCampaignById")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        Ok(row.map(|r| row_to_campaign(&r)))
    }

    async fn update_campaign_status(
        &self,
        campaign_id: Uuid,
        new_status: i16,
        total_recipients: Option<i32>,
        total_sent: Option<i32>,
        total_failed: Option<i32>,
    ) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let status_u8 = new_status as u8;

        let sp_query = "EXEC sp_UpdateCampaignStatus
            @CampaignID = @P1,
            @NewStatus = @P2,
            @TotalRecipients = @P3,
            @TotalSent = @P4,
            @TotalFailed = @P5";

        client
            .execute(
                sp_query,
                &[
                    &campaign_id,
                    &status_u8,
                    &total_recipients,
                    &total_sent,
                    &total_failed,
                ],
            )
            .await
            .context("Failed to execute sp_UpdateCampaignStatus")?;

        Ok(())
    }

    async fn populate_campaign_recipients(
        &self,
        campaign_id: Uuid,
        segment_type: i16,
        segment_param: Option<i32>,
    ) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let seg_type_u8 = segment_type as u8;

        let sp_query = "EXEC sp_PopulateCampaignRecipients
            @CampaignID = @P1,
            @SegmentType = @P2,
            @SegmentParam = @P3";

        let row = client
            .query(sp_query, &[&campaign_id, &seg_type_u8, &segment_param])
            .await
            .context("Failed to execute sp_PopulateCampaignRecipients")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        let count = row
            .and_then(|r| r.get::<i32, _>("TotalRecipients"))
            .unwrap_or(0);

        Ok(count)
    }

    async fn get_pending_recipient_batch(
        &self,
        campaign_id: Uuid,
        batch_size: i32,
    ) -> Result<Vec<CampaignRecipient>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query =
            "EXEC sp_GetPendingRecipientBatch @CampaignID = @P1, @BatchSize = @P2";

        let results = client
            .query(sp_query, &[&campaign_id, &batch_size])
            .await
            .context("Failed to execute sp_GetPendingRecipientBatch")?
            .into_first_result()
            .await
            .context("Failed to fetch results")?;

        Ok(results.iter().map(|r| row_to_recipient(r)).collect())
    }

    async fn update_recipient_status(
        &self,
        campaign_id: Uuid,
        user_num: i64,
        send_status: i16,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let status_u8 = send_status as u8;

        let sp_query = "EXEC sp_UpdateRecipientStatus
            @CampaignID = @P1,
            @UserNum = @P2,
            @SendStatus = @P3,
            @ErrorMsg = @P4";

        client
            .execute(
                sp_query,
                &[
                    &campaign_id,
                    &user_num,
                    &status_u8,
                    &error_message,
                ],
            )
            .await
            .context("Failed to execute sp_UpdateRecipientStatus")?;

        Ok(())
    }

    async fn get_scheduled_campaigns(&self) -> Result<Vec<EmailCampaign>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let results = client
            .query("EXEC sp_GetScheduledCampaigns", &[])
            .await
            .context("Failed to execute sp_GetScheduledCampaigns")?
            .into_first_result()
            .await
            .context("Failed to fetch results")?;

        Ok(results.iter().map(|r| row_to_campaign(r)).collect())
    }

    async fn get_active_campaign(&self) -> Result<Option<EmailCampaign>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let row = client
            .query("EXEC sp_GetActiveCampaign", &[])
            .await
            .context("Failed to execute sp_GetActiveCampaign")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        Ok(row.map(|r| row_to_campaign(&r)))
    }

    async fn retry_campaign(&self, campaign_id: Uuid) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let row = client
            .query("EXEC sp_RetryCampaign @CampaignId = @P1", &[&campaign_id])
            .await
            .context("Failed to execute sp_RetryCampaign")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        Ok(row.and_then(|r| r.get::<i32, _>("reset_count")).unwrap_or(0))
    }

    async fn get_campaign_progress(&self, campaign_id: Uuid) -> Result<Option<CampaignProgress>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetCampaignProgress @CampaignID = @P1";

        let row = client
            .query(sp_query, &[&campaign_id])
            .await
            .context("Failed to execute sp_GetCampaignProgress")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        Ok(row.map(|r| CampaignProgress {
            campaign_id: r
                .try_get::<Uuid, _>("CampaignID")
                .ok()
                .flatten()
                .unwrap_or_default(),
            status: r.try_get::<u8, _>("Status").ok().flatten().unwrap_or(0) as i16,
            total_recipients: r
                .try_get::<i32, _>("TotalRecipients")
                .ok()
                .flatten()
                .unwrap_or(0),
            total_sent: r
                .try_get::<i32, _>("TotalSent")
                .ok()
                .flatten()
                .unwrap_or(0),
            total_failed: r
                .try_get::<i32, _>("TotalFailed")
                .ok()
                .flatten()
                .unwrap_or(0),
            total_pending: r
                .try_get::<i32, _>("TotalPending")
                .ok()
                .flatten()
                .unwrap_or(0),
        }))
    }
}
