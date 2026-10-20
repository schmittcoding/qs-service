use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::category::EventCategory;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::DBState;

use super::{get_admin_id, map_event_error};

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub cha_class: Option<i32>,
}

/// GET /admin/events/{event_id}/leaderboard/guild — guild rankings
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/leaderboard/guild",
    operation_id = "admin_guild_leaderboard",
    tag = "Admin - Leaderboard",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "Guild leaderboard with pagination", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/leaderboard/guild")]
pub async fn admin_guild_leaderboard(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let _event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false,
                "code": "NOT_FOUND",
                "message": "Event not found"
            }));
        }
        Err(e) => return map_event_error(e),
    };

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
            tracing::error!("Failed to fetch guild leaderboard for event {}: {}", event_id, e);
            map_event_error(e)
        }
    }
}

/// GET /admin/events/{event_id}/leaderboard/school — school rankings
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/leaderboard/school",
    operation_id = "admin_school_leaderboard",
    tag = "Admin - Leaderboard",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "School leaderboard", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/leaderboard/school")]
pub async fn admin_school_leaderboard(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let _event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false,
                "code": "NOT_FOUND",
                "message": "Event not found"
            }));
        }
        Err(e) => return map_event_error(e),
    };

    match db.web_db.get_school_leaderboard(event_id).await {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "rankings": scores
        })),
        Err(e) => {
            tracing::error!("Failed to fetch school leaderboard for event {}: {}", event_id, e);
            map_event_error(e)
        }
    }
}

/// GET /admin/events/{event_id}/leaderboard/koth — KOTH per-class rankings
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/leaderboard/koth",
    operation_id = "admin_koth_leaderboard",
    tag = "Admin - Leaderboard",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "KOTH leaderboard with pagination", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/leaderboard/koth")]
pub async fn admin_koth_leaderboard(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let _event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false,
                "code": "NOT_FOUND",
                "message": "Event not found"
            }));
        }
        Err(e) => return map_event_error(e),
    };

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
            tracing::error!("Failed to fetch KOTH leaderboard for event {}: {}", event_id, e);
            map_event_error(e)
        }
    }
}

/// GET /admin/events/{event_id}/leaderboard — generic leaderboard dispatched by event category
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/leaderboard",
    operation_id = "admin_generic_leaderboard",
    tag = "Admin - Leaderboard",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "Leaderboard dispatched by event category", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error"),
        (status = 501, description = "Leaderboard type not yet implemented")
    )
)]
#[get("/{event_id}/leaderboard")]
pub async fn admin_generic_leaderboard(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    params: web::Query<LeaderboardQuery>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false,
                "code": "NOT_FOUND",
                "message": "Event not found"
            }));
        }
        Err(e) => return map_event_error(e),
    };

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
                    tracing::error!("Failed to fetch guild leaderboard for event {}: {}", event_id, e);
                    map_event_error(e)
                }
            }
        }
        EventCategory::Koth => {
            match db
                .web_db
                .get_koth_leaderboard(event_id, params.cha_class, page, page_size)
                .await
            {
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
                    tracing::error!("Failed to fetch KOTH leaderboard for event {}: {}", event_id, e);
                    map_event_error(e)
                }
            }
        }
        EventCategory::LevelCapRace => {
            // Admin bypass: fetch fresh data, no cache
            match db
                .web_db
                .get_level_cap_race_leaderboard(event_id, None, 1, i32::MAX as u32)
                .await
            {
                Ok((all_scores, _)) => {
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
                Err(e) => {
                    tracing::error!("Failed to fetch level cap race leaderboard for event {}: {}", event_id, e);
                    map_event_error(e)
                }
            }
        }
        EventCategory::SpeedRun => {
            HttpResponse::NotImplemented().json(json!({
                "success": false,
                "code": "NOT_IMPLEMENTED",
                "message": "Leaderboard for speedrun events is not yet available"
            }))
        }
        EventCategory::GvgKoth => {
            let guild_result = db.web_db.get_guild_leaderboard(event_id, page, page_size).await;
            let school_result = db.web_db.get_school_leaderboard(event_id).await;
            let koth_result = db
                .web_db
                .get_koth_leaderboard(event_id, params.cha_class, page, page_size)
                .await;

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
                    tracing::error!("Failed to fetch combined leaderboard for GvGKoth event {}", event_id);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "code": "INTERNAL",
                        "message": "Failed to fetch leaderboard"
                    }))
                }
            }
        }
    }
}
