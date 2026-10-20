use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod auth;
pub mod characters;
pub mod items;
pub mod players;

pub mod users;
pub mod game;

pub mod purchases;
pub mod item_bank;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSortBy {
    ContributionPoint,
    Level,
    Money,
}

impl CharacterSortBy {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            CharacterSortBy::ContributionPoint => "contribution_point",
            CharacterSortBy::Level => "level",
            CharacterSortBy::Money => "money",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

#[derive(Deserialize)]
pub struct AdminPaginationParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub user_num: Option<i16>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct BasicPaginationParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}


fn validate_cha_online_vec(v: &Vec<i16>) -> Result<(), validator::ValidationError> {
    if !v.iter().all(|&val| val == 0 || val == 1) {
        let mut err = validator::ValidationError::new("cha_online");
        err.message = Some("each cha_online value must be 0 or 1".into());
        return Err(err);
    }
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct CharacterPaginationParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<i16>,
    #[serde(default)]
    pub class: Option<Vec<i32>>,
    #[serde(default)]
    pub school: Option<Vec<i32>>,
    pub search: Option<String>,
    pub sort_by: Option<CharacterSortBy>,
    pub sort_order: Option<SortOrder>,
    #[validate(custom(function = "validate_cha_online_vec"))]
    #[serde(default)]
    pub cha_online: Option<Vec<i16>>,
}

#[derive(Deserialize)]
pub struct AdminPlayersFilterParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    #[serde(default)]
    pub category: Option<Vec<i16>>,
    #[serde(default)]
    pub status: Option<Vec<i16>>,
}


#[derive(Deserialize)]
pub struct LogPaginationParams {
    pub from: Option<i32>,
    pub to: Option<i32>,
    pub id: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct AdminItemShopFilterParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub shop_type: Option<i16>,
    pub search: Option<String>,
    #[serde(default)]
    pub category: Option<Vec<i32>>,
    #[serde(default)]
    pub status: Option<Vec<i32>>,
    #[serde(default)]
    pub purchase_limit: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct TransactionFilterParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    #[serde(default)]
    pub method: Option<Vec<i16>>,
    #[serde(default)]
    pub status: Option<Vec<i16>>,
}
