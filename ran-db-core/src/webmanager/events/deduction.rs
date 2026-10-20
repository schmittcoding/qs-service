use anyhow::Result;
use ran_models::events::deduction::AdminDeduction;
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// Apply a new deduction and immediately recalculate the affected score
    pub async fn apply_deduction(
        &self,
        event_id: Uuid,
        target_type: &str,
        target_id: Option<i32>,
        target_cha_school: Option<i32>,
        points: i32,
        reason: &str,
        admin_user_id: String,
        match_id: Option<Uuid>,
    ) -> Result<AdminDeduction> {
        let mut client = self.pool.get().await?;
        let deduction_id = Uuid::new_v4();

        // Insert the deduction record
        client
            .execute(
                "INSERT INTO AdminDeduction
                    (DeductionID, EventID, TargetType, TargetID, TargetChaSchool,
                     Points, Reason, AdminUserID, AppliedAt, MatchID, IsReverted)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, GETDATE(), @P9, 0)",
                &[
                    &deduction_id,
                    &event_id,
                    &target_type,
                    &target_id,
                    &target_cha_school,
                    &points,
                    &reason,
                    &admin_user_id,
                    &match_id,
                ],
            )
            .await?;

        // Recalculate the affected score table
        self.recalculate_deductions(event_id, target_type, target_id, target_cha_school)
            .await?;

        // Return the newly created deduction
        let row = client
            .query(
                "SELECT * FROM AdminDeduction WHERE DeductionID = @P1",
                &[&deduction_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row
            .map(|r| AdminDeduction::from_row(&r))
            .expect("Deduction was just inserted"))
    }

    /// Revert a deduction and recalculate the affected score
    pub async fn revert_deduction(
        &self,
        deduction_id: Uuid,
        reverted_by: String,
    ) -> Result<AdminDeduction> {
        let mut client = self.pool.get().await?;

        // Get the deduction first so we know which score to recalculate
        let row = client
            .query(
                "SELECT * FROM AdminDeduction WHERE DeductionID = @P1",
                &[&deduction_id],
            )
            .await?
            .into_row()
            .await?;

        let deduction = match row {
            Some(r) => AdminDeduction::from_row(&r),
            None => anyhow::bail!("Deduction not found"),
        };

        if deduction.is_reverted {
            anyhow::bail!("Deduction is already reverted");
        }

        // Mark as reverted
        client
            .execute(
                "UPDATE AdminDeduction
                 SET IsReverted = 1, RevertedAt = GETDATE(), RevertedBy = @P1
                 WHERE DeductionID = @P2",
                &[&reverted_by, &deduction_id],
            )
            .await?;

        // Recalculate
        self.recalculate_deductions(
            deduction.event_id,
            &deduction.target_type,
            deduction.target_id,
            deduction.target_cha_school,
        )
        .await?;

        // Return updated deduction
        let row = client
            .query(
                "SELECT * FROM AdminDeduction WHERE DeductionID = @P1",
                &[&deduction_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row
            .map(|r| AdminDeduction::from_row(&r))
            .expect("Deduction exists"))
    }

    /// List all deductions for an event, optionally filtered by target type
    pub async fn list_deductions(
        &self,
        event_id: Uuid,
        target_type: Option<&str>,
    ) -> Result<Vec<AdminDeduction>> {
        let mut client = self.pool.get().await?;

        let rows = if let Some(tt) = target_type {
            client
                .query(
                    "SELECT * FROM AdminDeduction
                     WHERE EventID = @P1 AND TargetType = @P2
                     ORDER BY AppliedAt DESC",
                    &[&event_id, &tt],
                )
                .await?
                .into_first_result()
                .await?
        } else {
            client
                .query(
                    "SELECT * FROM AdminDeduction
                     WHERE EventID = @P1
                     ORDER BY AppliedAt DESC",
                    &[&event_id],
                )
                .await?
                .into_first_result()
                .await?
        };

        Ok(rows.iter().map(AdminDeduction::from_row).collect())
    }

    /// Recalculate the Deduction column on the appropriate score table
    /// based on SUM of non-reverted deductions for the target
    async fn recalculate_deductions(
        &self,
        event_id: Uuid,
        target_type: &str,
        target_id: Option<i32>,
        target_cha_school: Option<i32>,
    ) -> Result<()> {
        let mut client = self.pool.get().await?;

        match target_type {
            "GUILD" => {
                let guild_num = target_id.ok_or_else(|| anyhow::anyhow!("GUILD deduction requires target_id"))?;
                client
                    .execute(
                        "UPDATE GuildEventScore
                         SET Deduction = COALESCE((
                             SELECT SUM(Points) FROM AdminDeduction
                             WHERE EventID = @P1 AND TargetType = 'GUILD'
                               AND TargetID = @P2 AND IsReverted = 0
                         ), 0)
                         WHERE EventID = @P1 AND GuildNum = @P2",
                        &[&event_id, &guild_num],
                    )
                    .await?;
            }
            "PLAYER" => {
                let cha_num = target_id.ok_or_else(|| anyhow::anyhow!("PLAYER deduction requires target_id"))?;
                client
                    .execute(
                        "UPDATE KOTHScore
                         SET Deduction = COALESCE((
                             SELECT SUM(Points) FROM AdminDeduction
                             WHERE EventID = @P1 AND TargetType = 'PLAYER'
                               AND TargetID = @P2 AND IsReverted = 0
                         ), 0)
                         WHERE EventID = @P1 AND ChaNum = @P2",
                        &[&event_id, &cha_num],
                    )
                    .await?;
            }
            "SCHOOL" => {
                let cha_school = target_cha_school.ok_or_else(|| anyhow::anyhow!("SCHOOL deduction requires target_cha_school"))?;
                client
                    .execute(
                        "UPDATE SchoolEventScore
                         SET Deduction = COALESCE((
                             SELECT SUM(Points) FROM AdminDeduction
                             WHERE EventID = @P1 AND TargetType = 'SCHOOL'
                               AND TargetChaSchool = @P2 AND IsReverted = 0
                         ), 0)
                         WHERE EventID = @P1 AND ChaSchool = @P2",
                        &[&event_id, &cha_school],
                    )
                    .await?;
            }
            _ => anyhow::bail!("Invalid target_type: {}", target_type),
        }

        Ok(())
    }
}
