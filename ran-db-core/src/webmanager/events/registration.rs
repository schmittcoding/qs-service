use anyhow::Result;
use ran_models::events::registration::EventRegistration;
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// Insert a new event registration
    pub async fn insert_event_registration(
        &self,
        event_id: Uuid,
        cha_num: i32,
        user_num: i32,
        guild_num: Option<i32>,
        cha_school: Option<i32>,
        cha_class: i32,
        cha_level: i32,
        ip_address: Option<String>,
    ) -> Result<EventRegistration> {
        let mut client = self.pool.get().await?;
        let registration_id = Uuid::new_v4();

        let row = client
            .query(
                "INSERT INTO EventRegistration
                    (RegistrationID, EventID, ChaNum, UserNum, GuildNum, ChaSchool, ChaClass, ChaLevel, Status, RegisteredAt, IPAddress)
                VALUES
                    (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, 1, GETDATE(), @P9);
                SELECT * FROM EventRegistration WHERE RegistrationID = @P1",
                &[
                    &registration_id,
                    &event_id,
                    &cha_num,
                    &user_num,
                    &guild_num,
                    &cha_school,
                    &cha_class,
                    &cha_level,
                    &ip_address,
                ],
            )
            .await?
            .into_row()
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to insert registration"))?;

        Ok(EventRegistration::from_row(&row))
    }

    /// Check if a character is already registered for an event
    pub async fn get_registration_by_event_and_char(
        &self,
        event_id: Uuid,
        cha_num: i32,
    ) -> Result<Option<EventRegistration>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT * FROM EventRegistration WHERE EventID = @P1 AND ChaNum = @P2",
                &[&event_id, &cha_num],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| EventRegistration::from_row(&r)))
    }

    /// Get all registrations for a user in an event
    pub async fn get_registrations_by_user(
        &self,
        event_id: Uuid,
        user_num: i32,
    ) -> Result<Vec<EventRegistration>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM EventRegistration WHERE EventID = @P1 AND UserNum = @P2 ORDER BY RegisteredAt DESC",
                &[&event_id, &user_num],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(EventRegistration::from_row).collect())
    }

    /// Count active registrations for a user in an event (for account limit V6)
    pub async fn count_active_registrations(
        &self,
        event_id: Uuid,
        user_num: i32,
    ) -> Result<i32> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT COUNT(*) AS Cnt FROM EventRegistration WHERE EventID = @P1 AND UserNum = @P2 AND Status = 1",
                &[&event_id, &user_num],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.and_then(|r| r.get::<i32, _>("Cnt")).unwrap_or(0))
    }

    /// Check the school of existing guild members registered in this event.
    /// Returns the ChaSchool of existing members if any are registered, or None.
    pub async fn get_guild_registered_school(
        &self,
        event_id: Uuid,
        guild_num: i32,
    ) -> Result<Option<i32>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT TOP 1 ChaSchool FROM EventRegistration
                 WHERE EventID = @P1 AND GuildNum = @P2 AND Status = 1
                 AND ChaSchool IS NOT NULL",
                &[&event_id, &guild_num],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.and_then(|r| r.get::<i32, _>("ChaSchool")))
    }

    /// List all participants by character for an event (admin view)
    pub async fn list_participants_by_character(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<tiberius::Row>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT er.RegistrationID, er.ChaNum, er.UserNum, er.GuildNum, er.ChaSchool,
                        er.ChaClass, er.ChaLevel, er.Status,
                        er.RegisteredAt, er.UpdatedAt,
                        c.ChaName,
                        g.GuName AS GuildName
                 FROM EventRegistration er
                 LEFT JOIN [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = er.ChaNum
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
                 WHERE er.EventID = @P1
                 ORDER BY er.GuildNum, er.ChaNum",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows)
    }

    /// List participants grouped by guild for an event (admin view)
    pub async fn list_participants_by_guild(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<tiberius::Row>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT er.GuildNum,
                        g.GuName AS GuildName,
                        er.ChaSchool,
                        COUNT(*) AS MemberCount,
                        SUM(CASE WHEN er.Status = 1 THEN 1 ELSE 0 END) AS ActiveCount,
                        SUM(CASE WHEN er.Status = 2 THEN 1 ELSE 0 END) AS WithdrawnCount,
                        SUM(CASE WHEN er.Status = 3 THEN 1 ELSE 0 END) AS DisqualifiedCount
                 FROM EventRegistration er
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
                 WHERE er.EventID = @P1 AND er.GuildNum <> 0
                 GROUP BY er.GuildNum, g.GuName, er.ChaSchool
                 ORDER BY MemberCount DESC",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows)
    }

    /// Public paginated character participants (active only, with optional class filter)
    pub async fn get_public_participants_by_character(
        &self,
        event_id: Uuid,
        cha_class: Option<i32>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<tiberius::Row>, i64)> {
        let mut client = self.pool.get().await?;
        let offset = (page.saturating_sub(1)) * page_size;

        let (count_query, data_query) = match cha_class {
            Some(_) => (
                "SELECT COUNT(*) AS Total FROM EventRegistration WHERE EventID = @P1 AND Status = 1 AND ChaClass = @P2",
                "SELECT er.ChaNum, er.GuildNum, er.ChaSchool, er.ChaClass, er.ChaLevel,
                        c.ChaName,
                        g.GuName AS GuildName,
                        ROW_NUMBER() OVER (ORDER BY er.ChaLevel DESC, er.ChaNum) AS Rank
                 FROM EventRegistration er
                 LEFT JOIN [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = er.ChaNum
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
                 WHERE er.EventID = @P1 AND er.Status = 1 AND er.ChaClass = @P2
                 ORDER BY er.ChaLevel DESC, er.ChaNum
                 OFFSET @P3 ROWS FETCH NEXT @P4 ROWS ONLY",
            ),
            None => (
                "SELECT COUNT(*) AS Total FROM EventRegistration WHERE EventID = @P1 AND Status = 1 AND @P2 IS NULL",
                "SELECT er.ChaNum, er.GuildNum, er.ChaSchool, er.ChaClass, er.ChaLevel,
                        c.ChaName,
                        g.GuName AS GuildName,
                        ROW_NUMBER() OVER (ORDER BY er.ChaLevel DESC, er.ChaNum) AS Rank
                 FROM EventRegistration er
                 LEFT JOIN [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = er.ChaNum
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
                 WHERE er.EventID = @P1 AND er.Status = 1 AND @P2 IS NULL
                 ORDER BY er.ChaLevel DESC, er.ChaNum
                 OFFSET @P3 ROWS FETCH NEXT @P4 ROWS ONLY",
            ),
        };

        let count_row = client
            .query(count_query, &[&event_id, &cha_class])
            .await?
            .into_row()
            .await?;
        let total: i64 = count_row
            .and_then(|r| r.get::<i32, _>("Total").map(|v| v as i64))
            .unwrap_or(0);

        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                data_query,
                &[&event_id, &cha_class, &(offset as i32), &(page_size as i32)],
            )
            .await?
            .into_first_result()
            .await?;

        Ok((rows, total))
    }

    /// Public paginated guild participants (active members only)
    pub async fn get_public_participants_by_guild(
        &self,
        event_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<tiberius::Row>, i64)> {
        let mut client = self.pool.get().await?;
        let offset = (page.saturating_sub(1)) * page_size;

        let count_row = client
            .query(
                "SELECT COUNT(DISTINCT er.GuildNum) AS Total
                 FROM EventRegistration er
                 WHERE er.EventID = @P1 AND er.Status = 1 AND er.GuildNum <> 0",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;
        let total: i64 = count_row
            .and_then(|r| r.get::<i32, _>("Total").map(|v| v as i64))
            .unwrap_or(0);

        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT er.GuildNum,
                        g.GuName AS GuildName,
                        er.ChaSchool,
                        COUNT(*) AS MemberCount,
                        ROW_NUMBER() OVER (ORDER BY COUNT(*) DESC) AS Rank
                 FROM EventRegistration er
                 LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
                 WHERE er.EventID = @P1 AND er.Status = 1 AND er.GuildNum <> 0
                 GROUP BY er.GuildNum, g.GuName, er.ChaSchool
                 ORDER BY MemberCount DESC
                 OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY",
                &[&event_id, &(offset as i32), &(page_size as i32)],
            )
            .await?
            .into_first_result()
            .await?;

        Ok((rows, total))
    }

    /// Admin update a player's registration status (e.g., withdraw/disqualify a player who left their guild)
    pub async fn admin_update_registration(
        &self,
        event_id: Uuid,
        cha_num: i32,
        status: Option<i16>,
        guild_num: Option<Option<i32>>,
    ) -> Result<Option<EventRegistration>> {
        let mut client = self.pool.get().await?;

        let mut sets: Vec<String> = Vec::new();

        if let Some(s) = status {
            if !(1..=3).contains(&s) {
                anyhow::bail!("VALIDATION:Status must be 1 (Active), 2 (Withdrawn), or 3 (Disqualified)");
            }
            sets.push(format!("Status = {}", s));
        }

        if let Some(gn) = guild_num {
            match gn {
                Some(g) => sets.push(format!("GuildNum = {}", g)),
                None => sets.push("GuildNum = NULL".to_string()),
            }
        }

        if sets.is_empty() {
            anyhow::bail!("VALIDATION:No fields to update");
        }

        sets.push("UpdatedAt = GETDATE()".to_string());

        let sql = format!(
            "UPDATE EventRegistration SET {} WHERE EventID = @P1 AND ChaNum = @P2;
             SELECT * FROM EventRegistration WHERE EventID = @P1 AND ChaNum = @P2",
            sets.join(", ")
        );

        let row = client
            .query(&sql, &[&event_id, &cha_num])
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| EventRegistration::from_row(&r)))
    }

    /// Withdraw registration (set Status = 2)
    pub async fn withdraw_registration(
        &self,
        event_id: Uuid,
        cha_num: i32,
        user_num: i32,
    ) -> Result<bool> {
        let mut client = self.pool.get().await?;
        let rows_affected = client
            .execute(
                "DELETE FROM EventRegistration
                WHERE EventID = @P1 AND ChaNum = @P2 AND UserNum = @P3",
                &[&event_id, &cha_num, &user_num],
            )
            .await?
            .total();

        Ok(rows_affected > 0)
    }

    /// Admin remove registration (no user ownership check, admin bypass)
    pub async fn admin_remove_registration(
        &self,
        event_id: Uuid,
        cha_num: i32,
    ) -> Result<bool> {
        let mut client = self.pool.get().await?;
        let rows_affected = client
            .execute(
                "DELETE FROM EventRegistration WHERE EventID = @P1 AND ChaNum = @P2",
                &[&event_id, &cha_num],
            )
            .await?
            .total();

        Ok(rows_affected > 0)
    }
}
