use anyhow::Result;
use chrono::NaiveDateTime;
use ran_models::BaseDataCountResult;

use crate::gamemanager::GameDB;

#[async_trait::async_trait]
pub trait GameDispatchFilterTrait {
    async fn filter_characters_for_dispatch(
        &self,
        search_num: Option<i32>,
        search: Option<String>,
        min_level: Option<i32>,
        max_level: Option<i32>,
        login_from: Option<NaiveDateTime>,
        login_to: Option<NaiveDateTime>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult>;
}

#[async_trait::async_trait]
impl GameDispatchFilterTrait for GameDB {
    async fn filter_characters_for_dispatch(
        &self,
        search_num: Option<i32>,
        search: Option<String>,
        min_level: Option<i32>,
        max_level: Option<i32>,
        login_from: Option<NaiveDateTime>,
        login_to: Option<NaiveDateTime>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult> {
        let mut client = self.pool.get().await?;

        let sp_query = "EXEC sp_FilterCharactersForDispatch
            @SearchNum = @P1,
            @Search = @P2,
            @MinLevel = @P3,
            @MaxLevel = @P4,
            @LoginFrom = @P5,
            @LoginTo = @P6,
            @Page = @P7,
            @PageSize = @P8";

        let results = client
            .query(sp_query, &[
                &search_num,
                &search,
                &min_level,
                &max_level,
                &login_from,
                &login_to,
                &page,
                &page_size,
            ])
            .await?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();

        let total_records = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        Ok(BaseDataCountResult {
            data: result_sets.next().unwrap_or_default(),
            total_records,
        })
    }
}
