use std::sync::Arc;

use actix_web::{get, patch, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::Local;
use ran_models::events::event::{CreateEventRequest, UpdateEventRequest};
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::{get_admin_id, map_event_error};

/// POST /admin/events
#[utoipa::path(
    post,
    path = "/admin/v1/events",
    operation_id = "admin_create_event",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created", body = serde_json::Value),
        (status = 400, description = "Validation error or legacy category"),
        (status = 401, description = "Not authenticated"),
        (status = 409, description = "Slug already exists"),
        (status = 500, description = "Database error")
    )
)]
#[post("")]
pub async fn create_event(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    body: web::Json<CreateEventRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let req_body = body.into_inner();

    if let Some(cat) = req_body.event_category.as_ref() {
        if cat.is_legacy() {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "code": "LEGACY_CATEGORY",
                "message": "GvGKoth is a legacy category and cannot be used for new events"
            }));
        }
    }

    match db.web_db.create_event(&req_body).await {
        Ok(event) => HttpResponse::Created().json(json!({
            "success": true,
            "event": event
        })),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("Violation of UNIQUE KEY constraint") || err_str.contains("2627") {
                return HttpResponse::Conflict().json(json!({
                    "success": false, "code": "CONFLICT", "message": "An event with this slug already exists"
                }));
            }
            map_event_error(e)
        }
    }
}

/// PUT /admin/events/{event_id}
#[utoipa::path(
    put,
    path = "/admin/v1/events/{event_id}",
    operation_id = "admin_update_event",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Event updated", body = serde_json::Value),
        (status = 400, description = "Validation error, locked, or legacy category"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 409, description = "Slug already exists"),
        (status = 500, description = "Database error")
    )
)]
#[put("/{event_id}")]
pub async fn update_event(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateEventRequest>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();
    let req_body = body.into_inner();

    if let Some(cat) = req_body.event_category.as_ref() {
        if cat.is_legacy() {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "code": "LEGACY_CATEGORY",
                "message": "Cannot re-categorize an event to a legacy category"
            }));
        }
    }

    if req_body.event_slug.is_some() {
        match db.web_db.get_event_by_id(event_id).await {
            Ok(Some(event)) => {
                let now = Local::now().naive_local();
                if now >= event.event_start {
                    return HttpResponse::BadRequest().json(json!({
                        "success": false, "code": "LOCKED", "message": "Cannot change slug after event is live"
                    }));
                }
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(json!({
                    "success": false, "code": "NOT_FOUND", "message": "Event not found"
                }));
            }
            Err(e) => return map_event_error(e),
        }
    }

    match db.web_db.update_event(event_id, &req_body).await {
        Ok(event) => HttpResponse::Ok().json(json!({
            "success": true,
            "event": event
        })),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("Violation of UNIQUE KEY constraint") || err_str.contains("2627") {
                return HttpResponse::Conflict().json(json!({
                    "success": false, "code": "CONFLICT", "message": "An event with this slug already exists"
                }));
            }
            map_event_error(e)
        }
    }
}

/// PATCH /admin/events/{event_id}/activate
#[utoipa::path(
    patch,
    path = "/admin/v1/events/{event_id}/activate",
    operation_id = "admin_activate_event",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "Event activated", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[patch("/{event_id}/activate")]
pub async fn activate_event(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    match db.web_db.activate_event(path.into_inner()).await {
        Ok(event) => HttpResponse::Ok().json(json!({
            "success": true,
            "event": event
        })),
        Err(e) => map_event_error(e),
    }
}

/// PATCH /admin/events/{event_id}/deactivate
#[utoipa::path(
    patch,
    path = "/admin/v1/events/{event_id}/deactivate",
    operation_id = "admin_deactivate_event",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "Event deactivated", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[patch("/{event_id}/deactivate")]
pub async fn deactivate_event(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    match db.web_db.deactivate_event(path.into_inner()).await {
        Ok(event) => HttpResponse::Ok().json(json!({
            "success": true,
            "event": event
        })),
        Err(e) => map_event_error(e),
    }
}

/// GET /admin/events
#[utoipa::path(
    get,
    path = "/admin/v1/events",
    operation_id = "admin_list_events",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    responses(
        (status = 200, description = "List of all events", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("")]
pub async fn list_events(db: web::Data<Arc<DBState>>) -> impl Responder {
    match db.web_db.list_events().await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "success": true,
            "events": events
        })),
        Err(e) => map_event_error(e),
    }
}

/// GET /admin/events/{event_id}
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}",
    operation_id = "admin_get_event_detail",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "Event detail with matches", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}")]
pub async fn get_event_detail(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
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
        Err(e) => return map_event_error(e),
    };

    let matches = match db.web_db.list_matches_admin(event_id).await {
        Ok(m) => m,
        Err(e) => return map_event_error(e),
    };

    HttpResponse::Ok().json(json!({
        "success": true,
        "event": event,
        "matches": matches
    }))
}

/// GET /admin/events/{event_id}/dashboard
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/dashboard",
    operation_id = "admin_get_event_dashboard",
    tag = "Admin - Events",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "Event dashboard with statistics", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/dashboard")]
pub async fn get_event_dashboard(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match db.web_db.get_event_dashboard(path.into_inner()).await {
        Ok(dashboard) => HttpResponse::Ok().json(json!({
            "success": true,
            "event": dashboard.event,
            "registration_stats": dashboard.registration_stats,
            "match_stats": dashboard.match_stats,
            "deduction_stats": dashboard.deduction_stats
        })),
        Err(e) => map_event_error(e),
    }
}
