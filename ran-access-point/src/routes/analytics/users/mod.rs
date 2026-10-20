use std::{ sync::Arc, time::Instant};


use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

use crate::{database::DBState, states::AppState};

#[get("/online")]
async fn get_online_players(
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>
) -> impl Responder {
    let cache_instance = states.online_players_cache.lock().await;

    let data_guard = cache_instance.data.lock().await;
    let last_fetched_at_guard = cache_instance.last_fetched_at.lock().await;

    if let (Some(data), Some(last_fetched_at)) = (&*data_guard, *last_fetched_at_guard) {
        if last_fetched_at.elapsed() <= cache_instance.refresh_interval {
            log::info!("Cache is fresh, returning existing data.");

            return HttpResponse::Ok()
                .json(json!({ 
                    "data": data.clone(),
                    "total_items": data.clone().len()
                }))
        }
    }

    drop(data_guard);
    drop(last_fetched_at_guard);

    let mut fetching_guard = cache_instance.fetching_in_progress.lock().await;

    if !*fetching_guard {
        *fetching_guard = true;
        drop(fetching_guard);

        log::info!("Cache is stale or empty, initiating new data fetch.");
        let new_data = match db.user_db.get_online_account_analytics().await {
            Ok(res) => res,
            Err(err) => {
                let mut fetching_guard_on_error = cache_instance.fetching_in_progress.lock().await;
                *fetching_guard_on_error = false;
                drop(fetching_guard_on_error); // Release the lock

                log::error!("❌ Failed to fetch online accounts: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to fetch online accounts" }));
            }
        };
        *cache_instance.data.lock().await = Some(new_data.clone());
        *cache_instance.last_fetched_at.lock().await = Some(Instant::now());
        *cache_instance.fetching_in_progress.lock().await = false;

        log::info!("New data fetched and cached.");

        drop(cache_instance); // Release the outer lock

        HttpResponse::Ok()
            .json(json!({ 
                "data": new_data.clone(),
                "total_items": new_data.clone().len()
            }))
    } else {
        log::info!("Another fetch is in progress, waiting for it to complete...");
        drop(fetching_guard);

        HttpResponse::InternalServerError()
            .json(json!({ "error": "Failed to fetch online accounts" }))
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_online_players);
}
