use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::Row;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CharacterInfo {
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub cha_sex: i16,
    pub cha_level: i32,
    pub cha_money: f64,
    pub cha_online: i32,
    pub cha_created_date: DateTime<Local>,

    pub cha_deleted: i32,
    pub cha_deleted_date: DateTime<Local>,

    pub cha_guild_num: i32,
    pub cha_guild_name: String,

    pub cha_contribution_point: i32,

    pub user_num: i32,
    pub user_name: String,

    pub cha_vip: i32,
    pub cha_vip_expiry_date: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CharacterInfoList {
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub cha_sex: i16,
    pub cha_level: i32,
    pub cha_money: f64,
    pub cha_online: i32,

    pub cha_deleted: i32,
    pub cha_deleted_date: DateTime<Local>,

    pub cha_contribution_point: i32,

    pub user_num: i32,
    pub user_name: String,
}

impl CharacterInfo {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            cha_class: row.get::<i32, _>("ChaClass").unwrap_or(1),
            cha_sex: row.get::<u8, _>("ChaSex").unwrap() as i16,
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or(1),
            cha_level: row.get::<i32, _>("ChaLevel").unwrap_or(1),

            cha_money: row.get::<f64, _>("ChaMoney").unwrap_or(0.0),

            cha_online: row.get::<i32, _>("ChaOnline").unwrap_or(0),

            cha_deleted: row.get::<i32, _>("ChaDeleted").unwrap_or(0),
            cha_deleted_date: row
                .get::<NaiveDateTime, _>("ChaDeletedDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),

            cha_contribution_point: row.get::<i32, _>("ContributionPoint").unwrap_or(0),

            cha_created_date: row
                .get::<NaiveDateTime, _>("ChaCreateDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),

            cha_guild_num: row.get::<i32, _>("GuNum").unwrap_or_default(),
            cha_guild_name: row.get::<&str, _>("GuName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            cha_vip: row.get::<i32, _>("ChaVIP").unwrap_or_default(),
            cha_vip_expiry_date: row
                .get::<NaiveDateTime, _>("ChaVIPExpiryTime")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),

            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            user_name: row
                .get::<&str, _>("UserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
        }
    }
}

impl CharacterInfoList {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            cha_class: row.get::<i32, _>("ChaClass").unwrap_or(1),
            cha_sex: row.get::<u8, _>("ChaSex").unwrap() as i16,
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or(1),
            cha_level: row.get::<i32, _>("ChaLevel").unwrap_or(1),

            cha_money: row.get::<f64, _>("ChaMoney").unwrap_or(0.0),

            cha_online: row.get::<i32, _>("ChaOnline").unwrap_or(0),

            cha_deleted: row.get::<i32, _>("ChaDeleted").unwrap_or(0),
            cha_deleted_date: row
                .get::<NaiveDateTime, _>("ChaDeletedDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),

            cha_contribution_point: row.get::<i32, _>("ContributionPoint").unwrap_or(0),

            user_num: row.get::<i32, _>("UserNum").unwrap_or_default(),
            user_name: row
                .get::<&str, _>("UserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
        }
    }
}
