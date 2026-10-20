use super::super::WebDB;
use anyhow::{Context, Result};
use tracing::instrument;

impl WebDB {
    #[instrument(skip(self), level = "debug", err)]
    pub async fn log_email_sent(
        &self,
        user_num: i64,
        email_type: &str,
        recipient_email: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        client
            .execute(
                "EXEC sp_LogEmailSent @UserNum = @P1, @EmailType = @P2, @RecipientEmail = @P3, @Status = @P4, @ErrorMessage = @P5",
                &[
                    &user_num,
                    &email_type,
                    &recipient_email,
                    &status,
                    &error_message,
                ],
            )
            .await
            .context("Failed to execute sp_LogEmailSent")?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug", err)]
    pub async fn get_recent_verification_email_count(
        &self,
        user_num: i64,
        minute_window: i32,
    ) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let row = client
            .query(
                "EXEC sp_GetRecentVerificationEmailCount @UserNum = @P1, @MinuteWindow = @P2",
                &[&user_num, &minute_window],
            )
            .await
            .context("Failed to execute sp_GetRecentVerificationEmailCount")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(row) => {
                let count: i32 = row
                    .try_get("EmailCount")
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                Ok(count)
            }
            None => Ok(0),
        }
    }
}
