use anyhow::{bail, Context, Result};
use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};
use ran_models::events::category_config::CategoryConfig;
use ran_models::events::event::{CreateMatchRequest, GenerateMatchesRequest, UpdateMatchRequest};
use ran_models::events::match_result::{MatchGuildResultExport, MatchKothResultExport};
use ran_models::events::match_schedule::MatchSchedule;
use uuid::Uuid;

use crate::webmanager::WebDB;

/// Extract match timing from event's category_config
fn match_timing_from_event(event: &ran_models::events::event::EventDefinition) -> (i32, i32) {
    match &event.category_config {
        Some(CategoryConfig::Gvg(c)) => (c.match_duration_min, c.tally_delay_min),
        Some(CategoryConfig::Koth(c)) => (c.match_duration_min, c.tally_delay_min),
        Some(CategoryConfig::GvgKoth(c)) => (c.match_duration_min, c.tally_delay_min),
        _ => (45, 30),
    }
}

impl WebDB {
    // ──────────────────────── Create Single Match ────────────────────────

    pub async fn create_match(
        &self,
        event_id: Uuid,
        req: &CreateMatchRequest,
    ) -> Result<MatchSchedule> {
        let event = self
            .get_event_by_id(event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        let (match_duration_min, tally_delay_min) = match_timing_from_event(&event);
        let end_time = req.start_time + Duration::minutes(match_duration_min as i64);
        let tally_scheduled = end_time + Duration::minutes(tally_delay_min as i64);

        // M2: StartTime within event range
        if req.start_time < event.event_start || req.start_time > event.event_end {
            bail!("OUT_OF_EVENT_RANGE:Match start time must be within the event date range");
        }

        // M3: Overlap check
        self.check_match_overlap(event_id, req.start_time, end_time, None)
            .await?;

        let match_id = Uuid::new_v4();
        let mut client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO MatchSchedule
                    (MatchID, EventID, MatchLabel, MatchDate, StartTime, EndTime, TallyScheduled, TallyStatus,
                     TowerPointsTH, TowerPointsNuc, TowerPointsFaci)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, 'Pending', @P8, @P9, @P10)",
                &[
                    &match_id,
                    &event_id,
                    &req.match_label.as_deref(),
                    &req.match_date,
                    &req.start_time,
                    &end_time,
                    &tally_scheduled,
                    &req.tower_points_th,
                    &req.tower_points_nuc,
                    &req.tower_points_faci,
                ],
            )
            .await?;

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
            .expect("Match was just inserted"))
    }

    // ──────────────────────── Create Bulk Matches ────────────────────────

    pub async fn create_matches_bulk(
        &self,
        event_id: Uuid,
        requests: &[CreateMatchRequest],
    ) -> Result<Vec<MatchSchedule>> {
        let mut results = Vec::with_capacity(requests.len());

        for req in requests {
            let m = self.create_match(event_id, req).await?;
            results.push(m);
        }

        Ok(results)
    }

    // ──────────────────────── Auto-Generate Matches ────────────────────────

    pub async fn generate_matches(
        &self,
        event_id: Uuid,
        req: &GenerateMatchesRequest,
    ) -> Result<Vec<MatchSchedule>> {
        let _event = self
            .get_event_by_id(event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        if req.days_of_week.is_empty() {
            bail!("VALIDATION:daysOfWeek must not be empty");
        }
        if req.start_date > req.end_date {
            bail!("VALIDATION:startDate must be on or before endDate");
        }

        // Parse target weekdays
        let target_weekdays: Vec<Weekday> = req
            .days_of_week
            .iter()
            .map(|d| parse_weekday(d))
            .collect::<Result<Vec<_>>>()?;

        // The first day in daysOfWeek determines when week number increments
        let first_weekday = target_weekdays[0];

        let time = NaiveTime::from_hms_opt(req.start_hour, req.start_minute, 0)
            .ok_or_else(|| anyhow::anyhow!("VALIDATION:Invalid start_hour/start_minute"))?;

        let mut matches_to_create: Vec<CreateMatchRequest> = Vec::new();
        let mut current_date = req.start_date;
        let mut week_num = 1u32;
        let mut seen_first_weekday = false;

        while current_date <= req.end_date {
            let weekday = current_date.weekday();

            // Track week number: increments when we see the first weekday again
            if weekday == first_weekday {
                if seen_first_weekday {
                    week_num += 1;
                }
                seen_first_weekday = true;
            }

            if target_weekdays.contains(&weekday) {
                let start_time = NaiveDateTime::new(current_date, time);
                let day_name = weekday_name(weekday);
                let label = format!("{} {} - {}", req.label_prefix, week_num, day_name);

                matches_to_create.push(CreateMatchRequest {
                    match_label: Some(label),
                    match_date: current_date,
                    start_time,
                    tower_points_th: 0,
                    tower_points_nuc: 0,
                    tower_points_faci: 0,
                });
            }

            current_date += Duration::days(1);
        }

        // Safety limit
        if matches_to_create.len() > 50 {
            bail!("VALIDATION:Cannot generate more than 50 matches at once ({} would be created)", matches_to_create.len());
        }

        if matches_to_create.is_empty() {
            bail!("VALIDATION:No matches would be generated for the given date range and days of week");
        }

        self.create_matches_bulk(event_id, &matches_to_create).await
    }

    // ──────────────────────── Update Match ────────────────────────

    pub async fn update_match(
        &self,
        match_id: Uuid,
        req: &UpdateMatchRequest,
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
            bail!("MATCH_LOCKED:Cannot modify a match that has already been tallied or is in progress");
        }

        // Get event for duration/delay config
        let event = self
            .get_event_by_id(existing.event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("NOT_FOUND:Event not found"))?;

        // Handle start_time change (recomputes end_time, tally_scheduled, match_date)
        if let Some(start_time) = req.start_time {
            let (match_duration_min, tally_delay_min) = match_timing_from_event(&event);
            let end_time = start_time + Duration::minutes(match_duration_min as i64);
            let tally_scheduled = end_time + Duration::minutes(tally_delay_min as i64);

            // M2: within event range
            if start_time < event.event_start || start_time > event.event_end {
                bail!("OUT_OF_EVENT_RANGE:Match start time must be within the event date range");
            }

            // M3: overlap (excluding self)
            self.check_match_overlap(existing.event_id, start_time, end_time, Some(match_id))
                .await?;

            let match_date = start_time.date();
            let label = req
                .match_label
                .as_deref()
                .or(existing.match_label.as_deref());

            // Use a direct query for time changes since it involves multiple computed columns
            let tower_th = req.tower_points_th.unwrap_or(existing.tower_points_th);
            let tower_nuc = req.tower_points_nuc.unwrap_or(existing.tower_points_nuc);
            let tower_faci = req.tower_points_faci.unwrap_or(existing.tower_points_faci);

            client
                .execute(
                    "UPDATE MatchSchedule
                     SET MatchLabel = @P1, MatchDate = @P2, StartTime = @P3,
                         EndTime = @P4, TallyScheduled = @P5,
                         TowerPointsTH = @P6, TowerPointsNuc = @P7, TowerPointsFaci = @P8
                     WHERE MatchID = @P9 AND TallyStatus = 'Pending'",
                    &[
                        &label,
                        &match_date,
                        &start_time,
                        &end_time,
                        &tally_scheduled,
                        &tower_th,
                        &tower_nuc,
                        &tower_faci,
                        &match_id,
                    ],
                )
                .await?;
        } else {
            // No start_time change — update label and/or tower points
            let has_label = req.match_label.is_some();
            let has_tower = req.tower_points_th.is_some()
                || req.tower_points_nuc.is_some()
                || req.tower_points_faci.is_some();

            if has_label || has_tower {
                let label = req
                    .match_label
                    .as_deref()
                    .or(existing.match_label.as_deref());
                let tower_th = req.tower_points_th.unwrap_or(existing.tower_points_th);
                let tower_nuc = req.tower_points_nuc.unwrap_or(existing.tower_points_nuc);
                let tower_faci = req.tower_points_faci.unwrap_or(existing.tower_points_faci);

                client
                    .execute(
                        "UPDATE MatchSchedule
                         SET MatchLabel = @P1,
                             TowerPointsTH = @P2, TowerPointsNuc = @P3, TowerPointsFaci = @P4
                         WHERE MatchID = @P5 AND TallyStatus = 'Pending'",
                        &[
                            &label,
                            &tower_th,
                            &tower_nuc,
                            &tower_faci,
                            &match_id,
                        ],
                    )
                    .await?;
            }
        }

        // Return updated
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

    // ──────────────────────── Delete Match ────────────────────────

    pub async fn delete_match(&self, match_id: Uuid) -> Result<()> {
        let mut client = self.pool.get().await?;

        // Delete associated gimmicks
        client
            .execute(
                "DELETE FROM EventMatchGimmick WHERE MatchID = @P1",
                &[&match_id],
            )
            .await?;

        let result = client
            .execute(
                "DELETE FROM MatchSchedule WHERE MatchID = @P1 AND TallyStatus = 'Pending'",
                &[&match_id],
            )
            .await?;

        if result.total() == 0 {
            // Check if it exists
            let row = client
                .query(
                    "SELECT TallyStatus FROM MatchSchedule WHERE MatchID = @P1",
                    &[&match_id],
                )
                .await?
                .into_row()
                .await?;

            match row {
                Some(_) => bail!("MATCH_LOCKED:Cannot delete a match that has already been tallied"),
                None => bail!("NOT_FOUND:Match not found"),
            }
        }

        Ok(())
    }

    // ──────────────────────── List Matches (Admin) ────────────────────────

    pub async fn list_matches_admin(&self, event_id: Uuid) -> Result<Vec<MatchSchedule>> {
        self.get_matches_by_event(event_id).await
    }

    // ──────────────────────── Overlap Check ────────────────────────

    async fn check_match_overlap(
        &self,
        event_id: Uuid,
        proposed_start: NaiveDateTime,
        proposed_end: NaiveDateTime,
        exclude_match_id: Option<Uuid>,
    ) -> Result<()> {
        let mut client = self.pool.get().await?;

        let overlap_count = if let Some(exclude_id) = exclude_match_id {
            let row = client
                .query(
                    "SELECT COUNT(*) AS OverlapCount
                     FROM MatchSchedule
                     WHERE EventID = @P1
                       AND MatchID != @P4
                       AND (
                           (@P2 >= StartTime AND @P2 < EndTime)
                           OR (@P3 > StartTime AND @P3 <= EndTime)
                           OR (@P2 <= StartTime AND @P3 >= EndTime)
                       )",
                    &[&event_id, &proposed_start, &proposed_end, &exclude_id],
                )
                .await?
                .into_row()
                .await?;
            row
        } else {
            let row = client
                .query(
                    "SELECT COUNT(*) AS OverlapCount
                     FROM MatchSchedule
                     WHERE EventID = @P1
                       AND (
                           (@P2 >= StartTime AND @P2 < EndTime)
                           OR (@P3 > StartTime AND @P3 <= EndTime)
                           OR (@P2 <= StartTime AND @P3 >= EndTime)
                       )",
                    &[&event_id, &proposed_start, &proposed_end],
                )
                .await?
                .into_row()
                .await?;
            row
        };

        if let Some(row) = overlap_count {
            if row.get::<i32, _>("OverlapCount").unwrap_or(0) > 0 {
                bail!("MATCH_OVERLAP:This match overlaps with an existing match");
            }
        }

        Ok(())
    }

    // ──────────────────────── Guild Match Results Export ────────────────────────

    pub async fn get_guild_match_results_export(
        &self,
        event_id: Uuid,
        match_id: Uuid,
    ) -> Result<Vec<MatchGuildResultExport>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let rows = client
            .query(
                "SELECT
                    ms.MatchID, ms.MatchLabel, ms.MatchDate, ms.StartTime, ms.EndTime,
                    ms.TallyStatus, ms.RowsTallied, ms.KillsCounted, ms.KillsRejected,
                    mgr.GuildNum, mgr.GuildName, mgr.ChaSchool,
                    mgr.MatchKills, mgr.MatchDeaths, mgr.MatchTower
                 FROM MatchGuildResult mgr
                 JOIN MatchSchedule ms ON ms.MatchID = mgr.MatchID
                 WHERE ms.EventID = @P1 AND ms.MatchID = @P2
                 ORDER BY ms.MatchDate ASC, mgr.GuildName ASC",
                &[&event_id, &match_id],
            )
            .await
            .context("Failed to query guild match results")?
            .into_first_result()
            .await
            .context("Failed to fetch guild match result rows")?;

        let results = rows
            .iter()
            .map(|r| MatchGuildResultExport {
                match_id: r.get::<Uuid, _>("MatchID").unwrap_or_default(),
                match_label: r.get::<&str, _>("MatchLabel").map(|s| s.to_owned()),
                match_date: r.get::<chrono::NaiveDate, _>("MatchDate").unwrap_or_default(),
                start_time: r.get::<NaiveDateTime, _>("StartTime").unwrap_or_default(),
                end_time: r.get::<NaiveDateTime, _>("EndTime").unwrap_or_default(),
                tally_status: r.get::<&str, _>("TallyStatus").unwrap_or("").to_owned(),
                rows_tallied: r.get::<i32, _>("RowsTallied"),
                kills_counted: r.get::<i32, _>("KillsCounted"),
                kills_rejected: r.get::<i32, _>("KillsRejected"),
                guild_num: r.get::<i32, _>("GuildNum").unwrap_or_default(),
                guild_name: r.get::<&str, _>("GuildName").unwrap_or("").to_owned(),
                cha_school: r.get::<i32, _>("ChaSchool").unwrap_or_default(),
                match_kills: r.get::<i32, _>("MatchKills").unwrap_or_default(),
                match_deaths: r.get::<i32, _>("MatchDeaths").unwrap_or_default(),
                match_tower: r.get::<i32, _>("MatchTower").unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }

    // ──────────────────────── KOTH Match Results Export ────────────────────────

    pub async fn get_koth_match_results_export(
        &self,
        event_id: Uuid,
        match_id: Uuid,
    ) -> Result<Vec<MatchKothResultExport>> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;

        let rows = client
            .query(
                "SELECT
                    ms.MatchID, ms.MatchLabel, ms.MatchDate,
                    ms.StartTime, ms.EndTime, ms.TallyStatus,
                    mcr.ChaNum, mcr.ChaName, mcr.ChaClass,
                    mcr.GuildNum, mcr.GuildName,
                    mcr.MatchKills, mcr.MatchDeaths
                 FROM MatchCharResult mcr
                 JOIN MatchSchedule ms ON ms.MatchID = mcr.MatchID
                 WHERE ms.EventID = @P1 AND ms.MatchID = @P2
                 ORDER BY ms.MatchDate ASC, mcr.ChaName ASC",
                &[&event_id, &match_id],
            )
            .await
            .context("Failed to query KOTH match results")?
            .into_first_result()
            .await
            .context("Failed to fetch KOTH match result rows")?;

        let results = rows
            .iter()
            .map(|r| MatchKothResultExport {
                match_id: r.get::<Uuid, _>("MatchID").unwrap_or_default(),
                match_label: r.get::<&str, _>("MatchLabel").map(|s| s.to_owned()),
                match_date: r.get::<chrono::NaiveDate, _>("MatchDate").unwrap_or_default(),
                start_time: r.get::<NaiveDateTime, _>("StartTime").unwrap_or_default(),
                end_time: r.get::<NaiveDateTime, _>("EndTime").unwrap_or_default(),
                tally_status: r.get::<&str, _>("TallyStatus").unwrap_or("").to_owned(),
                cha_num: r.get::<i32, _>("ChaNum").unwrap_or_default(),
                cha_name: r.get::<&str, _>("ChaName").unwrap_or("").to_owned(),
                cha_class: r.get::<i32, _>("ChaClass").unwrap_or_default(),
                guild_num: r.get::<i32, _>("GuildNum"),
                guild_name: r.get::<&str, _>("GuildName").map(|s| s.to_owned()),
                match_kills: r.get::<i32, _>("MatchKills").unwrap_or_default(),
                match_deaths: r.get::<i32, _>("MatchDeaths").unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }
}

// ──────────────────────── Helpers ────────────────────────

fn parse_weekday(s: &str) -> Result<Weekday> {
    match s.to_lowercase().as_str() {
        "monday" | "mon" => Ok(Weekday::Mon),
        "tuesday" | "tue" => Ok(Weekday::Tue),
        "wednesday" | "wed" => Ok(Weekday::Wed),
        "thursday" | "thu" => Ok(Weekday::Thu),
        "friday" | "fri" => Ok(Weekday::Fri),
        "saturday" | "sat" => Ok(Weekday::Sat),
        "sunday" | "sun" => Ok(Weekday::Sun),
        _ => bail!("VALIDATION:Invalid day of week: {}", s),
    }
}

fn weekday_name(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}
