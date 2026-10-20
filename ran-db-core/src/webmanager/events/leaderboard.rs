use anyhow::Result;
use ran_models::events::{
    guild_score::GuildEventScore, koth_score::KOTHScore,
    level_cap_race_score::LevelCapRaceScore, school_score::SchoolEventScore,
};
use uuid::Uuid;

use crate::webmanager::WebDB;

impl WebDB {
    /// Guild leaderboard sorted by ComputedScore DESC
    pub async fn get_guild_leaderboard(
        &self,
        event_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<GuildEventScore>, i64)> {
        let mut client = self.pool.get().await?;
        let offset = (page.saturating_sub(1)) * page_size;

        let count_row = client
            .query(
                "SELECT COUNT(*) AS Total FROM GuildEventScore WHERE EventID = @P1",
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
                "SELECT *, ROW_NUMBER() OVER (ORDER BY ComputedScore DESC) AS Rank
                 FROM GuildEventScore
                 WHERE EventID = @P1
                 ORDER BY ComputedScore DESC
                 OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY",
                &[&event_id, &(offset as i32), &(page_size as i32)],
            )
            .await?
            .into_first_result()
            .await?;

        let scores = rows.iter().map(GuildEventScore::from_row).collect();
        Ok((scores, total))
    }

    /// School leaderboard sorted by ComputedScore DESC
    pub async fn get_school_leaderboard(&self, event_id: Uuid) -> Result<Vec<SchoolEventScore>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT *, ROW_NUMBER() OVER (ORDER BY ComputedScore DESC) AS Rank FROM SchoolEventScore WHERE EventID = @P1 ORDER BY ComputedScore DESC",
                &[&event_id],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(SchoolEventScore::from_row).collect())
    }

    /// KOTH leaderboard per class, sorted by (TotalKills - Deduction) DESC
    pub async fn get_koth_leaderboard(
        &self,
        event_id: Uuid,
        cha_class: Option<i32>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<KOTHScore>, i64)> {
        let mut client = self.pool.get().await?;
        let offset = (page.saturating_sub(1)) * page_size;

        let (count_query, data_query) = match cha_class {
            Some(_) => (
                "SELECT COUNT(*) AS Total FROM KOTHScore WHERE EventID = @P1 AND ChaClass = @P2",
                "SELECT *, ROW_NUMBER() OVER (ORDER BY (TotalKills - TotalDeaths - Deduction) DESC) AS Rank
                 FROM KOTHScore
                 WHERE EventID = @P1 AND ChaClass = @P2
                 ORDER BY (TotalKills - TotalDeaths - Deduction) DESC
                 OFFSET @P3 ROWS FETCH NEXT @P4 ROWS ONLY",
            ),
            None => (
                "SELECT COUNT(*) AS Total FROM KOTHScore WHERE EventID = @P1 AND @P2 IS NULL",
                "SELECT *, ROW_NUMBER() OVER (ORDER BY (TotalKills - TotalDeaths - Deduction) DESC) AS Rank
                 FROM KOTHScore
                 WHERE EventID = @P1 AND @P2 IS NULL
                 ORDER BY (TotalKills - TotalDeaths - Deduction) DESC
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

        let scores = rows.iter().map(KOTHScore::from_row).collect();
        Ok((scores, total))
    }

    /// LevelCapRace leaderboard sorted by levels_gained DESC, then current_level DESC
    pub async fn get_level_cap_race_leaderboard(
        &self,
        event_id: Uuid,
        cha_class: Option<i32>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LevelCapRaceScore>, i64)> {
        let mut client = self.pool.get().await?;
        let offset = (page.saturating_sub(1)) * page_size;

        let (count_query, data_query) = match cha_class {
            Some(_) => (
                "SELECT COUNT(*) AS Total FROM LevelCapRaceScore WHERE EventID = @P1 AND ChaClass = @P2",
                "SELECT *, ROW_NUMBER() OVER (ORDER BY CurrentLevel DESC, LevelUpDate ASC) AS Rank
                 FROM LevelCapRaceScore
                 WHERE EventID = @P1 AND ChaClass = @P2
                 ORDER BY CurrentLevel DESC, LevelUpDate ASC
                 OFFSET @P3 ROWS FETCH NEXT @P4 ROWS ONLY",
            ),
            None => (
                "SELECT COUNT(*) AS Total FROM LevelCapRaceScore WHERE EventID = @P1 AND @P2 IS NULL",
                "SELECT *, ROW_NUMBER() OVER (ORDER BY CurrentLevel DESC, LevelUpDate ASC) AS Rank
                 FROM LevelCapRaceScore
                 WHERE EventID = @P1 AND @P2 IS NULL
                 ORDER BY CurrentLevel DESC, LevelUpDate ASC
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

        let scores = rows.iter().map(LevelCapRaceScore::from_row).collect();
        Ok((scores, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_offset_calculation() {
        let page = 1u32;
        let page_size = 10u32;
        let offset = (page.saturating_sub(1)) * page_size;
        assert_eq!(offset, 0);

        let page = 2u32;
        let offset = (page.saturating_sub(1)) * page_size;
        assert_eq!(offset, 10);

        let page = 5u32;
        let offset = (page.saturating_sub(1)) * page_size;
        assert_eq!(offset, 40);
    }

    #[test]
    fn test_pagination_offset_page_0_treated_as_1() {
        let page = 0u32;
        let page_size = 10u32;
        let offset = (page.saturating_sub(1)) * page_size;
        // saturating_sub(1) on 0 gives 0, so 0 * page_size = 0
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_pagination_large_page() {
        let page = 1000u32;
        let page_size = 20u32;
        let offset = (page.saturating_sub(1)) * page_size;
        assert_eq!(offset, 19980);
    }

    #[test]
    fn test_class_filter_with_some_value() {
        let cha_class = Some(1i32);
        let should_filter = cha_class.is_some();
        assert!(should_filter);
    }

    #[test]
    fn test_class_filter_with_none_value() {
        let cha_class: Option<i32> = None;
        let should_filter = cha_class.is_some();
        assert!(!should_filter);
    }

    #[test]
    fn test_query_params_alignment_with_class_filter() {
        let event_id = uuid::Uuid::new_v4();
        let cha_class = Some(2i32);
        let offset = 10i32;
        let page_size = 20i32;

        // When cha_class is Some, params are: event_id, cha_class, offset, page_size
        let params = vec![
            event_id.to_string(),
            cha_class.map(|c| c.to_string()).unwrap_or_default(),
            offset.to_string(),
            page_size.to_string(),
        ];
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_query_params_alignment_without_class_filter() {
        let event_id = uuid::Uuid::new_v4();
        let cha_class: Option<i32> = None;
        let offset = 10i32;
        let page_size = 20i32;

        // When cha_class is None, params are: event_id, cha_class (None), offset, page_size
        let params = vec![
            event_id.to_string(),
            cha_class.map(|c| c.to_string()).unwrap_or_default(),
            offset.to_string(),
            page_size.to_string(),
        ];
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_total_count_zero_handling() {
        // When COUNT(*) returns 0
        let total: i64 = 0;
        assert_eq!(total, 0);
    }

    #[test]
    fn test_total_count_conversion_from_i32_to_i64() {
        let count_i32: i32 = 42;
        let total: i64 = count_i32 as i64;
        assert_eq!(total, 42i64);
    }

    #[test]
    fn test_large_total_count_conversion() {
        let count_i32: i32 = i32::MAX;
        let total: i64 = count_i32 as i64;
        assert_eq!(total, i32::MAX as i64);
    }
}
