use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::category::EventCategory;
use ran_models::events::event::{
    CreateMatchRequest, CreateMatchesRequest, GenerateMatchesRequest, UpdateMatchRequest,
};
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::{get_admin_id, map_event_error};

/// POST /admin/events/{event_id}/matches — single or bulk
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/matches",
    operation_id = "admin_create_matches",
    tag = "Admin - Match Management",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 201, description = "Match(es) created", body = serde_json::Value),
        (status = 400, description = "Validation error or unsupported event category"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/matches")]
pub async fn create_matches(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
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
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to fetch event"
            }));
        }
    };

    if !matches!(event.event_category, EventCategory::Gvg | EventCategory::Koth | EventCategory::GvgKoth) {
        tracing::warn!("Rejected match creation for event {} with category {:?}", event_id, event.event_category);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Matches are only supported for GvG, KOTH, and GvG+KOTH events"
        }));
    }

    let value = body.into_inner();

    if value.get("matches").is_some() {
        let bulk_req: CreateMatchesRequest = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(e) => {
                return HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "VALIDATION_FAILED",
                    "message": format!("Invalid request body: {}", e)
                }))
            }
        };

        match db.web_db.create_matches_bulk(event_id, &bulk_req.matches).await {
            Ok(matches) => HttpResponse::Created().json(json!({
                "success": true,
                "created": matches.len(),
                "matches": matches
            })),
            Err(e) => map_event_error(e),
        }
    } else {
        let single_req: CreateMatchRequest = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(e) => {
                return HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "VALIDATION_FAILED",
                    "message": format!("Invalid request body: {}", e)
                }))
            }
        };

        match db.web_db.create_match(event_id, &single_req).await {
            Ok(m) => HttpResponse::Created().json(json!({
                "success": true,
                "created": 1,
                "matches": [m]
            })),
            Err(e) => map_event_error(e),
        }
    }
}

/// POST /admin/events/{event_id}/matches/generate
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/matches/generate",
    operation_id = "admin_generate_matches",
    tag = "Admin - Match Management",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    request_body = GenerateMatchesRequest,
    responses(
        (status = 201, description = "Matches generated", body = serde_json::Value),
        (status = 400, description = "Unsupported event category"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/matches/generate")]
pub async fn generate_matches(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<GenerateMatchesRequest>,
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
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to fetch event"
            }));
        }
    };

    if !matches!(event.event_category, EventCategory::Gvg | EventCategory::Koth | EventCategory::GvgKoth) {
        tracing::warn!("Rejected match generation for event {} with category {:?}", event_id, event.event_category);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Matches are only supported for GvG, KOTH, and GvG+KOTH events"
        }));
    }

    match db.web_db.generate_matches(event_id, &body.into_inner()).await {
        Ok(matches) => HttpResponse::Created().json(json!({
            "success": true,
            "created": matches.len(),
            "matches": matches
        })),
        Err(e) => map_event_error(e),
    }
}

/// GET /admin/events/{event_id}/matches
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/matches",
    operation_id = "admin_list_matches",
    tag = "Admin - Match Management",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "List of matches for event", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/matches")]
pub async fn list_matches(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match db.web_db.list_matches_admin(path.into_inner()).await {
        Ok(matches) => HttpResponse::Ok().json(json!({
            "success": true,
            "matches": matches
        })),
        Err(e) => map_event_error(e),
    }
}

/// PUT /admin/events/matches/{match_id}
#[utoipa::path(
    put,
    path = "/admin/v1/events/matches/{match_id}",
    operation_id = "admin_update_match",
    tag = "Admin - Match Management",
    security(("bearer_jwt" = [])),
    params(
        ("match_id" = String, Path, description = "Match UUID")
    ),
    request_body = UpdateMatchRequest,
    responses(
        (status = 200, description = "Match updated", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[put("/matches/{match_id}")]
pub async fn update_match(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateMatchRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    match db.web_db.update_match(path.into_inner(), &body.into_inner()).await {
        Ok(m) => HttpResponse::Ok().json(json!({
            "success": true,
            "match": m
        })),
        Err(e) => map_event_error(e),
    }
}

/// DELETE /admin/events/matches/{match_id}
#[utoipa::path(
    delete,
    path = "/admin/v1/events/matches/{match_id}",
    operation_id = "admin_delete_match",
    tag = "Admin - Match Management",
    security(("bearer_jwt" = [])),
    params(
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "Match deleted", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[delete("/matches/{match_id}")]
pub async fn delete_match(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    match db.web_db.delete_match(path.into_inner()).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Match deleted"
        })),
        Err(e) => map_event_error(e),
    }
}
