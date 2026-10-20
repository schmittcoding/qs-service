pub mod crud;
pub mod deductions;
pub mod export;
pub mod gimmicks;
pub mod leaderboard;
pub mod match_management;
pub mod participants;
pub mod registration;
pub mod tally;

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::services::jwt::Claims;

fn get_admin_id(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(json!({
                "success": false,
                "code": "UNAUTHORIZED",
                "message": "Missing authentication"
            }))
        })?;

    Uuid::parse_str(&claims.sub).map_err(|_| {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "INVALID_TOKEN",
            "message": "Invalid admin user ID in token"
        }))
    })
}

fn map_event_error(e: anyhow::Error) -> HttpResponse {
    let msg = e.to_string();

    if let Some(rest) = msg.strip_prefix("NOT_FOUND:") {
        HttpResponse::NotFound().json(json!({
            "success": false, "code": "NOT_FOUND", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("SEASON_CONFLICT:") {
        HttpResponse::Conflict().json(json!({
            "success": false, "code": "SEASON_CONFLICT", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("INVALID_DATES:") {
        HttpResponse::BadRequest().json(json!({
            "success": false, "code": "INVALID_DATES", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("VALIDATION:") {
        HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("LOCKED:") {
        HttpResponse::UnprocessableEntity().json(json!({
            "success": false, "code": "LOCKED", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("NO_MATCHES:") {
        HttpResponse::BadRequest().json(json!({
            "success": false, "code": "NO_MATCHES", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("TALLY_IN_PROGRESS:") {
        HttpResponse::Conflict().json(json!({
            "success": false, "code": "TALLY_IN_PROGRESS", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("OUT_OF_EVENT_RANGE:") {
        HttpResponse::BadRequest().json(json!({
            "success": false, "code": "OUT_OF_EVENT_RANGE", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("OUT_OF_MATCH_RANGE:") {
        HttpResponse::BadRequest().json(json!({
            "success": false, "code": "OUT_OF_MATCH_RANGE", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("MATCH_OVERLAP:") {
        HttpResponse::Conflict().json(json!({
            "success": false, "code": "MATCH_OVERLAP", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("MATCH_LOCKED:") {
        HttpResponse::UnprocessableEntity().json(json!({
            "success": false, "code": "MATCH_LOCKED", "message": rest
        }))
    } else if let Some(rest) = msg.strip_prefix("INVALID_STATUS:") {
        HttpResponse::UnprocessableEntity().json(json!({
            "success": false, "code": "INVALID_STATUS", "message": rest
        }))
    } else {
        tracing::error!("Unexpected error: {:?}", e);
        HttpResponse::InternalServerError().json(json!({
            "success": false, "code": "INTERNAL", "message": "Internal server error"
        }))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Event CRUD
        .service(crud::create_event)
        .service(crud::list_events)
        .service(crud::get_event_dashboard)
        .service(crud::activate_event)
        .service(crud::deactivate_event)
        .service(crud::get_event_detail)
        .service(crud::update_event)
        // Match management
        .service(match_management::create_matches)
        .service(match_management::generate_matches)
        .service(match_management::list_matches)
        .service(match_management::update_match)
        .service(match_management::delete_match)
        // Gimmicks
        .service(gimmicks::create_gimmick)
        .service(gimmicks::list_gimmicks)
        .service(gimmicks::delete_gimmick)
        // Participants
        .service(participants::list_participants_by_character)
        .service(participants::list_participants_by_guild)
        // Admin participation overrides
        .service(registration::admin_remove_participant)
        // CSV Export
        .service(export::export_characters_csv)
        .service(export::export_guilds_csv)
        .service(export::export_guild_match_results_csv)
        .service(export::export_koth_match_results_csv)
        // Admin registration update
        .service(registration::admin_update_registration)
        // Deductions
        .service(deductions::list_deductions)
        .service(deductions::apply_deduction)
        .service(deductions::revert_deduction)
        // Leaderboards
        .service(leaderboard::admin_generic_leaderboard)
        .service(leaderboard::admin_guild_leaderboard)
        .service(leaderboard::admin_school_leaderboard)
        .service(leaderboard::admin_koth_leaderboard)
        // Re-tally / status overrides / tower results
        .service(tally::retally_match)
        .service(tally::set_tower_results)
        .service(tally::set_match_waiting)
        .service(tally::force_tally_level_cap_race);
}
