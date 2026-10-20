use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::webmanager::news::WebNewsTrait;
use ran_models::{news::NewsItem, PaginationParams};
use serde_json::json;
use tracing::error;

use crate::{database::DBState, states::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/news/get-news",
    tag = "News",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated news list", body = serde_json::Value),
        (status = 500, description = "Failed to fetch news")
    )
)]
#[get("/get-news")]
pub async fn get_news(
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    query: web::Query<PaginationParams>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);

    let cached_news = {
        let cache = states.news_data.lock().await;
        cache.get(&0).cloned()
    };

    let all_news: Vec<NewsItem> = if let Some(news) = cached_news {
        news
    } else {
        log::warn!("⚠️ News cache is empty. Fetching from database...");
        let fetched_news: Vec<ran_models::news::NewsData> = match db.web_db.get_news_data().await {
            Ok(news) => news,
            Err(err) => {
                error!("❌ Failed to fetch news: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to fetch news" }));
            }
        };

        // Convert fetched_news into Vec<NewsItem>
        let news_items: Vec<NewsItem> = fetched_news
            .into_iter()
            .map(|item| NewsItem {
                id: item.news_id,
                title: item.news_title,
                slug: item.news_slug,
                content: item.news_content,
                author: item.news_author,
                created_at: item.news_creation_date,
                status: item.news_status,
                category: item.news_category,
            })
            .collect();

        let mut cache = states.news_data.lock().await;
        cache.insert(0, news_items.clone());
        news_items
    };

    let filtered_news: Vec<NewsItem> = all_news
        .into_iter()
        .filter(|item| {
            let status_ok = query.status.map(|s| item.status == s).unwrap_or(true);
            let category_ok = query.category.map(|c| item.category == c).unwrap_or(true);
            status_ok && category_ok
        })
        .collect();

    let total_items = filtered_news.len();
    let total_pages = if total_items == 0 {
        1
    } else {
        (total_items as f64 / limit as f64).ceil() as usize
    };

    // Adjust page if the requested page is out-of-range.
    let current_page = if page > total_pages {
        total_pages
    } else {
        page
    };
    let start = (current_page - 1) * limit;

    // Paginate the filtered news.
    let paginated_news: Vec<NewsItem> = filtered_news.into_iter().skip(start).take(limit).collect();

    HttpResponse::Ok().json(json!({
        "page": current_page,
        "limit": limit,
        "total_pages": total_pages,
        "total_items": total_items,
        "data": paginated_news
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_news);
}
