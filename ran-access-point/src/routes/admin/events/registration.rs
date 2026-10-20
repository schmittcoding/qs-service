use std::sync::Arc;

use actix_web::{delete, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::get_admin_id;

#[derive(Debug, Deserialize)]
struct AdminUpdateRegistrationRequest {
    /// 1=Active, 2=Withdrawn, 3=Disqualified
    pub status: Option<i16>,
    /// Set to null to clear guild, or Some(guild_num) to change guild.
    /// Use Option<Option<i32>> so JSON `"guild_num": null` clears it.
    pub guild_num: Option<Option<i32>>,
}

/// PUT /admin/events/{event_id}/registrations/{cha_num}
#[utoipa::path(
    put,
    path = "/admin/v1/events/{event_id}/registrations/{cha_num}",
    operation_id = "admin_update_registration",
    tag = "Admin - Registration",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("cha_num" = i32, Path, description = "Character number")
    ),
    responses(
        (status = 200, description = "Registration updated", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Registration not found"),
        (status = 500, description = "Database error")
    )
)]
#[put("/{event_id}/registrations/{cha_num}")]
pub async fn admin_update_registration(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, i32)>,
    body: web::Json<AdminUpdateRegistrationRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (event_id, cha_num) = path.into_inner();
    let req_body = body.into_inner();

    if req_body.status.is_none() && req_body.guild_num.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "VALIDATION_FAILED",
            "message": "At least one field (status or guild_num) must be provided"
        }));
    }

    match db.web_db.admin_update_registration(event_id, cha_num, req_body.status, req_body.guild_num).await {
        Ok(Some(reg)) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Registration updated",
            "registration": reg
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "code": "NOT_FOUND",
            "message": "Registration not found for this event and character"
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.starts_with("VALIDATION:") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "VALIDATION_FAILED",
                    "message": msg.strip_prefix("VALIDATION:").unwrap_or(&msg)
                }))
            } else {
                tracing::error!("Failed to update registration: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "code": "DB_ERROR",
                    "message": "Failed to update registration"
                }))
            }
        }
    }
}

/// DELETE /admin/events/{event_id}/registrations/{cha_num}
#[utoipa::path(
    delete,
    path = "/admin/v1/events/{event_id}/registrations/{cha_num}",
    operation_id = "admin_remove_participant",
    tag = "Admin - Registration",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("cha_num" = i32, Path, description = "Character number")
    ),
    responses(
        (status = 200, description = "Participant removed", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Registration not found"),
        (status = 500, description = "Database error")
    )
)]
#[delete("/{event_id}/registrations/{cha_num}")]
pub async fn admin_remove_participant(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, i32)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (event_id, cha_num) = path.into_inner();

    let _existing = match db.web_db.get_registration_by_event_and_char(event_id, cha_num).await {
        Ok(Some(reg)) => reg,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Registration not found"
            }));
        }
        Err(e) => {
            tracing::error!("Failed to fetch registration: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch registration"
            }));
        }
    };

    match db.web_db.admin_remove_registration(event_id, cha_num).await {
        Ok(true) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Participant removed"
        })),
        Ok(false) => HttpResponse::NotFound().json(json!({
            "success": false, "code": "NOT_FOUND", "message": "Registration not found"
        })),
        Err(e) => {
            tracing::error!("Failed to remove participant: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to remove participant"
            }))
        }
    }
}
