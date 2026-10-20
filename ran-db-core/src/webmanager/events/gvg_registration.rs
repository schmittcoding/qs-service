use anyhow::{Context, Result};
use ran_models::events::registration::EventRegistration;
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// Bulk-insert GvG registrations for a guild leader's selected members.
    pub async fn insert_gvg_bulk_registration(
        &self,
        event_id: Uuid,
        guild_num: i32,
        leader_cha_num: i32,
        member_cha_nums: &[i32],
        cha_school: i32,
        ip_address: Option<&str>,
    ) -> Result<Vec<EventRegistration>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;
        let mut registrations = Vec::new();

        for &cha_num in member_cha_nums {
            let registration_id = Uuid::new_v4();

            // Get the character's info for class/level from game DB via cross-DB query
            let row = client
                .query(
                    "DECLARE @ChaClass INT, @ChaLevel INT, @UserNum INT;
                     SELECT @ChaClass = ChaClass, @ChaLevel = ChaLevel, @UserNum = UserNum
                     FROM [RanGameS1].[dbo].[ChaInfo] WHERE ChaNum = @P3;

                     INSERT INTO EventRegistration
                        (RegistrationID, EventID, ChaNum, UserNum, GuildNum, ChaSchool,
                         ChaClass, ChaLevel, Status, RegisteredAt, IPAddress)
                     VALUES
                        (@P1, @P2, @P3, @UserNum, @P4, @P5,
                         @ChaClass, @ChaLevel, 1, GETDATE(), @P6);

                     SELECT * FROM EventRegistration WHERE RegistrationID = @P1",
                    &[
                        &registration_id,
                        &event_id,
                        &cha_num,
                        &guild_num,
                        &cha_school,
                        &ip_address,
                    ],
                )
                .await
                .context("Failed to insert GvG registration")?
                .into_row()
                .await
                .context("Failed to fetch inserted registration")?;

            if let Some(r) = row {
                registrations.push(EventRegistration::from_row(&r));
            }
        }

        // Also register the leader if not already registered
        let existing = self
            .get_registration_by_event_and_char(event_id, leader_cha_num)
            .await?;

        if existing.is_none() {
            let registration_id = Uuid::new_v4();
            let row = client
                .query(
                    "DECLARE @ChaClass INT, @ChaLevel INT, @UserNum INT;
                     SELECT @ChaClass = ChaClass, @ChaLevel = ChaLevel, @UserNum = UserNum
                     FROM [RanGameS1].[dbo].[ChaInfo] WHERE ChaNum = @P3;

                     INSERT INTO EventRegistration
                        (RegistrationID, EventID, ChaNum, UserNum, GuildNum, ChaSchool,
                         ChaClass, ChaLevel, Status, RegisteredAt, IPAddress)
                     VALUES
                        (@P1, @P2, @P3, @UserNum, @P4, @P5,
                         @ChaClass, @ChaLevel, 1, GETDATE(), @P6);

                     SELECT * FROM EventRegistration WHERE RegistrationID = @P1",
                    &[
                        &registration_id,
                        &event_id,
                        &leader_cha_num,
                        &guild_num,
                        &cha_school,
                        &ip_address,
                    ],
                )
                .await
                .context("Failed to insert leader registration")?
                .into_row()
                .await
                .context("Failed to fetch leader registration")?;

            if let Some(r) = row {
                registrations.push(EventRegistration::from_row(&r));
            }
        }

        Ok(registrations)
    }

    /// Get guild members eligible for GvG registration.
    /// Returns characters that are: in the specified guild, same school as leader,
    /// meet min_level, not already registered, not deleted.
    pub async fn get_guild_eligible_members(
        &self,
        event_id: Uuid,
        guild_num: i32,
        leader_school: i32,
        min_level: i32,
    ) -> Result<Vec<tiberius::Row>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let rows = client
            .query(
                "SELECT c.ChaNum, c.ChaName, c.ChaClass, c.ChaSchool, c.ChaLevel,
                        g.GuName AS GuildName,
                        CASE WHEN er.RegistrationID IS NOT NULL THEN 1 ELSE 0 END AS AlreadyRegistered
                 FROM [RanGameS1].[dbo].[ChaInfo] c
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = c.GuNum
                 LEFT JOIN EventRegistration er
                    ON er.EventID = @P1 AND er.ChaNum = c.ChaNum AND er.Status = 1
                 WHERE c.GuNum = @P2
                   AND c.ChaSchool = @P3
                   AND c.ChaLevel >= @P4
                   AND c.ChaDeleted = 0
                 ORDER BY c.ChaLevel DESC, c.ChaName",
                &[&event_id, &guild_num, &leader_school, &min_level],
            )
            .await
            .context("Failed to query eligible members")?
            .into_first_result()
            .await
            .context("Failed to fetch eligible members")?;

        Ok(rows)
    }
}
