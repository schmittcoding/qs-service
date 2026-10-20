use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::gimmick::CreateGimmickRequest;
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::{get_admin_id, map_event_error};

/// POST /admin/events/{event_id}/matches/{match_id}/gimmicks
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/matches/{match_id}/gimmicks",
    operation_id = "admin_create_gimmick",
    tag = "Admin - Gimmicks",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("match_id" = String, Path, description = "Match UUID")
    ),
    request_body = CreateGimmickRequest,
    responses(
        (status = 201, description = "Gimmick created", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/matches/{match_id}/gimmicks")]
pub async fn create_gimmick(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
    body: web::Json<CreateGimmickRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (_event_id, match_id) = path.into_inner();

    match db.web_db.create_gimmick(match_id, &body.into_inner()).await {
        Ok(gimmick) => HttpResponse::Created().json(json!({
            "success": true,
            "gimmick": gimmick
        })),
        Err(e) => map_event_error(e),
    }
}

/// GET /admin/events/{event_id}/matches/{match_id}/gimmicks
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/matches/{match_id}/gimmicks",
    operation_id = "admin_list_gimmicks",
    tag = "Admin - Gimmicks",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "List of gimmicks", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/matches/{match_id}/gimmicks")]
pub async fn list_gimmicks(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (_event_id, match_id) = path.into_inner();

    match db.web_db.list_gimmicks_for_match(match_id).await {
        Ok(gimmicks) => HttpResponse::Ok().json(json!({
            "success": true,
            "gimmicks": gimmicks
        })),
        Err(e) => map_event_error(e),
    }
}

/// DELETE /admin/events/matches/{match_id}/gimmicks/{gimmick_id}
#[utoipa::path(
    delete,
    path = "/admin/v1/events/matches/{match_id}/gimmicks/{gimmick_id}",
    operation_id = "admin_delete_gimmick",
    tag = "Admin - Gimmicks",
    security(("bearer_jwt" = [])),
    params(
        ("match_id" = String, Path, description = "Match UUID"),
        ("gimmick_id" = String, Path, description = "Gimmick UUID")
    ),
    responses(
        (status = 200, description = "Gimmick deleted", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[delete("/matches/{match_id}/gimmicks/{gimmick_id}")]
pub async fn delete_gimmick(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (match_id, gimmick_id) = path.into_inner();

    match db.web_db.delete_gimmick(match_id, gimmick_id).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Gimmick deleted"
        })),
        Err(e) => map_event_error(e),
    }
}
