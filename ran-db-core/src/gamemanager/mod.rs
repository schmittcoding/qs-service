use crate::{metrics, DBManager, DatabaseTrait};
use anyhow::{Context, Result};
use deadpool_tiberius::Pool;
use futures_util::StreamExt;
use ran_models::errors::DatabaseError;
use ran_models::events::ref_data::map_game_class_to_event_class;
use tracing::{error, info};

pub mod admin;

#[derive(Clone)]
pub struct GameDB {
    pub pool: Pool,
}

#[async_trait::async_trait]
impl DatabaseTrait for GameDB {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError> {
        metrics::record_connection_attempt("game");
        let pool = match db_manager.get_database("game") {
            Some(pool) => {
                metrics::record_connection_success("game");
                pool.clone()
            }
            None => {
                metrics::record_connection_failure("game");
                return Err(DatabaseError::DatabaseNotConfigured);
            }
        };

        Ok(Self { pool })
    }

    async fn test_db_connection(&self) -> Result<(), DatabaseError> {
        let mut client = self.pool.get().await?;
        let stream = client.query("SELECT 1", &[]).await?;
        let row = stream
            .into_row()
            .await?
            .ok_or(DatabaseError::TestConnectionFailed)?;

        match row.get::<i32, _>(0) {
            Some(_) => {
                info!("✅ Web DB connection verified");
                Ok(())
            }
            _ => {
                error!("Unexpected test value");
                Err(DatabaseError::TestConnectionFailed)
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CommonRankingData {
    pub rank: i32,
    pub cha_num: i32,
    pub user_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub cha_sex: i16,
    pub cha_level: i32,
    pub cha_money: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct LevelRankingData {
    pub rank: i32,
    pub cha_num: i32,
    pub user_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub cha_level: i32,
    pub cha_exp: i64,
}

impl GameDB {
    pub async fn get_characters_by_user_num(
        &self,
        user_num: i32,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut client = self.pool.get().await?;
        let stream = client
            .query("EXEC GetCharacters @UserNum = @P1", &[&user_num])
            .await?;
        let row = stream.into_row().await?;

        Ok(row.map(|r| r.get::<&str, _>(0).unwrap().to_string()))
    }

    pub async fn get_common_ranking_data(&self) -> Result<Vec<CommonRankingData>> {
        let mut client = self.pool.get().await?;

        let mut stream = client
            .query("EXEC sp_GetCommonRankingData", &[])
            .await?;

        let mut ranking_data_vec = Vec::new();

        while let Some(result) = stream.next().await {
            let row = result.context("Error processing row")?;
            if let Some(row_data) = row.as_row() {
                let cha_num = row_data.get::<i32, _>("ChaNum").unwrap();
                let user_num = row_data.get::<i32, _>("UserNum").unwrap();
                let cha_name = row_data.get::<&str, _>("ChaName").unwrap();
                let cha_class = row_data.get::<i32, _>("ChaClass").unwrap();
                let cha_school = row_data.get::<i32, _>("ChaSchool").unwrap();
                let cha_sex = row_data.get::<u8, _>("ChaSex").unwrap() as i16;
                let cha_level = row_data.get::<i32, _>("ChaLevel").unwrap();
                let cha_money = row_data.get::<f64, _>("ChaMoney").unwrap();

                if let Some(event_class) = map_game_class_to_event_class(cha_class) {
                    ranking_data_vec.push(CommonRankingData {
                        rank: 0,
                        cha_num,
                        user_num,
                        cha_name: cha_name.to_string(),
                        cha_class: event_class,
                        cha_sex,
                        cha_school,
                        cha_level,
                        cha_money,
                    });
                }
            }
        }

        ranking_data_vec.sort_by(|a, b| {
            b.cha_money
                .partial_cmp(&a.cha_money)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, data) in ranking_data_vec.iter_mut().enumerate() {
            data.rank = (i + 1) as i32;
        }
        Ok(ranking_data_vec)
    }


    pub async fn get_level_ranking_data(&self) -> Result<Vec<LevelRankingData>> {
        let mut client = self.pool.get().await?;

        let mut stream = client
            .query("EXEC sp_GetLevelRankingData", &[])
            .await?;

        let mut ranking_data_vec = Vec::new();

        while let Some(result) = stream.next().await {
            let row = result.context("Error processing row")?;
            if let Some(row_data) = row.as_row() {
                let cha_num = row_data.get::<i32, _>("ChaNum").unwrap();
                let user_num = row_data.get::<i32, _>("UserNum").unwrap();
                let cha_name = row_data.get::<&str, _>("ChaName").unwrap();
                let cha_class = row_data.get::<i32, _>("ChaClass").unwrap();
                let cha_school = row_data.get::<i32, _>("ChaSchool").unwrap();
                let cha_level = row_data.get::<i32, _>("ChaLevel").unwrap();
                let cha_exp = row_data.get::<f64, _>("ChaExp").unwrap_or(0.0) as i64;

                if let Some(event_class) = map_game_class_to_event_class(cha_class) {
                    ranking_data_vec.push(LevelRankingData {
                        rank: 0,
                        cha_num,
                        user_num,
                        cha_name: cha_name.to_string(),
                        cha_class: event_class,
                        cha_school,
                        cha_level,
                        cha_exp,
                    });
                }
            }
        }

        ranking_data_vec.sort_by(|a, b| {
            b.cha_level
                .cmp(&a.cha_level)
                .then_with(|| b.cha_exp.cmp(&a.cha_exp))
        });
        for (i, data) in ranking_data_vec.iter_mut().enumerate() {
            data.rank = (i + 1) as i32;
        }
        Ok(ranking_data_vec)
    }
}
