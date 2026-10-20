use actix_web::{get, web, HttpResponse, Responder};
use ran_models::events::match_schedule::MatchWithGimmicks;
use serde_json::json;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::DBState;

/// GET /events/{event_slug}/match-status — frontend banner state
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/match-status",
    tag = "Events - Match Status",
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "Current match status", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/match-status")]
pub async fn get_match_status(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
) -> impl Responder {
    let event_slug = path.into_inner();

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

    match db.web_db.get_current_match_status(event_id).await {
        Ok(Some(m)) => {
            let now = chrono::Local::now().naive_local();
            let status = if now < m.end_time {
                "LIVE"
            } else if m.tally_status == "Processing" {
                "TALLYING_NOW"
            } else {
                "TALLYING_SOON"
            };

            HttpResponse::Ok().json(json!({
                "success": true,
                "match_ongoing": true,
                "status": status,
                "match_id": m.match_id,
                "match_label": m.match_label,
                "start_time": m.start_time,
                "end_time": m.end_time,
                "tally_scheduled": m.tally_scheduled,
                "message": format!(
                    "The match is ongoing. Tally will be updated after {}.",
                    m.tally_scheduled.format("%I:%M %p")
                )
            }))
        }
        Ok(None) => HttpResponse::Ok().json(json!({
            "success": true,
            "match_ongoing": false,
            "status": "IDLE"
        })),
        Err(e) => {
            log::error!("Failed to get match status: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to get match status"
            }))
        }
    }
}

/// GET /events/{event_slug}/matches — list all matches
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/matches",
    tag = "Events - Match Status",
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "List of matches with gimmicks", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/matches")]
pub async fn get_matches(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
) -> impl Responder {
    let event_slug = path.into_inner();

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

    let matches = match db.web_db.get_matches_by_event(event_id).await {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to fetch matches: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch matches"
            }));
        }
    };

    let gimmicks = match db.web_db.get_gimmicks_by_event(event_id).await {
        Ok(g) => g,
        Err(e) => {
            log::error!("Failed to fetch gimmicks: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch gimmicks"
            }));
        }
    };

    let mut gimmick_map: HashMap<Uuid, Vec<_>> = HashMap::new();
    for g in gimmicks {
        gimmick_map.entry(g.match_id).or_default().push(g);
    }

    let matches_with_gimmicks: Vec<MatchWithGimmicks> = matches
        .into_iter()
        .map(|m| {
            let gimmicks = gimmick_map.remove(&m.match_id).unwrap_or_default();
            MatchWithGimmicks { match_schedule: m, gimmicks }
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "success": true,
        "matches": matches_with_gimmicks
    }))
}
