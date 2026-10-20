use anyhow::Result;
use ran_models::{admin::{characters::CharacterInfo}, BaseDataCountResult};

use crate::{gamemanager::GameDB};

#[async_trait::async_trait]
pub trait GameCharacterTrait {
    async fn get_characters_by_user(
        &self,
        page: i16,
        page_size: i16,
        user_num: i32,
    ) -> Result<BaseDataCountResult>;

    async fn get_characters(
        &self,
        page: i16,
        page_size: i16,
        class: Option<Vec<i32>>,
        school: Option<Vec<i32>>,
        status: Option<i16>,
        search: Option<String>,
        sort_by: &str,
        sort_order: &str,
        cha_online: Option<Vec<i16>>,
    ) -> Result<BaseDataCountResult>;

    async fn get_character_by_id(
        &self,
        cha_num: i32
    ) -> Result<CharacterInfo>;

    async fn get_guild_leader_school(
        &self,
        guild_num: i32,
    ) -> Result<Option<(i32, String)>>;

    async fn get_character_item_logs(
        &self,
        page: i16,
        page_size: i16,
        cha_num: i32
    ) -> Result<BaseDataCountResult>;
}

#[async_trait::async_trait]
impl GameCharacterTrait for GameDB {
    async fn get_characters_by_user(
        &self,
        page: i16,
        page_size: i16,
        user_num: i32,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await?;

        let sp_query = "EXEC sp_GetCharactersByUser @UserNum = @P1, @Page = @P2, @PageSize = @P3";

        let results = client
            .query(sp_query, &[
                &user_num,
                &page,
                &page_size
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
            data: result_sets
                .next()
                .unwrap_or_default(), 
            total_records
        })
    }

    async fn get_characters(
        &self,
        page: i16,
        page_size: i16,
        class: Option<Vec<i32>>,
        school: Option<Vec<i32>>,
        status: Option<i16>,
        search: Option<String>,
        sort_by: &str,
        sort_order: &str,
        cha_online: Option<Vec<i16>>,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await?;

        let sp_query = "EXEC sp_GetCharacters
            @SearchNum = @P1,
            @Search = @P2,
            @Class = @P3,
            @School = @P4,
            @Status = @P5,
            @ChaOnline = @P6,
            @SortBy = @P7,
            @SortOrder = @P8,
            @Page = @P9,
            @PageSize = @P10
        ";

        let parsed_search: Option<i32> = search.as_ref().and_then(|s| s.parse::<i32>().ok());

        let class_list = class
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(|v|
                v.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let school_list = school
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(|v|
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let cha_online_list = cha_online
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(|v|
                v.iter()
                .map(|o| o.to_string())
                .collect::<Vec<_>>().join(",")
            );

        let results = client
            .query(sp_query, &[
                &parsed_search,
                &search,
                &class_list,
                &school_list,
                &status,
                &cha_online_list,
                &sort_by,
                &sort_order,
                &page,
                &page_size
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
            data: result_sets
                .next()
                .unwrap_or_default(),
            total_records
        })
    }

    async fn get_character_by_id(
        &self,
        cha_num: i32
    ) -> Result<CharacterInfo> {
        let mut client = self
            .pool
            .get()
            .await?;

        let sp_query = "EXEC sp_GetCharacterById @ChaNum = @P1";

        let results = client
            .query(sp_query, &[&cha_num])
            .await?
            .into_row()
            .await?;

        match results {
            Some(row) => {
                Ok(CharacterInfo::from_row(&row))
            }
            None => Err(anyhow::anyhow!("Character {} not found", cha_num))
        }
    }

    async fn get_guild_leader_school(
        &self,
        guild_num: i32,
    ) -> Result<Option<(i32, String)>> {
        let mut client = self.pool.get().await?;

        let row = client
            .query(
                "SELECT CI.ChaSchool, CI.ChaName
                 FROM RanGameS1.dbo.GuildInfo GI WITH (NOLOCK)
                 INNER JOIN RanGameS1.dbo.ChaInfo CI WITH (NOLOCK) ON CI.ChaNum = GI.ChaNum
                 WHERE GI.GuNum = @P1",
                &[&guild_num],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| {
            let school = r.get::<i32, _>("ChaSchool").unwrap_or_default();
            let name = r
                .get::<&str, _>("ChaName")
                .map_or_else(String::new, |s| s.to_string());
            (school, name)
        }))
    }

    async fn get_character_item_logs(
        &self,
        page: i16,
        page_size: i16,
        cha_num: i32,
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await?;

        let sp_query = "EXEC sp_GetCharacterItemLogs @ChaNum = @P1";

        let results = client
            .query(sp_query, &[
                &cha_num,
                &page,
                &page_size
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
            data: result_sets
                .next()
                .unwrap_or_default(), 
            total_records
        })
    }
}