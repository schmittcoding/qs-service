use anyhow::{bail, Result};
use ran_models::events::category::EventCategory;
use ran_models::events::category_config::CategoryConfig;
use ran_models::events::event::{
    CreateEventRequest, DeductionStats, EventDashboard, EventDefinition, EventDefinitionAdmin, MatchStats, NextMatch, RegistrationStats, SchoolBreakdown, UpdateEventRequest
};
use uuid::Uuid;

use crate::webmanager::WebDB;

/// Editability guard data returned from the DB
struct EventGuard {
    pub reg_count: i32,
    pub match_count: i32,
    pub tallied_count: i32,
    #[allow(dead_code)]
    pub processing_count: i32,
    pub event: EventDefinition,
}

impl WebDB {
    // ──────────────────────── Create Event ────────────────────────

    pub async fn create_event(&self, req: &CreateEventRequest) -> Result<EventDefinition> {
        let mut client = self.pool.get().await?;

        // E2: EventName not empty
        if req.event_name.trim().is_empty() {
            bail!("VALIDATION:EventName is required");
        }

        // E4: Date sequence
        if req.registration_open >= req.registration_close {
            bail!("INVALID_DATES:RegistrationOpen must be before RegistrationClose");
        }
        if req.registration_close > req.event_start {
            bail!("INVALID_DATES:RegistrationClose must be on or before EventStart");
        }
        if req.event_start >= req.event_end {
            bail!("INVALID_DATES:EventStart must be before EventEnd");
        }

        if req.min_level <= 0 {
            bail!("VALIDATION:MinLevel must be greater than 0");
        }

        // E3: Season uniqueness among active events
        let season_check = client
            .query(
                "SELECT COUNT(*) AS Cnt FROM EventDefinition WHERE Season = @P1 AND IsActive = 1",
                &[&req.season],
            )
            .await?
            .into_row()
            .await?;

        if let Some(row) = season_check {
            if row.get::<i32, _>(0).unwrap_or(0) > 0 {
                bail!("SEASON_CONFLICT:An active event for Season {} already exists", req.season);
            }
        }

        let event_id = Uuid::new_v4();
        let category = req
            .event_category
            .as_ref()
            .unwrap_or(&EventCategory::Gvg);
        let category_str = category.to_string();

        // Extract flat values from typed category_config
        let (match_duration_min, tally_delay_min, min_guild_members, event_map_mid, event_map_sid) =
            match &req.category_config {
                Some(CategoryConfig::Gvg(c)) => (c.match_duration_min, c.tally_delay_min, c.min_guild_members, c.event_map_mid, c.event_map_sid),
                Some(CategoryConfig::Koth(c)) => (c.match_duration_min, c.tally_delay_min, 0, c.event_map_mid, c.event_map_sid),
                Some(CategoryConfig::GvgKoth(c)) => (c.match_duration_min, c.tally_delay_min, c.min_guild_members, c.event_map_mid, c.event_map_sid),
                Some(CategoryConfig::LevelCapRace(_)) | Some(CategoryConfig::SpeedRun(_)) | None => (45, 30, 0, 0, 0),
            };

        // Validate timing constraints
        if match_duration_min <= 0 {
            bail!("VALIDATION:MatchDurationMin must be greater than 0");
        }
        if tally_delay_min <= 0 {
            bail!("VALIDATION:TallyDelayMin must be greater than 0");
        }

        // Serialize the typed CategoryConfig to JSON
        let category_config_json = serde_json::to_string(&req.category_config).unwrap_or_default();

        client
            .execute(
                "INSERT INTO EventDefinition
                    (EventID, EventName, EventSlug, Season, EventMapMID, EventMapSID,
                     MatchDurationMin, TallyDelayMin,
                     RegistrationOpen, RegistrationClose,
                     EventStart, EventEnd, MinLevel, IsActive,
                     MinGuildMembers,
                     EventCategory, CategoryConfig, CreatedAt)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13, 0, @P14, @P15, @P16, GETDATE())",
                &[
                    &event_id,
                    &req.event_name.as_str(),
                    &req.event_slug.as_str(),
                    &req.season,
                    &event_map_mid,
                    &event_map_sid,
                    &match_duration_min,
                    &tally_delay_min,
                    &req.registration_open,
                    &req.registration_close,
                    &req.event_start,
                    &req.event_end,
                    &req.min_level,
                    &min_guild_members,
                    &category_str.as_str(),
                    &category_config_json.as_str(),
                ],
            )
            .await?;

        // Return the created event
        let row = client
            .query(
                "SELECT * FROM EventDefinition WHERE EventID = @P1",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row
            .map(|r| EventDefinition::from_row(&r))
            .expect("Event was just inserted"))
    }

    // ──────────────────────── Update Event ────────────────────────

    pub async fn update_event(
        &self,
        event_id: Uuid,
        req: &UpdateEventRequest,
    ) -> Result<EventDefinition> {
        let guard = self.get_event_guard(event_id).await?;
        let event = &guard.event;
        let now = chrono::Local::now().naive_local();

        // Build SET clauses dynamically based on what's provided + editability rules
        let mut sets: Vec<String> = Vec::new();

        if let Some(ref name) = req.event_name {
            if name.trim().is_empty() {
                bail!("VALIDATION:EventName cannot be empty");
            }
            sets.push(format!("EventName = '{}'", name.replace('\'', "''")));
        }

        if let Some(reg_open) = req.registration_open {
            if guard.reg_count > 0 {
                bail!("LOCKED:RegistrationOpen cannot be changed after registrations exist");
            }
            sets.push(format!("RegistrationOpen = '{}'", reg_open));
        }

        if let Some(reg_close) = req.registration_close {
            if now > event.registration_close {
                bail!("LOCKED:RegistrationClose cannot be changed after registration has closed");
            }
            sets.push(format!("RegistrationClose = '{}'", reg_close));
        }

        if let Some(ev_start) = req.event_start {
            if guard.tallied_count > 0 {
                bail!("LOCKED:EventStart cannot be changed after matches have been tallied");
            }
            if now > event.event_start {
                bail!("LOCKED:EventStart cannot be changed after the event has started");
            }
            sets.push(format!("EventStart = '{}'", ev_start));
        }

        if let Some(ev_end) = req.event_end {
            if now > event.event_end {
                bail!("LOCKED:EventEnd cannot be changed after the event has ended");
            }
            sets.push(format!("EventEnd = '{}'", ev_end));
        }

        if let Some(min_level) = req.min_level {
            if guard.reg_count > 0 {
                bail!("LOCKED:MinLevel cannot be changed after registrations exist");
            }
            if min_level <= 0 {
                bail!("VALIDATION:MinLevel must be greater than 0");
            }
            sets.push(format!("MinLevel = {}", min_level));
        }

        if let Some(ref cat) = req.event_category {
            if guard.tallied_count > 0 {
                bail!("LOCKED:EventCategory cannot be changed after matches have been tallied");
            }
            if req.category_config.is_none() {
                bail!("VALIDATION:category_config must be supplied when changing event_category");
            }
            sets.push(format!(
                "EventCategory = '{}'",
                cat.to_string().replace('\'', "''")
            ));
        }

        if let Some(ref config) = req.category_config {
            // Extract flat values from typed category_config
            let (match_duration_min, tally_delay_min, min_guild_members, event_map_mid, event_map_sid) =
                match config {
                    CategoryConfig::Gvg(c) => (c.match_duration_min, c.tally_delay_min, c.min_guild_members, c.event_map_mid, c.event_map_sid),
                    CategoryConfig::Koth(c) => (c.match_duration_min, c.tally_delay_min, 0, c.event_map_mid, c.event_map_sid),
                    CategoryConfig::GvgKoth(c) => (c.match_duration_min, c.tally_delay_min, c.min_guild_members, c.event_map_mid, c.event_map_sid),
                    CategoryConfig::LevelCapRace(_) | CategoryConfig::SpeedRun(_) => (45, 30, 0, 0, 0),
                };

            // Validate timing constraints
            if match_duration_min <= 0 {
                bail!("VALIDATION:MatchDurationMin must be greater than 0");
            }
            if tally_delay_min <= 0 {
                bail!("VALIDATION:TallyDelayMin must be greater than 0");
            }
            if min_guild_members < 0 {
                bail!("VALIDATION:MinGuildMembers must be 0 or greater");
            }

            // Check if match_duration_min is being changed (lock guard)
            let current_duration = match &event.category_config {
                Some(CategoryConfig::Gvg(gc)) => gc.match_duration_min,
                Some(CategoryConfig::Koth(kc)) => kc.match_duration_min,
                Some(CategoryConfig::GvgKoth(gkc)) => gkc.match_duration_min,
                _ => 45,
            };
            // Get the new duration
            let new_duration = match config {
                CategoryConfig::Gvg(gc) => gc.match_duration_min,
                CategoryConfig::Koth(kc) => kc.match_duration_min,
                CategoryConfig::GvgKoth(gkc) => gkc.match_duration_min,
                _ => 45,
            };
            if current_duration != new_duration && guard.match_count > 0 {
                bail!("LOCKED:MatchDurationMin cannot be changed after matches exist");
            }

            // Serialize and set all flat columns
            sets.push(format!("MatchDurationMin = {}", match_duration_min));
            sets.push(format!("TallyDelayMin = {}", tally_delay_min));
            sets.push(format!("MinGuildMembers = {}", min_guild_members));
            sets.push(format!("EventMapMID = {}", event_map_mid));
            sets.push(format!("EventMapSID = {}", event_map_sid));

            let json_str = serde_json::to_string(config).unwrap_or_default();
            sets.push(format!(
                "CategoryConfig = '{}'",
                json_str.replace('\'', "''")
            ));
        }

        if let Some(ref slug) = req.event_slug {
            if slug.trim().is_empty() {
                bail!("VALIDATION:EventSlug cannot be empty");
            }
            sets.push(format!("EventSlug = '{}'", slug.replace('\'', "''")));
        }

        if sets.is_empty() {
            bail!("VALIDATION:No fields to update");
        }

        let sql = format!(
            "UPDATE EventDefinition SET {} WHERE EventID = @P1",
            sets.join(", ")
        );

        let mut client = self.pool.get().await?;
        client.execute(sql.as_str(), &[&event_id]).await?;

        // Validate date sequence on the updated record
        let updated = self.get_event_by_id(event_id).await?;
        if let Some(ref ev) = updated {
            if ev.registration_open >= ev.registration_close
                || ev.registration_close > ev.event_start
                || ev.event_start >= ev.event_end
            {
                // Revert the update by restoring previous values
                client
                    .execute(
                        "UPDATE EventDefinition SET
                            RegistrationOpen = @P2, RegistrationClose = @P3,
                            EventStart = @P4, EventEnd = @P5
                         WHERE EventID = @P1",
                        &[
                            &event_id,
                            &event.registration_open,
                            &event.registration_close,
                            &event.event_start,
                            &event.event_end,
                        ],
                    )
                    .await?;
                bail!("INVALID_DATES:Date sequence is invalid after update. Registration must open before it closes, and close before the event starts.");
            }
        }

        Ok(updated.expect("Event exists"))
    }

    // ──────────────────────── Activate / Deactivate ────────────────────────

    pub async fn activate_event(&self, event_id: Uuid) -> Result<EventDefinition> {
        let event = self
            .get_event_by_id(event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        let mut client = self.pool.get().await?;

        if event.event_category.is_match_based() {
            let result = client
                .execute(
                    "UPDATE EventDefinition SET IsActive = 1
                     WHERE EventID = @P1
                       AND EXISTS (SELECT 1 FROM MatchSchedule WHERE EventID = @P1)",
                    &[&event_id],
                )
                .await?;

            if result.total() == 0 {
                bail!("NO_MATCHES:Add at least one match before activating the event");
            }
        } else {
            client
                .execute(
                    "UPDATE EventDefinition SET IsActive = 1 WHERE EventID = @P1",
                    &[&event_id],
                )
                .await?;
        }

        Ok(self.get_event_by_id(event_id).await?.expect("Event exists"))
    }

    pub async fn deactivate_event(&self, event_id: Uuid) -> Result<EventDefinition> {
        let mut client = self.pool.get().await?;

        // No matches currently Processing
        let result = client
            .execute(
                "UPDATE EventDefinition SET IsActive = 0
                 WHERE EventID = @P1
                   AND NOT EXISTS (
                       SELECT 1 FROM MatchSchedule
                       WHERE EventID = @P1 AND TallyStatus = 'Processing'
                   )",
                &[&event_id],
            )
            .await?;

        if result.total() == 0 {
            let exists = self.get_event_by_id(event_id).await?;
            if exists.is_none() {
                bail!("NOT_FOUND:Event not found");
            }
            bail!("TALLY_IN_PROGRESS:A tally is currently running. Wait for it to complete.");
        }

        Ok(self.get_event_by_id(event_id).await?.expect("Event exists"))
    }

    // ──────────────────────── List Events ────────────────────────

    pub async fn list_events(&self) -> Result<Vec<EventDefinitionAdmin>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT
                    ED.*,
                    (
                        SELECT COUNT(*)
                        FROM EventRegistration ER
                        WHERE ER.EventID = ED.EventID
                    ) AS RegistrationCount,
                    (
                        SELECT COUNT(*)
                        FROM MatchSchedule MS
                        WHERE MS.EventID = ED.EventID
                    ) AS MatchCount
                FROM EventDefinition ED
                ORDER BY ED.CreatedAt DESC;",
                &[],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(EventDefinitionAdmin::from_row).collect())
    }

    // ──────────────────────── Dashboard ────────────────────────

    pub async fn get_event_dashboard(&self, event_id: Uuid) -> Result<EventDashboard> {
        let event = self
            .get_event_by_id(event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        let mut client = self.pool.get().await?;

        // Registration stats
        let reg_row = client
            .query(
                "SELECT
                    COUNT(*) AS TotalRegistered,
                    SUM(CASE WHEN Status = 1 THEN 1 ELSE 0 END) AS Active,
                    SUM(CASE WHEN Status = 2 THEN 1 ELSE 0 END) AS Withdrawn,
                    SUM(CASE WHEN Status = 3 THEN 1 ELSE 0 END) AS Disqualified,
                    COUNT(DISTINCT GuildNum) AS GuildCount
                 FROM EventRegistration
                 WHERE EventID = @P1",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        let (total_registered, active, withdrawn, disqualified, guild_count) =
            match reg_row {
                Some(ref r) => (
                    r.get::<i32, _>("TotalRegistered").unwrap_or(0),
                    r.get::<i32, _>("Active").unwrap_or(0),
                    r.get::<i32, _>("Withdrawn").unwrap_or(0),
                    r.get::<i32, _>("Disqualified").unwrap_or(0),
                    r.get::<i32, _>("GuildCount").unwrap_or(0),
                ),
                None => (0, 0, 0, 0, 0),
            };

        // School breakdown
        let school_rows = client
            .query(
                "SELECT ChaSchool, COUNT(*) AS Count
                 FROM EventRegistration
                 WHERE EventID = @P1 AND Status = 1
                 GROUP BY ChaSchool",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        let school_breakdown: Vec<SchoolBreakdown> = school_rows
            .iter()
            .map(|r| SchoolBreakdown {
                cha_school: r.get::<i32, _>("ChaSchool").unwrap_or(0),
                count: r.get::<i32, _>("Count").unwrap_or(0),
            })
            .collect();

        // Match stats
        let match_row = client
            .query(
                "SELECT
                    COUNT(*) AS TotalMatches,
                    SUM(CASE WHEN TallyStatus = 'Completed' THEN 1 ELSE 0 END) AS Completed,
                    SUM(CASE WHEN TallyStatus IN ('Pending','Waiting') THEN 1 ELSE 0 END) AS Pending
                 FROM MatchSchedule
                 WHERE EventID = @P1",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        let (total_matches, completed, pending) = match match_row {
            Some(ref r) => (
                r.get::<i32, _>("TotalMatches").unwrap_or(0),
                r.get::<i32, _>("Completed").unwrap_or(0),
                r.get::<i32, _>("Pending").unwrap_or(0),
            ),
            None => (0, 0, 0),
        };

        // Next match
        let next_row = client
            .query(
                "SELECT TOP 1 MatchID, MatchLabel, StartTime
                 FROM MatchSchedule
                 WHERE EventID = @P1 AND TallyStatus IN ('Pending','Waiting')
                   AND StartTime > GETDATE()
                 ORDER BY StartTime",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        let next_match = next_row.map(|r| NextMatch {
            match_id: r.get::<Uuid, _>("MatchID").unwrap_or_else(Uuid::nil),
            label: r.get::<&str, _>("MatchLabel").map(|s| s.to_string()),
            start_time: r
                .get::<chrono::NaiveDateTime, _>("StartTime")
                .unwrap_or_default(),
        });

        // Deduction stats
        let ded_row = client
            .query(
                "SELECT
                    COUNT(*) AS TotalDeductions,
                    COALESCE(SUM(CASE WHEN IsReverted = 0 THEN Points ELSE 0 END), 0) AS TotalPointsDeducted,
                    SUM(CASE WHEN IsReverted = 1 THEN 1 ELSE 0 END) AS Reverted
                 FROM AdminDeduction
                 WHERE EventID = @P1",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        let (total_deductions, total_points_deducted, reverted) = match ded_row {
            Some(ref r) => (
                r.get::<i32, _>("TotalDeductions").unwrap_or(0),
                r.get::<i32, _>("TotalPointsDeducted").unwrap_or(0),
                r.get::<i32, _>("Reverted").unwrap_or(0),
            ),
            None => (0, 0, 0),
        };

        Ok(EventDashboard {
            event,
            registration_stats: RegistrationStats {
                total_registered,
                active,
                withdrawn,
                disqualified,
                guild_count,
                school_breakdown,
            },
            match_stats: MatchStats {
                total_matches,
                completed,
                pending,
                next_match,
            },
            deduction_stats: DeductionStats {
                total_deductions,
                total_points_deducted,
                reverted,
            },
        })
    }

    // ──────────────────────── Guard helper ────────────────────────

    async fn get_event_guard(&self, event_id: Uuid) -> Result<EventGuard> {
        let mut client = self.pool.get().await?;

        let row = client
            .query(
                "SELECT
                    (SELECT COUNT(*) FROM EventRegistration WHERE EventID = @P1) AS RegCount,
                    (SELECT COUNT(*) FROM MatchSchedule WHERE EventID = @P1) AS MatchCount,
                    (SELECT COUNT(*) FROM MatchSchedule WHERE EventID = @P1 AND TallyStatus = 'Completed') AS TalliedCount,
                    (SELECT COUNT(*) FROM MatchSchedule WHERE EventID = @P1 AND TallyStatus = 'Processing') AS ProcessingCount",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        let (reg_count, match_count, tallied_count, processing_count) = match row {
            Some(ref r) => (
                r.get::<i32, _>("RegCount").unwrap_or(0),
                r.get::<i32, _>("MatchCount").unwrap_or(0),
                r.get::<i32, _>("TalliedCount").unwrap_or(0),
                r.get::<i32, _>("ProcessingCount").unwrap_or(0),
            ),
            None => bail!("NOT_FOUND:Event not found"),
        };

        let event = self
            .get_event_by_id(event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        Ok(EventGuard {
            reg_count,
            match_count,
            tallied_count,
            processing_count,
            event,
        })
    }
}
