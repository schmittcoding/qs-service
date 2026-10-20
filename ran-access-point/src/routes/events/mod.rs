pub mod leaderboard;
pub mod match_status;
pub mod participants;
pub mod registration;
pub mod snapshots;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::Local;
use serde_json::json;
use std::sync::Arc;

use crate::database::DBState;

#[utoipa::path(
    get,
    path = "/api/v1/events/health",
    tag = "Events",
    responses(
        (status = 200, description = "Event system health", body = serde_json::Value),
        (status = 500, description = "Event system unhealthy")
    )
)]
#[get("/health")]
pub async fn event_health(data: web::Data<Arc<DBState>>) -> impl Responder {
    match data.web_db.event_health_check().await {
        Ok(true) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Event system healthy"
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": "Event system unhealthy"
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/events",
    tag = "Events",
    responses(
        (status = 200, description = "List of events", body = serde_json::Value),
        (status = 500, description = "Database error")
    )
)]
#[get("")]
pub async fn list_events(data: web::Data<Arc<DBState>>) -> impl Responder {
    match data.web_db.list_events_public().await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "success": true,
            "events": events
        })),
        Err(e) => {
            tracing::error!("Failed to fetch events list: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to fetch events"
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}",
    tag = "Events",
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "Event details with registration availability", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("")]
pub async fn get_event_by_slug(
    data: web::Data<Arc<DBState>>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    match data.web_db.get_event_by_slug(&slug).await {
        Ok(Some(event)) => {
            let now = Local::now().naive_local();
            let is_registration_available = event.is_registration_open(now);
            HttpResponse::Ok().json(json!({
                "success": true,
                "event": event,
                "is_registration_available": is_registration_available
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "code": "NOT_FOUND",
            "message": "No active event found for the given slug"
        })),
        Err(e) => {
            tracing::error!("Failed to fetch event by slug '{}': {:?}", slug, e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to fetch event"
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(event_health)
        .service(list_events)
        .service(
            web::scope("/{event_slug}")
                // Event details
                .service(get_event_by_slug)
                // Registration
                .service(registration::get_eligible_characters)
                .service(registration::register_character)
                .service(registration::register_guild)
                .service(registration::get_registration_status)
                .service(registration::withdraw_registration)
                .service(registration::get_guild_members)
                // Match status
                .service(match_status::get_match_status)
                .service(match_status::get_matches)
                // Leaderboards
                .service(leaderboard::generic_leaderboard)
                .service(leaderboard::guild_leaderboard)
                .service(leaderboard::school_leaderboard)
                .service(leaderboard::koth_leaderboard)
                // Participants (public)
                .service(participants::participant_characters)
                .service(participants::participant_guilds)
                // Snapshots
                .service(snapshots::list_snapshots)
                .service(snapshots::snapshot_guild_scores)
                .service(snapshots::snapshot_koth_scores)
                .service(snapshots::snapshot_school_scores)
                .service(snapshots::snapshot_level_cap_race_scores),
        );
}
