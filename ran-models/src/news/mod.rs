use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, utoipa::ToSchema)]
pub struct NewsData {
    pub news_id: i32,
    pub news_title: String,
    pub news_slug: String,
    pub news_content: String,
    pub news_author: String,
    pub news_creation_date: DateTime<Local>,
    pub news_status: i16,
    pub news_category: i16,
}

impl From<NewsData> for NewsItem {
    fn from(data: NewsData) -> Self {
        NewsItem {
            id: data.news_id,
            title: data.news_title,
            slug: data.news_slug,
            content: data.news_content,
            author: data.news_author,
            created_at: data.news_creation_date,
            status: data.news_status,
            category: data.news_category,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct NewsItem {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Local>,
    pub status: i16,
    pub category: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateNewsRequest {
    pub category: i16,
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub status: i16,
}
