use crate::{database::DBState, states::AppState};
use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::gamemanager::{CommonRankingData, LevelRankingData};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RankingQueryParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    pub cha_class: Option<i32>,
    pub cha_school: Option<i32>,
}

#[derive(Serialize)]
pub struct GoldRankingResponse {
    pub rank_number: i32,
    pub cha_name: String,
    pub cha_money: f64,
    pub cha_level: i32,
    pub cha_class: i32,
    pub cha_school: i32,
}

fn calculate_start(page: usize, limit: usize, total: usize) -> usize {
    let total_pages = if total == 0 { 1 } else { (total + limit - 1) / limit };
    let clamped_page = page.min(total_pages).max(1);
    (clamped_page - 1) * limit
}

#[derive(Serialize)]
pub struct LevelRankingResponse {
    pub rank_number: i32,
    pub cha_name: String,
    pub cha_exp: i64,
    pub cha_level: i32,
    pub cha_class: i32,
    pub cha_school: i32,
}

impl From<&LevelRankingData> for LevelRankingResponse {
    fn from(data: &LevelRankingData) -> Self {
        LevelRankingResponse {
            rank_number: data.rank,
            cha_name: data.cha_name.clone(),
            cha_exp: data.cha_exp,
            cha_level: data.cha_level,
            cha_class: data.cha_class,
            cha_school: data.cha_school,
        }
    }
}

impl From<&CommonRankingData> for GoldRankingResponse {
    fn from(data: &CommonRankingData) -> Self {
        GoldRankingResponse {
            rank_number: data.rank,
            cha_name: data.cha_name.clone(),
            cha_money: data.cha_money,
            cha_level: data.cha_level,
            cha_class: data.cha_class,
            cha_school: data.cha_school,
        }
    }
}

#[get("/gold")]
async fn get_gold_ranking(
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    query: web::Query<RankingQueryParams>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);

    let cache = states.gold_ranking_cache.lock().await;

    if cache.is_stale().await {
        let mut is_fetching = cache.fetching_in_progress.lock().await;

        if !*is_fetching {
            *is_fetching = true;
            drop(is_fetching);

            log::info!("Gold ranking cache is stale. Fetching from database...");
            match db.game_db.get_common_ranking_data().await {
                Ok(data) => {
                    cache.update(data).await;
                }
                Err(err) => {
                    *cache.fetching_in_progress.lock().await = false;
                    let data_guard = cache.data.lock().await;
                    if data_guard.is_none() {
                        log::error!("Failed to fetch gold ranking and no cached data available: {:?}", err);
                        return HttpResponse::InternalServerError()
                            .json(json!({ "error": "Failed to fetch ranking data" }));
                    }
                    log::warn!("Failed to refresh gold ranking, serving stale cache: {:?}", err);
                }
            }

            *cache.fetching_in_progress.lock().await = false;
        }
    }

    let data_guard = cache.data.lock().await;
    let indexed = match data_guard.as_ref() {
        Some(data) => data,
        None => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Ranking data not available" }));
        }
    };

    let search_lower = query.search.as_ref().map(|s| s.to_lowercase());
    let has_filter = query.cha_class.is_some() || query.cha_school.is_some();

    // Use pre-built index buckets instead of scanning all 98k records
    let (total_items, paginated) = if has_filter {
        let bucket = indexed.get_bucket(query.cha_class, query.cha_school);
        let indices = match bucket {
            Some(indices) => indices,
            None => {
                // Filter combo has no matching records
                return HttpResponse::Ok().json(json!({
                    "page": 1, "limit": limit, "total_pages": 1, "total_items": 0, "data": []
                }));
            }
        };

        if let Some(ref search) = search_lower {
            // Filter + search: must scan the bucket
            let matching: Vec<&CommonRankingData> = indices
                .iter()
                .map(|&i| &indexed.all[i])
                .filter(|item| item.cha_name.to_lowercase().contains(search.as_str()))
                .collect();

            let total = matching.len();
            let start = calculate_start(page, limit, total);
            let page_data: Vec<GoldRankingResponse> = matching
                .into_iter()
                .skip(start)
                .take(limit)
                .map(GoldRankingResponse::from)
                .collect();
            (total, page_data)
        } else {
            // Filter only, no search: direct slice pagination on the bucket
            let total = indices.len();
            let start = calculate_start(page, limit, total);
            let page_data: Vec<GoldRankingResponse> = indices
                .iter()
                .skip(start)
                .take(limit)
                .map(|&i| GoldRankingResponse::from(&indexed.all[i]))
                .collect();
            (total, page_data)
        }
    } else if let Some(ref search) = search_lower {
        // Search only, no class/school filter: must scan all
        let matching: Vec<&CommonRankingData> = indexed
            .all
            .iter()
            .filter(|item| item.cha_name.to_lowercase().contains(search.as_str()))
            .collect();

        let total = matching.len();
        let start = calculate_start(page, limit, total);
        let page_data: Vec<GoldRankingResponse> = matching
            .into_iter()
            .skip(start)
            .take(limit)
            .map(GoldRankingResponse::from)
            .collect();
        (total, page_data)
    } else {
        // No filters, no search: direct slice on all data
        let total = indexed.all.len();
        let start = calculate_start(page, limit, total);
        let page_data: Vec<GoldRankingResponse> = indexed
            .all
            .iter()
            .skip(start)
            .take(limit)
            .map(GoldRankingResponse::from)
            .collect();
        (total, page_data)
    };

    let total_pages = if total_items == 0 {
        1
    } else {
        (total_items as f64 / limit as f64).ceil() as usize
    };

    let current_page = if page > total_pages {
        total_pages
    } else {
        page
    };

    HttpResponse::Ok().json(json!({
        "page": current_page,
        "limit": limit,
        "total_pages": total_pages,
        "total_items": total_items,
        "data": paginated
    }))
}
#[get("/level")]
async fn get_level_ranking(
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    query: web::Query<RankingQueryParams>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);

    let cache = states.level_ranking_cache.lock().await;

    if cache.is_stale().await {
        let mut is_fetching = cache.fetching_in_progress.lock().await;

        if !*is_fetching {
            *is_fetching = true;
            drop(is_fetching);

            log::info!("Level ranking cache is stale. Fetching from database...");
            match db.game_db.get_level_ranking_data().await {
                Ok(data) => {
                    cache.update(data).await;
                }
                Err(err) => {
                    *cache.fetching_in_progress.lock().await = false;
                    let data_guard = cache.data.lock().await;
                    if data_guard.is_none() {
                        log::error!("Failed to fetch level ranking and no cached data available: {:?}", err);
                        return HttpResponse::InternalServerError()
                            .json(json!({ "error": "Failed to fetch ranking data" }));
                    }
                    log::warn!("Failed to refresh level ranking, serving stale cache: {:?}", err);
                }
            }

            *cache.fetching_in_progress.lock().await = false;
        }
    }

    let data_guard = cache.data.lock().await;
    let indexed = match data_guard.as_ref() {
        Some(data) => data,
        None => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Ranking data not available" }));
        }
    };

    let search_lower = query.search.as_ref().map(|s| s.to_lowercase());
    let has_filter = query.cha_class.is_some() || query.cha_school.is_some();

    let (total_items, paginated) = if has_filter {
        let bucket = indexed.get_bucket(query.cha_class, query.cha_school);
        let indices = match bucket {
            Some(indices) => indices,
            None => {
                return HttpResponse::Ok().json(json!({
                    "page": 1, "limit": limit, "total_pages": 1, "total_items": 0, "data": []
                }));
            }
        };

        if let Some(ref search) = search_lower {
            let matching: Vec<&LevelRankingData> = indices
                .iter()
                .map(|&i| &indexed.all[i])
                .filter(|item| item.cha_name.to_lowercase().contains(search.as_str()))
                .collect();

            let total = matching.len();
            let start = calculate_start(page, limit, total);
            let page_data: Vec<LevelRankingResponse> = matching
                .into_iter()
                .skip(start)
                .take(limit)
                .map(LevelRankingResponse::from)
                .collect();
            (total, page_data)
        } else {
            let total = indices.len();
            let start = calculate_start(page, limit, total);
            let page_data: Vec<LevelRankingResponse> = indices
                .iter()
                .skip(start)
                .take(limit)
                .map(|&i| LevelRankingResponse::from(&indexed.all[i]))
                .collect();
            (total, page_data)
        }
    } else if let Some(ref search) = search_lower {
        let matching: Vec<&LevelRankingData> = indexed
            .all
            .iter()
            .filter(|item| item.cha_name.to_lowercase().contains(search.as_str()))
            .collect();

        let total = matching.len();
        let start = calculate_start(page, limit, total);
        let page_data: Vec<LevelRankingResponse> = matching
            .into_iter()
            .skip(start)
            .take(limit)
            .map(LevelRankingResponse::from)
            .collect();
        (total, page_data)
    } else {
        let total = indexed.all.len();
        let start = calculate_start(page, limit, total);
        let page_data: Vec<LevelRankingResponse> = indexed
            .all
            .iter()
            .skip(start)
            .take(limit)
            .map(LevelRankingResponse::from)
            .collect();
        (total, page_data)
    };

    let total_pages = if total_items == 0 {
        1
    } else {
        (total_items as f64 / limit as f64).ceil() as usize
    };

    let current_page = if page > total_pages {
        total_pages
    } else {
        page
    };

    HttpResponse::Ok().json(json!({
        "page": current_page,
        "limit": limit,
        "total_pages": total_pages,
        "total_items": total_items,
        "data": paginated
    }))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_gold_ranking);
    cfg.service(get_level_ranking);
}
