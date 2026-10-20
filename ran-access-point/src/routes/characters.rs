use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::{get, web, HttpResponse, Responder};
use once_cell::sync::Lazy;
use ran_db_core::gamemanager::admin::characters::GameCharacterTrait;
use ran_models::admin::BasicPaginationParams;
use ran_models::admin::players::UserCharacterList;
use serde_json::json;
use tokio::sync::RwLock;

use crate::{database::DBState, middleware::session::get_user_session};

struct CachedCharacters {
    data: Vec<UserCharacterList>,
    cached_at: Instant,
}

static CHARACTER_CACHE: Lazy<RwLock<HashMap<i32, CachedCharacters>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL: Duration = Duration::from_secs(60);

#[get("")]
async fn get_characters(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    params: web::Query<BasicPaginationParams>,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "No session found",
                "message": "User not logged in"
            }));
        }
    };

    let user_num = profile.user_num as i32;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);

    // Try cache first
    {
        let cache = CHARACTER_CACHE.read().await;
        if let Some(cached) = cache.get(&user_num) {
            if cached.cached_at.elapsed() < CACHE_TTL {
                return build_paginated_response(&cached.data, page, limit);
            }
        }
    }

    // Cache miss or expired — fetch all characters from DB
    match db.game_db.get_characters_by_user(1, 1000, user_num).await {
        Ok(res) => {
            let characters: Vec<UserCharacterList> = res
                .data
                .iter()
                .map(|row| UserCharacterList::from_row(row))
                .collect();

            let response = build_paginated_response(&characters, page, limit);

            // Update cache
            let mut cache = CHARACTER_CACHE.write().await;
            cache.insert(
                user_num,
                CachedCharacters {
                    data: characters,
                    cached_at: Instant::now(),
                },
            );

            response
        }
        Err(e) => {
            log::error!("Failed to fetch characters: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch characters" }))
        }
    }
}

fn build_paginated_response(
    data: &[UserCharacterList],
    page: usize,
    limit: usize,
) -> HttpResponse {
    let total_items = data.len();
    let total_pages = total_items.div_ceil(limit);
    let start = (page - 1) * limit;
    let paginated: Vec<&UserCharacterList> = data.iter().skip(start).take(limit).collect();

    HttpResponse::Ok().json(json!({
        "page": page,
        "limit": limit,
        "total_pages": total_pages,
        "total_items": total_items,
        "data": paginated
    }))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_characters);
}
