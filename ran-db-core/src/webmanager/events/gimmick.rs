use anyhow::{bail, Context, Result};
use chrono::NaiveDateTime;
use ran_models::events::gimmick::{CreateGimmickRequest, EventMatchGimmick};
use ran_models::events::match_schedule::MatchSchedule;
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    pub async fn create_gimmick(
        &self,
        match_id: Uuid,
        req: &CreateGimmickRequest,
    ) -> Result<EventMatchGimmick> {
        // Validate the gimmick window against match times and status
        self.validate_gimmick_window(match_id, req.gimmick_start_time, req.gimmick_end_time)
            .await?;

        let gimmick_id = Uuid::new_v4();
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        client
            .execute(
                "INSERT INTO EventMatchGimmick
                    (GimmickID, MatchID, GimmickType, GimmickStartTime, GimmickEndTime, CreatedAt)
                 VALUES (@P1, @P2, @P3, @P4, @P5, GETDATE())",
                &[
                    &gimmick_id,
                    &match_id,
                    &req.gimmick_type.as_db_str(),
                    &req.gimmick_start_time,
                    &req.gimmick_end_time,
                ],
            )
            .await
            .context("Failed to insert gimmick")?;

        let row = client
            .query(
                "SELECT * FROM EventMatchGimmick WHERE GimmickID = @P1",
                &[&gimmick_id],
            )
            .await
            .context("Failed to fetch inserted gimmick")?
            .into_row()
            .await
            .context("Failed to read gimmick row")?;

        Ok(row
            .map(|r| EventMatchGimmick::from_row(&r))
            .expect("Gimmick was just inserted"))
    }

    pub async fn get_gimmicks_by_event(&self, event_id: Uuid) -> Result<Vec<EventMatchGimmick>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let rows = client
            .query(
                "SELECT g.* FROM EventMatchGimmick g
                 INNER JOIN MatchSchedule m ON m.MatchID = g.MatchID
                 WHERE m.EventID = @P1
                 ORDER BY g.GimmickStartTime ASC",
                &[&event_id],
            )
            .await
            .context("Failed to query gimmicks for event")?
            .into_first_result()
            .await
            .context("Failed to fetch gimmick rows")?;

        Ok(rows.iter().map(EventMatchGimmick::from_row).collect())
    }

    pub async fn list_gimmicks_for_match(&self, match_id: Uuid) -> Result<Vec<EventMatchGimmick>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let rows = client
            .query(
                "SELECT * FROM EventMatchGimmick WHERE MatchID = @P1 ORDER BY GimmickStartTime ASC",
                &[&match_id],
            )
            .await
            .context("Failed to query gimmicks")?
            .into_first_result()
            .await
            .context("Failed to fetch gimmick rows")?;

        Ok(rows.iter().map(EventMatchGimmick::from_row).collect())
    }

    pub async fn delete_gimmick(&self, match_id: Uuid, gimmick_id: Uuid) -> Result<()> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        // Check match exists and is not locked before attempting delete
        let status_row = client
            .query(
                "SELECT TallyStatus FROM MatchSchedule WHERE MatchID = @P1",
                &[&match_id],
            )
            .await
            .context("Failed to query match")?
            .into_row()
            .await
            .context("Failed to read match row")?;

        match status_row {
            Some(r) => {
                let tally_status = r.get::<&str, _>("TallyStatus").unwrap_or("Pending").to_string();
                if !["Pending", "Waiting", "Failed"].contains(&tally_status.as_str()) {
                    bail!(
                        "LOCKED:Cannot modify gimmicks while match status is '{}'",
                        tally_status
                    );
                }
            }
            None => bail!("NOT_FOUND:Match not found"),
        }

        // Embed status check in WHERE to close the TOCTOU window between the status read and delete
        let result = client
            .execute(
                "DELETE FROM EventMatchGimmick
                 WHERE GimmickID = @P1
                   AND MatchID = @P2
                   AND EXISTS (
                       SELECT 1 FROM MatchSchedule
                       WHERE MatchID = @P2
                         AND TallyStatus IN ('Pending','Waiting','Failed')
                   )",
                &[&gimmick_id, &match_id],
            )
            .await
            .context("Failed to delete gimmick")?;

        if result.total() == 0 {
            bail!("NOT_FOUND:Gimmick not found or match is locked");
        }

        Ok(())
    }

    async fn validate_gimmick_window(
        &self,
        match_id: Uuid,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Result<()> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let row = client
            .query(
                "SELECT * FROM MatchSchedule WHERE MatchID = @P1",
                &[&match_id],
            )
            .await
            .context("Failed to query match")?
            .into_row()
            .await
            .context("Failed to read match row")?;

        let match_data = match row {
            Some(r) => MatchSchedule::from_row(&r),
            None => bail!("NOT_FOUND:Match not found"),
        };

        // Check TallyStatus
        if !["Pending", "Waiting", "Failed"].contains(&match_data.tally_status.as_str()) {
            bail!(
                "LOCKED:Cannot modify gimmicks while match status is '{}'",
                match_data.tally_status
            );
        }

        // Check window validity
        if end <= start {
            bail!("VALIDATION:GimmickEndTime must be after GimmickStartTime");
        }

        // Check window is within match range
        if start < match_data.start_time || end > match_data.end_time {
            bail!("OUT_OF_MATCH_RANGE:Gimmick window must be within the match time range");
        }

        Ok(())
    }
}
