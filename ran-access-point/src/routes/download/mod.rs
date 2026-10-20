pub mod download_test;
use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::webmanager::download::WebDownloadTrait;
use ran_models::{download::DownloadItem, PaginationParams};
use serde_json::json;

use crate::{constants::http_messages::http_messages::error, database::DBState, states::AppState};

#[get("")]
pub async fn get_download_links(
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    let r#type = params.category;

    let cache_instance = states.download_links_cache.lock().await;
    let data_guard = cache_instance.data.lock().await;

    let links = if let Some(data) = data_guard.clone() {
        log::info!("Cache is fresh, returning existing data.");
        data
            .into_iter()
            .filter(|link| {
                let category_ok = r#type.map(|t| link.r#type == t).unwrap_or(true);
                
                let status_ok = link.status == 1;

                category_ok && status_ok
            })
            .collect()
    } else {
        log::warn!("⚠️ Download links cache is stale or empty, initiating new data fetch.");

        let fetched_links = match db
            .web_db
            .get_download_links(Some(1), r#type.clone())
            .await 
        {
            Ok(data) => data,
            Err(err) => {
                log::error!("❌ Failed to download links from database: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(json!({ "error": error::INTERNAL_SERVER_ERROR }));
            }
        };

        drop(data_guard);
        *cache_instance.data.lock().await = Some(fetched_links.clone());
        fetched_links
    };

    let total_items = links.len();
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

    let paginated: Vec<DownloadItem> = links
        .into_iter()
        .skip(start)
        .take(limit)
        .collect();

    HttpResponse::Ok().json(json!({
        "data": paginated,
        "page": page,
        "limit": limit,
        "total_items": total_items,
        "total_pages": total_pages
    }))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_download_links);
}
