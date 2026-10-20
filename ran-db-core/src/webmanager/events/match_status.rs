use anyhow::{bail, Result};
use ran_models::events::event::SetTowerResultsRequest;
use ran_models::events::match_schedule::MatchSchedule;
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// Get current match status for the frontend banner
    /// Returns the match that is currently live, tallying soon, or tallying now
    pub async fn get_current_match_status(&self, event_id: Uuid) -> Result<Option<MatchSchedule>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT TOP 1 * FROM MatchSchedule
                 WHERE EventID = @P1
                   AND GETDATE() BETWEEN StartTime AND DATEADD(MINUTE, 30, EndTime)
                   AND TallyStatus IN ('Pending','Waiting','Processing')
                 ORDER BY StartTime",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| MatchSchedule::from_row(&r)))
    }

    /// Get all matches for an event
    pub async fn get_matches_by_event(&self, event_id: Uuid) -> Result<Vec<MatchSchedule>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM MatchSchedule WHERE EventID = @P1 ORDER BY StartTime",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(MatchSchedule::from_row).collect())
    }

    /// Find the next match waiting for tally
    pub async fn get_next_tally_match(&self) -> Result<Option<MatchSchedule>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT TOP 1 * FROM MatchSchedule
                 WHERE TallyStatus = 'Waiting' AND TallyScheduled <= GETDATE()
                 ORDER BY TallyScheduled",
                &[],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| MatchSchedule::from_row(&r)))
    }

    /// Get first pending tally match for a specific event (admin force-tally endpoint)
    pub async fn get_pending_match_for_event(&self, event_id: Uuid) -> Result<Option<MatchSchedule>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT TOP 1 * FROM MatchSchedule
                 WHERE EventID = @P1 AND TallyStatus = 'Waiting'
                 ORDER BY TallyScheduled",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| MatchSchedule::from_row(&r)))
    }

    /// Mark match as Processing
    pub async fn set_match_processing(&self, match_id: Uuid) -> Result<()> {
        let mut client = self.pool.get().await?;
        client
            .execute(
                "UPDATE MatchSchedule SET TallyStatus = 'Processing', TallyStartedAt = GETDATE() WHERE MatchID = @P1",
                &[&match_id],
            )
            .await?;
        Ok(())
    }

    /// Mark match as Completed with stats
    pub async fn set_match_completed(
        &self,
        match_id: Uuid,
        rows_tallied: i32,
        kills_counted: i32,
        kills_rejected: i32,
    ) -> Result<()> {
        let mut client = self.pool.get().await?;
        client
            .execute(
                "UPDATE MatchSchedule
                 SET TallyStatus = 'Completed', TallyCompletedAt = GETDATE(),
                     RowsTallied = @P1, KillsCounted = @P2, KillsRejected = @P3
                 WHERE MatchID = @P4",
                &[&rows_tallied, &kills_counted, &kills_rejected, &match_id],
            )
            .await?;
        Ok(())
    }

    /// Mark match as Failed
    pub async fn set_match_failed(&self, match_id: Uuid) -> Result<()> {
        let mut client = self.pool.get().await?;
        client
            .execute(
                "UPDATE MatchSchedule SET TallyStatus = 'Failed', TallyCompletedAt = GETDATE() WHERE MatchID = @P1",
                &[&match_id],
            )
            .await?;
        Ok(())
    }

    /// Transition all matches whose EndTime has passed from 'Pending' to 'Waiting'.
    /// Called by the match readiness scheduler at fixed times (14:46, 17:46, 20:46, 23:46).
    /// Returns the number of matches transitioned.
    pub async fn transition_ended_matches_to_waiting(&self) -> Result<i32> {
        let mut client = self.pool.get().await?;
        let rows_affected = client
            .execute(
                "UPDATE MatchSchedule
                 SET TallyStatus = 'Waiting'
                 WHERE TallyStatus = 'Pending'
                   AND EndTime <= GETDATE()",
                &[],
            )
            .await?
            .rows_affected()
            .iter()
            .sum::<u64>() as i32;
        Ok(rows_affected)
    }

    /// Admin sets tower winners after a match ends and transitions to 'Waiting' for tally.
    /// Only allowed when current status is 'Pending' (match has ended but not yet tallied).
    pub async fn set_match_tower_results(
        &self,
        match_id: Uuid,
        req: &SetTowerResultsRequest,
    ) -> Result<MatchSchedule> {
        let mut client = self.pool.get().await?;

        // Get existing match
        let row = client
            .query(
                "SELECT * FROM MatchSchedule WHERE MatchID = @P1",
                &[&match_id],
            )
            .await?
            .into_row()
            .await?;

        let existing = match row {
            Some(r) => MatchSchedule::from_row(&r),
            None => bail!("NOT_FOUND:Match not found"),
        };

        if existing.tally_status != "Pending" {
            bail!(
                "INVALID_STATUS:Cannot set tower results when status is '{}'. Match must be in 'Pending' status (match ended, awaiting tower results).",
                existing.tally_status
            );
        }

        // Validate tower winners are valid ChaSchool values (0=SG, 1=MP, 2=PH) if provided
        for (name, val) in [
            ("tower_winner_th", req.tower_winner_th),
            ("tower_winner_nuc", req.tower_winner_nuc),
            ("tower_winner_faci", req.tower_winner_faci),
        ] {
            if let Some(school) = val {
                if !(0..=2).contains(&school) {
                    bail!("VALIDATION:{} must be a valid ChaSchool (0=SG, 1=MP, 2=PH)", name);
                }
            }
        }

        // Set tower winners and transition to 'Waiting' so tally scheduler picks it up
        client
            .execute(
                "UPDATE MatchSchedule
                 SET TowerWinnerTH = @P1, TowerWinnerNuc = @P2, TowerWinnerFaci = @P3,
                     TallyStatus = 'Waiting',
                     TallyStartedAt = NULL,
                     TallyCompletedAt = NULL,
                     RowsTallied = NULL,
                     KillsCounted = NULL,
                     KillsRejected = NULL
                 WHERE MatchID = @P4 AND TallyStatus = 'Pending'",
                &[
                    &req.tower_winner_th,
                    &req.tower_winner_nuc,
                    &req.tower_winner_faci,
                    &match_id,
                ],
            )
            .await?;

        // Return updated match
        let row = client
            .query(
                "SELECT * FROM MatchSchedule WHERE MatchID = @P1",
                &[&match_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row
            .map(|r| MatchSchedule::from_row(&r))
            .expect("Match exists"))
    }

    /// Admin manually sets a specific match to 'Waiting' status.
    /// Only allowed when current status is 'Pending' or 'Failed'.
    pub async fn set_match_waiting_admin(&self, match_id: Uuid) -> Result<bool> {
        let mut client = self.pool.get().await?;
        let rows_affected: u64 = client
            .execute(
                "UPDATE MatchSchedule
                 SET TallyStatus = 'Waiting',
                     TallyStartedAt = NULL,
                     TallyCompletedAt = NULL,
                     RowsTallied = NULL,
                     KillsCounted = NULL,
                     KillsRejected = NULL
                 WHERE MatchID = @P1
                   AND TallyStatus IN ('Pending', 'Failed')",
                &[&match_id],
            )
            .await?
            .rows_affected()
            .iter()
            .sum();
        Ok(rows_affected > 0)
    }
}
