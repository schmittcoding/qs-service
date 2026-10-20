mod util;

pub mod download;
mod marketplace;
pub mod news;
pub mod item_shop;
pub mod shop;
use serde::{Deserialize, Serialize};
use tiberius::Row;

pub mod errors;
pub mod payments;
pub mod users;

pub mod reports;

pub mod admin;

pub mod email;
pub mod events;

#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<i16>,
    pub category: Option<i16>,
    pub search: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct ActiveCheck {
    pub status: Option<i16>,
}

#[derive(Serialize, Deserialize)]
pub struct BaseDataCountResult<T = Row> {
    pub data: Vec<T>,
    pub total_records: i32
}
