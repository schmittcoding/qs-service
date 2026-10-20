use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::get_admin_id;

/// GET /admin/events/{event_id}/participants/characters
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/participants/characters",
    operation_id = "admin_list_participants_by_character",
    tag = "Admin - Participants",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "List of character participants", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/participants/characters")]
pub async fn list_participants_by_character(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    match db.web_db.list_participants_by_character(event_id).await {
        Ok(rows) => {
            let participants: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "registration_id": r.get::<uuid::Uuid, _>("RegistrationID").map(|u| u.to_string()),
                        "cha_num": r.get::<i32, _>("ChaNum").unwrap_or_default(),
                        "cha_name": r.get::<&str, _>("ChaName").unwrap_or(""),
                        "user_num": r.get::<i32, _>("UserNum").unwrap_or_default(),
                        "guild_num": r.get::<i32, _>("GuildNum"),
                        "guild_name": r.get::<&str, _>("GuildName"),
                        "cha_school": r.get::<i32, _>("ChaSchool"),
                        "cha_class": r.get::<i32, _>("ChaClass").unwrap_or_default(),
                        "cha_level": r.get::<i32, _>("ChaLevel").unwrap_or_default(),
                        "status": r.get::<u8, _>("Status").unwrap_or(1),
                        "registered_at": r.get::<chrono::NaiveDateTime, _>("RegisteredAt").map(|d| d.to_string()),
                        "updated_at": r.get::<chrono::NaiveDateTime, _>("UpdatedAt").map(|d| d.to_string()),
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "total": participants.len(),
                "participants": participants
            }))
        }
        Err(e) => {
            tracing::error!("Failed to list participants by character: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to list participants"
            }))
        }
    }
}

/// GET /admin/events/{event_id}/participants/guilds
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/participants/guilds",
    operation_id = "admin_list_participants_by_guild",
    tag = "Admin - Participants",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "List of guild participants", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/participants/guilds")]
pub async fn list_participants_by_guild(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    match db.web_db.list_participants_by_guild(event_id).await {
        Ok(rows) => {
            let guilds: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "guild_num": r.get::<i32, _>("GuildNum").unwrap_or_default(),
                        "guild_name": r.get::<&str, _>("GuildName"),
                        "cha_school": r.get::<i32, _>("ChaSchool"),
                        "member_count": r.get::<i32, _>("MemberCount").unwrap_or(0),
                        "active_count": r.get::<i32, _>("ActiveCount").unwrap_or(0),
                        "withdrawn_count": r.get::<i32, _>("WithdrawnCount").unwrap_or(0),
                        "disqualified_count": r.get::<i32, _>("DisqualifiedCount").unwrap_or(0),
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "total": guilds.len(),
                "participants": guilds
            }))
        }
        Err(e) => {
            tracing::error!("Failed to list participants by guild: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to list participants"
            }))
        }
    }
}
