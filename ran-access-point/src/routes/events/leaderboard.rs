use actix_web::{get, web, HttpResponse, Responder};
use ran_models::events::category::EventCategory;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::debug;

use crate::database::DBState;
use crate::states::AppState;

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub cha_class: Option<i32>,
}

/// GET /events/{event_slug}/leaderboard/guild — guild rankings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/leaderboard/guild",
    tag = "Events - Leaderboard",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "Guild leaderboard with pagination", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/leaderboard/guild")]
pub async fn guild_leaderboard(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    let event_slug = path.into_inner();

    let event = match db.web_db.get_event_by_slug(&event_slug).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch event"
            }));
        }
    };
    let event_id = event.event_id;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);

    match db
        .web_db
        .get_guild_leaderboard(event_id, page, page_size)
        .await
    {
        Ok((scores, total)) => HttpResponse::Ok().json(json!({
            "success": true,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_items": total
            },
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch guild leaderboard: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch guild rankings"
            }))
        }
    }
}

/// GET /events/{event_slug}/leaderboard/school — school rankings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/leaderboard/school",
    tag = "Events - Leaderboard",
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "School leaderboard", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/leaderboard/school")]
pub async fn school_leaderboard(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
) -> impl Responder {
    let event_slug = path.into_inner();

    let event = match db.web_db.get_event_by_slug(&event_slug).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch event"
            }));
        }
    };
    let event_id = event.event_id;

    match db.web_db.get_school_leaderboard(event_id).await {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch school leaderboard: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch school rankings"
            }))
        }
    }
}

/// GET /events/{event_slug}/leaderboard/koth — KOTH per-class rankings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/leaderboard/koth",
    tag = "Events - Leaderboard",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "KOTH leaderboard with pagination", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/leaderboard/koth")]
pub async fn koth_leaderboard(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    let event_slug = path.into_inner();

    let event = match db.web_db.get_event_by_slug(&event_slug).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch event"
            }));
        }
    };
    let event_id = event.event_id;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);

    match db
        .web_db
        .get_koth_leaderboard(event_id, params.cha_class, page, page_size)
        .await
    {
        Ok((scores, total)) => HttpResponse::Ok().json(json!({
            "success": true,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_items": total
            },
            "cha_class": params.cha_class,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch KOTH leaderboard: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch KOTH rankings"
            }))
        }
    }
}

/// GET /events/{event_slug}/leaderboard — generic leaderboard dispatched by event category
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/leaderboard",
    tag = "Events - Leaderboard",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "Leaderboard dispatched by event category", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/leaderboard")]
pub async fn generic_leaderboard(
    db: web::Data<Arc<DBState>>,
    _states: web::Data<Arc<AppState>>,
    path: web::Path<String>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    let event_slug = path.into_inner();

    // Load event to determine category
    let event = match db.web_db.get_event_by_slug(&event_slug).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch event"
            }));
        }
    };
    let event_id = event.event_id;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);

    match event.event_category {
        EventCategory::Gvg => {
            match db.web_db.get_guild_leaderboard(event_id, page, page_size).await {
                Ok((scores, total)) => HttpResponse::Ok().json(json!({
                    "success": true,
                    "event_category": "gvg",
                    "leaderboard_type": "guild",
                    "pagination": {
                        "page": page,
                        "page_size": page_size,
                        "total_items": total
                    },
                    "rankings": scores
                })),
                Err(e) => {
                    log::error!("Failed to fetch guild leaderboard: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false, "code": "DB_ERROR", "message": "Failed to fetch leaderboard"
                    }))
                }
            }
        }
        EventCategory::Koth => {
            match db.web_db.get_koth_leaderboard(event_id, params.cha_class, page, page_size).await {
                Ok((scores, total)) => HttpResponse::Ok().json(json!({
                    "success": true,
                    "event_category": "koth",
                    "leaderboard_type": "character",
                    "pagination": {
                        "page": page,
                        "page_size": page_size,
                        "total_items": total
                    },
                    "cha_class": params.cha_class,
                    "rankings": scores
                })),
                Err(e) => {
                    log::error!("Failed to fetch KOTH leaderboard: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false, "code": "DB_ERROR", "message": "Failed to fetch leaderboard"
                    }))
                }
            }
        }
        EventCategory::LevelCapRace => {
            let cache_ttl = Duration::from_secs(1800); // 30 minutes

            // Check cache first
            let cached_data = {
                let cache = _states.level_cap_race_leaderboard_cache.lock().await;
                cache.get(&event_id).cloned()
            };

            let all_scores = if let Some(cached) = cached_data {
                // Check if cache is stale
                if cached.last_fetched_at.elapsed() < cache_ttl {
                    debug!("LevelCapRace leaderboard cache hit for event {}", event_id);
                    cached.scores
                } else {
                    // Cache is stale, fetch fresh data
                    debug!("LevelCapRace leaderboard cache stale for event {}", event_id);
                    match db
                        .web_db
                        .get_level_cap_race_leaderboard(event_id, None, 1, i32::MAX as u32)
                        .await
                    {
                        Ok((scores, total)) => {
                            // Populate cache with full result set
                            let mut cache = _states.level_cap_race_leaderboard_cache.lock().await;
                            cache.insert(
                                event_id,
                                crate::states::CachedLevelCapRaceLeaderboard {
                                    scores: scores.clone(),
                                    total_count: total,
                                    last_fetched_at: Instant::now(),
                                },
                            );
                            drop(cache);
                            scores
                        }
                        Err(e) => {
                            log::error!("Failed to fetch LevelCapRace leaderboard: {:?}", e);
                            return HttpResponse::InternalServerError().json(json!({
                                "success": false, "code": "DB_ERROR", "message": "Failed to fetch leaderboard"
                            }));
                        }
                    }
                }
            } else {
                // Cache miss, fetch full data set
                debug!("LevelCapRace leaderboard cache miss for event {}", event_id);
                match db
                    .web_db
                    .get_level_cap_race_leaderboard(event_id, None, 1, i32::MAX as u32)
                    .await
                {
                    Ok((scores, total)) => {
                        // Populate cache with full result set
                        let mut cache = _states.level_cap_race_leaderboard_cache.lock().await;
                        cache.insert(
                            event_id,
                            crate::states::CachedLevelCapRaceLeaderboard {
                                scores: scores.clone(),
                                total_count: total,
                                last_fetched_at: Instant::now(),
                            },
                        );
                        drop(cache);
                        scores
                    }
                    Err(e) => {
                        log::error!("Failed to fetch LevelCapRace leaderboard: {:?}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "success": false, "code": "DB_ERROR", "message": "Failed to fetch leaderboard"
                        }));
                    }
                }
            };

            // Apply class filtering and pagination in memory
            let mut filtered_scores = all_scores;
            if let Some(cha_class) = params.cha_class {
                filtered_scores.retain(|s| s.cha_class == cha_class);
            }

            let filtered_total = filtered_scores.len() as i64;
            let offset = (page.saturating_sub(1)) * page_size;
            let paginated_scores: Vec<_> = filtered_scores
                .into_iter()
                .skip(offset as usize)
                .take(page_size as usize)
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "event_category": "level_cap_race",
                "leaderboard_type": "character",
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total_items": filtered_total
                },
                "cha_class": params.cha_class,
                "rankings": paginated_scores
            }))
        }
        EventCategory::SpeedRun => {
            HttpResponse::NotImplemented().json(json!({
                "success": false,
                "code": "NOT_IMPLEMENTED",
                "message": format!("Leaderboard for {} events is not yet available", event.event_category)
            }))
        }
        EventCategory::GvgKoth => {
            let guild_result = db.web_db.get_guild_leaderboard(event_id, page, page_size).await;
            let school_result = db.web_db.get_school_leaderboard(event_id).await;
            let koth_result = db.web_db.get_koth_leaderboard(event_id, params.cha_class, page, page_size).await;

            match (guild_result, school_result, koth_result) {
                (Ok((guild_scores, guild_total)), Ok(school_scores), Ok((koth_scores, koth_total))) => {
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "event_category": "gvg_koth",
                        "leaderboard_type": "combined",
                        "guild_rankings": guild_scores,
                        "school_rankings": school_scores,
                        "koth_rankings": koth_scores,
                        "pagination": {
                            "page": page,
                            "page_size": page_size,
                            "guild_total": guild_total,
                            "koth_total": koth_total
                        }
                    }))
                }
                _ => {
                    log::error!("Failed to fetch combined leaderboard for GvGKoth event");
                    HttpResponse::InternalServerError().json(json!({
                        "success": false, "code": "DB_ERROR", "message": "Failed to fetch leaderboard"
                    }))
                }
            }
        }
    }
}
