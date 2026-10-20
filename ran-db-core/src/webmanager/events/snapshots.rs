use anyhow::Result;
use ran_models::events::snapshot::{
    EventSnapshotWithSchedule, SnapshotGuildScore, SnapshotKOTHScore, SnapshotLevelCapRaceScore,
    SnapshotSchoolScore,
};
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// List all snapshots for an event with their associated match schedules (summary only)
    pub async fn get_snapshots(&self, event_id: Uuid) -> Result<Vec<EventSnapshotWithSchedule>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT
                    s.SnapshotID,
                    s.EventID,
                    s.MatchID,
                    s.SnapshotTime,
                    s.Label,
                    s.CreatedAt,
                    m.MatchID         AS MatchScheduleID,
                    m.MatchDate,
                    m.StartTime,
                    m.EndTime,
                    m.TowerPointsTH,
                    m.TowerPointsFaci,
                    m.TowerPointsNuc,
                    m.TowerWinnerTH,
                    m.TowerWinnerFaci,
                    m.TowerWinnerNuc,
                    m.KillsCounted
                 FROM EventSnapshot s
                 LEFT JOIN MatchSchedule m ON m.MatchID = s.MatchID
                 WHERE s.EventID = @P1
                 ORDER BY s.SnapshotTime DESC",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(EventSnapshotWithSchedule::from_joined_row).collect())
    }

    /// Get guild scores for a snapshot, with rank_change compared to the previous snapshot
    pub async fn get_snapshot_guild_scores(
        &self,
        snapshot_id: Uuid,
    ) -> Result<Vec<SnapshotGuildScore>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "WITH PrevSnapshot AS (
                    SELECT TOP 1 ps.SnapshotID
                    FROM EventSnapshot ps
                    INNER JOIN EventSnapshot cs ON cs.SnapshotID = @P1
                        AND ps.EventID = cs.EventID
                        AND ps.SnapshotTime < cs.SnapshotTime
                    ORDER BY ps.SnapshotTime DESC
                )
                SELECT
                    cur.*,
                    CASE
                        WHEN prev.Rank IS NULL THEN NULL
                        ELSE prev.Rank - cur.Rank
                    END AS RankChange
                FROM SnapshotGuildScore cur
                LEFT JOIN PrevSnapshot ON 1=1
                LEFT JOIN SnapshotGuildScore prev
                    ON prev.SnapshotID = PrevSnapshot.SnapshotID
                    AND prev.GuildNum = cur.GuildNum
                WHERE cur.SnapshotID = @P1
                ORDER BY cur.Rank",
                &[&snapshot_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(SnapshotGuildScore::from_row).collect())
    }

    /// Get KOTH scores for a snapshot, with rank_change compared to the previous snapshot
    pub async fn get_snapshot_koth_scores(
        &self,
        snapshot_id: Uuid,
        cha_class: Option<i32>,
    ) -> Result<Vec<SnapshotKOTHScore>> {
        let mut client = self.pool.get().await?;

        let base_query = "WITH PrevSnapshot AS (
                    SELECT TOP 1 ps.SnapshotID
                    FROM EventSnapshot ps
                    INNER JOIN EventSnapshot cs ON cs.SnapshotID = @P1
                        AND ps.EventID = cs.EventID
                        AND ps.SnapshotTime < cs.SnapshotTime
                    ORDER BY ps.SnapshotTime DESC
                )
                SELECT
                    cur.*,
                    CASE
                        WHEN prev.Rank IS NULL THEN NULL
                        ELSE prev.Rank - cur.Rank
                    END AS RankChange
                FROM SnapshotKOTHScore cur
                LEFT JOIN PrevSnapshot ON 1=1
                LEFT JOIN SnapshotKOTHScore prev
                    ON prev.SnapshotID = PrevSnapshot.SnapshotID
                    AND prev.ChaNum = cur.ChaNum
                WHERE cur.SnapshotID = @P1";

        let rows = match cha_class {
            Some(class) => {
                client
                    .query(
                        &format!("{} AND cur.ChaClass = @P2 ORDER BY cur.Rank", base_query),
                        &[&snapshot_id, &class],
                    )
                    .await?
                    .into_first_result()
                    .await?
            }
            None => {
                client
                    .query(
                        &format!("{} ORDER BY cur.Rank", base_query),
                        &[&snapshot_id],
                    )
                    .await?
                    .into_first_result()
                    .await?
            }
        };

        Ok(rows.iter().map(SnapshotKOTHScore::from_row).collect())
    }

    /// Get school scores for a snapshot, with rank_change compared to the previous snapshot
    pub async fn get_snapshot_school_scores(
        &self,
        snapshot_id: Uuid,
    ) -> Result<Vec<SnapshotSchoolScore>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "WITH PrevSnapshot AS (
                    SELECT TOP 1 ps.SnapshotID
                    FROM EventSnapshot ps
                    INNER JOIN EventSnapshot cs ON cs.SnapshotID = @P1
                        AND ps.EventID = cs.EventID
                        AND ps.SnapshotTime < cs.SnapshotTime
                    ORDER BY ps.SnapshotTime DESC
                )
                SELECT
                    cur.*,
                    CASE
                        WHEN prev.Rank IS NULL THEN NULL
                        ELSE prev.Rank - cur.Rank
                    END AS RankChange
                FROM SnapshotSchoolScore cur
                LEFT JOIN PrevSnapshot ON 1=1
                LEFT JOIN SnapshotSchoolScore prev
                    ON prev.SnapshotID = PrevSnapshot.SnapshotID
                    AND prev.ChaSchool = cur.ChaSchool
                WHERE cur.SnapshotID = @P1
                ORDER BY cur.Rank",
                &[&snapshot_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(SnapshotSchoolScore::from_row).collect())
    }

    /// Get LevelCapRace scores for a snapshot, with rank_change compared to the previous snapshot
    pub async fn get_snapshot_level_cap_race_scores(
        &self,
        snapshot_id: Uuid,
    ) -> Result<Vec<SnapshotLevelCapRaceScore>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "WITH PrevSnapshot AS (
                    SELECT TOP 1 ps.SnapshotID
                    FROM EventSnapshot ps
                    INNER JOIN EventSnapshot cs ON cs.SnapshotID = @P1
                        AND ps.EventID = cs.EventID
                        AND ps.SnapshotTime < cs.SnapshotTime
                    ORDER BY ps.SnapshotTime DESC
                )
                SELECT
                    cur.*,
                    CASE
                        WHEN prev.Rank IS NULL THEN NULL
                        ELSE prev.Rank - cur.Rank
                    END AS RankChange
                FROM SnapshotLevelCapRaceScore cur
                LEFT JOIN PrevSnapshot ON 1=1
                LEFT JOIN SnapshotLevelCapRaceScore prev
                    ON prev.SnapshotID = PrevSnapshot.SnapshotID
                    AND prev.ChaNum = cur.ChaNum
                WHERE cur.SnapshotID = @P1
                ORDER BY cur.Rank",
                &[&snapshot_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows
            .iter()
            .map(SnapshotLevelCapRaceScore::from_row)
            .collect())
    }
}
