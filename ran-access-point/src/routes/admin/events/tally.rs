use std::sync::Arc;

use actix_web::{patch, post, put, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::category::EventCategory;
use ran_models::events::event::SetTowerResultsRequest;
use serde_json::json;
use uuid::Uuid;

use crate::{database::DBState, states::AppState};
use crate::services::discord;

use super::{get_admin_id, map_event_error};

/// POST /admin/events/{event_id}/matches/{match_id}/retally
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/matches/{match_id}/retally",
    operation_id = "admin_retally_match",
    tag = "Admin - Tally",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "Match prepared for re-tally", body = serde_json::Value),
        (status = 400, description = "Invalid match status"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Match not found"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/matches/{match_id}/retally")]
pub async fn retally_match(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (_event_id, match_id) = path.into_inner();

    match db.web_db.prepare_retally(match_id).await {
        Ok(match_info) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Match has been reset for re-tally. The tally scheduler will pick it up shortly.",
            "match": match_info
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "code": "NOT_FOUND",
                    "message": "Match not found"
                }))
            } else if msg.contains("Cannot re-tally") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "INVALID_STATUS",
                    "message": msg
                }))
            } else {
                tracing::error!("Failed to prepare re-tally: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "code": "DB_ERROR",
                    "message": "Failed to prepare re-tally"
                }))
            }
        }
    }
}

/// PATCH /admin/events/matches/{match_id}/set-waiting
#[utoipa::path(
    patch,
    path = "/admin/v1/events/matches/{match_id}/set-waiting",
    operation_id = "admin_set_match_waiting",
    tag = "Admin - Tally",
    security(("bearer_jwt" = [])),
    params(
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "Match status set to Waiting", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Match not found or invalid status transition"),
        (status = 500, description = "Database error")
    )
)]
#[patch("/matches/{match_id}/set-waiting")]
pub async fn set_match_waiting(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let match_id = path.into_inner();

    match db.web_db.set_match_waiting_admin(match_id).await {
        Ok(true) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Match status set to 'Waiting'. The tally scheduler will pick it up shortly.",
            "matchId": match_id
        })),
        Ok(false) => HttpResponse::UnprocessableEntity().json(json!({
            "success": false,
            "code": "INVALID_STATUS",
            "message": "Match not found or current status does not allow transition to Waiting. Only Pending or Failed matches can be set to Waiting."
        })),
        Err(e) => {
            tracing::error!("Failed to set match to Waiting: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to update match status"
            }))
        }
    }
}

/// PUT /admin/events/matches/{match_id}/tower-results
#[utoipa::path(
    put,
    path = "/admin/v1/events/matches/{match_id}/tower-results",
    operation_id = "admin_set_tower_results",
    tag = "Admin - Tally",
    security(("bearer_jwt" = [])),
    params(
        ("match_id" = String, Path, description = "Match UUID")
    ),
    request_body = SetTowerResultsRequest,
    responses(
        (status = 200, description = "Tower results saved, match set to Waiting", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[put("/matches/{match_id}/tower-results")]
pub async fn set_tower_results(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<SetTowerResultsRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let match_id = path.into_inner();

    match db.web_db.set_match_tower_results(match_id, &body.into_inner()).await {
        Ok(m) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Tower results saved. Match status set to 'Waiting' — tally will run shortly.",
            "match": m
        })),
        Err(e) => map_event_error(e),
    }
}

/// POST /admin/events/{event_id}/level-cap-race/force-tally
/// Force immediate tally for a LevelCapRace event (bypasses scheduler wait)
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/level-cap-race/force-tally",
    operation_id = "admin_force_tally_level_cap_race",
    tag = "Admin - Tally",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "Level cap race tally completed", body = serde_json::Value),
        (status = 400, description = "Not a level cap race event"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/level-cap-race/force-tally")]
pub async fn force_tally_level_cap_race(
    req: HttpRequest,
    state: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    // Load event and validate it's LevelCapRace
    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false,
                "code": "NOT_FOUND",
                "message": "Event not found"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to load event for force-tally: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to load event"
            }));
        }
    };

    if event.event_category != EventCategory::LevelCapRace {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "INVALID_EVENT_TYPE",
            "message": format!("Event is {}, not LevelCapRace", event.event_category)
        }));
    }

    state.level_cap_race_leaderboard_cache.lock().await.clear();

    // Execute the tally directly (level cap race doesn't have match tracking)
    let tally_result = match db
        .web_db
        .execute_category_tally(
            Uuid::nil(),
            event_id,
            event.event_start,
            event.event_end,
            &event.event_category,
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Force-tally failed for event {}: {:?}", event_id, e);
            return map_event_error(e);
        }
    };

    // For LevelCapRace, fetch new winners and announce them (fire-and-forget)
    // let started_at = match_info.start_time;
    // let mut new_winners_count = 0;
    // if let Ok(winners) = db.web_db.take_lcr_winners_since(event_id, started_at).await {
    //     new_winners_count = winners.len() as i32;
    //     if !winners.is_empty() {
    //         let event_name = event.event_name.clone();
    //         tokio::spawn(async move {
    //             discord::announce_level_cap_winners(event_name.as_str(), &winners).await
    //         });
    //     }
    // }

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Tally completed successfully",
        "stats": {
            "characters_processed": tally_result.rows_tallied,
            // "new_winners": new_winners_count
        }
    }))
}
