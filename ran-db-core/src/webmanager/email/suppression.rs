use super::super::WebDB;
use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime, TimeZone};
use ran_models::{email::EmailSuppressionEntry, BaseDataCountResult};
use uuid::Uuid;

fn opt_naive_to_local(naive: Option<NaiveDateTime>) -> Option<chrono::DateTime<Local>> {
    naive.and_then(|n| Local.from_local_datetime(&n).single())
}

fn naive_to_local(naive: NaiveDateTime) -> chrono::DateTime<Local> {
    Local
        .from_local_datetime(&naive)
        .single()
        .unwrap_or_else(|| Local::now())
}

fn row_to_suppression_entry(row: &tiberius::Row) -> EmailSuppressionEntry {
    EmailSuppressionEntry {
        suppression_id: row
            .try_get::<Uuid, _>("SuppressionID")
            .ok()
            .flatten()
            .unwrap_or_default(),
        email: row
            .try_get::<&str, _>("Email")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        reason: row
            .try_get::<&str, _>("Reason")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        diagnostic_info: row
            .try_get::<&str, _>("DiagnosticInfo")
            .ok()
            .flatten()
            .map(|s| s.to_string()),
        source: row
            .try_get::<&str, _>("Source")
            .ok()
            .flatten()
            .map(|s| s.to_string()),
        suppressed_at: naive_to_local(
            row.try_get::<NaiveDateTime, _>("SuppressedAt")
                .ok()
                .flatten()
                .unwrap_or_default(),
        ),
        removed_at: opt_naive_to_local(
            row.try_get::<NaiveDateTime, _>("RemovedAt").ok().flatten(),
        ),
        removed_by: row
            .try_get::<&str, _>("RemovedBy")
            .ok()
            .flatten()
            .map(|s| s.to_string()),
    }
}

#[async_trait::async_trait]
pub trait WebEmailSuppressionTrait {
    async fn add_email_suppression(
        &self,
        email: &str,
        reason: &str,
        diagnostic_info: Option<&str>,
        source: Option<&str>,
    ) -> Result<()>;

    async fn is_email_suppressed(&self, email: &str) -> Result<bool>;

    async fn get_all_suppressed_emails(&self) -> Result<Vec<String>>;

    async fn get_suppression_list(
        &self,
        page: i16,
        page_size: i16,
        reason: Option<&str>,
    ) -> Result<BaseDataCountResult<EmailSuppressionEntry>>;

    async fn remove_email_suppression(&self, email: &str, removed_by: &str) -> Result<()>;

    async fn seed_suppression_from_failures(&self) -> Result<i32>;
}

#[async_trait::async_trait]
impl WebEmailSuppressionTrait for WebDB {
    async fn add_email_suppression(
        &self,
        email: &str,
        reason: &str,
        diagnostic_info: Option<&str>,
        source: Option<&str>,
    ) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_AddEmailSuppression
            @Email = @P1,
            @Reason = @P2,
            @DiagnosticInfo = @P3,
            @Source = @P4";

        client
            .execute(sp_query, &[&email, &reason, &diagnostic_info, &source])
            .await
            .context("Failed to execute sp_AddEmailSuppression")?;

        Ok(())
    }

    async fn is_email_suppressed(&self, email: &str) -> Result<bool> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_IsEmailSuppressed @Email = @P1";

        let row = client
            .query(sp_query, &[&email])
            .await
            .context("Failed to execute sp_IsEmailSuppressed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        let is_suppressed = row
            .and_then(|r| r.get::<bool, _>("IsSuppressed"))
            .unwrap_or(false);

        Ok(is_suppressed)
    }

    async fn get_all_suppressed_emails(&self) -> Result<Vec<String>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetSuppressedEmails";

        let results = client
            .query(sp_query, &[])
            .await
            .context("Failed to execute sp_GetSuppressedEmails")?
            .into_first_result()
            .await
            .context("Failed to fetch results")?;

        let emails: Vec<String> = results
            .iter()
            .filter_map(|row| {
                row.try_get::<&str, _>("Email")
                    .ok()
                    .flatten()
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(emails)
    }

    async fn get_suppression_list(
        &self,
        page: i16,
        page_size: i16,
        reason: Option<&str>,
    ) -> Result<BaseDataCountResult<EmailSuppressionEntry>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetSuppressionList @Page = @P1, @PageSize = @P2, @Reason = @P3";

        let results = client
            .query(sp_query, &[&page, &page_size, &reason])
            .await
            .context("Failed to execute sp_GetSuppressionList")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        let mut result_sets = results.into_iter();

        let total_count = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalCount"))
            .unwrap_or(0);

        let data: Vec<EmailSuppressionEntry> = result_sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| row_to_suppression_entry(row))
            .collect();

        Ok(BaseDataCountResult {
            data,
            total_records: total_count,
        })
    }

    async fn remove_email_suppression(&self, email: &str, removed_by: &str) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_RemoveEmailSuppression @Email = @P1, @RemovedBy = @P2";

        client
            .execute(sp_query, &[&email, &removed_by])
            .await
            .context("Failed to execute sp_RemoveEmailSuppression")?;

        Ok(())
    }

    async fn seed_suppression_from_failures(&self) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_SeedSuppressionFromFailures";

        let row = client
            .query(sp_query, &[])
            .await
            .context("Failed to execute sp_SeedSuppressionFromFailures")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        let seeded_count = row
            .and_then(|r| r.get::<i32, _>("SeededCount"))
            .unwrap_or(0);

        Ok(seeded_count)
    }
}
