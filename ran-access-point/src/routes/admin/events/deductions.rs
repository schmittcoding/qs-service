use std::sync::Arc;

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::deduction::DeductionRequest;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::get_admin_id;

#[derive(Deserialize)]
struct DeductionListQuery {
    target_type: Option<String>,
}

/// GET /admin/events/{event_id}/deductions
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/deductions",
    operation_id = "admin_list_deductions",
    tag = "Admin - Deductions",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("target_type" = Option<String>, Query, description = "Filter by GUILD, PLAYER, or SCHOOL")
    ),
    responses(
        (status = 200, description = "List of deductions", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/deductions")]
pub async fn list_deductions(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    query: web::Query<DeductionListQuery>,
) -> impl Responder {
    let event_id = path.into_inner();

    match db.web_db.list_deductions(event_id, query.target_type.as_deref()).await {
        Ok(deductions) => HttpResponse::Ok().json(json!({
            "success": true,
            "deductions": deductions
        })),
        Err(e) => {
            tracing::error!("Failed to list deductions: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to list deductions"
            }))
        }
    }
}

/// POST /admin/events/{event_id}/deductions
#[utoipa::path(
    post,
    path = "/admin/v1/events/{event_id}/deductions",
    operation_id = "admin_apply_deduction",
    tag = "Admin - Deductions",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    request_body = DeductionRequest,
    responses(
        (status = 200, description = "Deduction applied", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Database error")
    )
)]
#[post("/{event_id}/deductions")]
pub async fn apply_deduction(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<DeductionRequest>,
) -> impl Responder {
    let admin_id = match get_admin_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let event_id = path.into_inner();
    let deduction = body.into_inner();

    if !["GUILD", "PLAYER", "SCHOOL"].contains(&deduction.target_type.as_str()) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "VALIDATION_FAILED",
            "message": "target_type must be GUILD, PLAYER, or SCHOOL"
        }));
    }

    match deduction.target_type.as_str() {
        "GUILD" | "PLAYER" => {
            if deduction.target_id.is_none() {
                return HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "VALIDATION_FAILED",
                    "message": "target_id is required for GUILD/PLAYER deductions"
                }));
            }
        }
        "SCHOOL" => {
            if deduction.target_cha_school.is_none() {
                return HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "VALIDATION_FAILED",
                    "message": "target_cha_school is required for SCHOOL deductions"
                }));
            }
        }
        _ => {}
    }

    if deduction.points <= 0 {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "VALIDATION_FAILED",
            "message": "points must be a positive integer"
        }));
    }

    if deduction.reason.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": "VALIDATION_FAILED",
            "message": "reason is required"
        }));
    }

    match db
        .web_db
        .apply_deduction(
            event_id,
            &deduction.target_type,
            deduction.target_id,
            deduction.target_cha_school,
            deduction.points,
            &deduction.reason,
            admin_id.to_string(),
            deduction.match_id,
        )
        .await
    {
        Ok(result) => HttpResponse::Ok().json(json!({
            "success": true,
            "deduction": result
        })),
        Err(e) => {
            tracing::error!("Failed to apply deduction: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": "DB_ERROR",
                "message": "Failed to apply deduction"
            }))
        }
    }
}

/// PUT /admin/events/{event_id}/deductions/{deduction_id}/revert
#[utoipa::path(
    put,
    path = "/admin/v1/events/{event_id}/deductions/{deduction_id}/revert",
    operation_id = "admin_revert_deduction",
    tag = "Admin - Deductions",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("deduction_id" = String, Path, description = "Deduction UUID")
    ),
    responses(
        (status = 200, description = "Deduction reverted", body = serde_json::Value),
        (status = 400, description = "Deduction already reverted"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Deduction not found"),
        (status = 500, description = "Database error")
    )
)]
#[put("/{event_id}/deductions/{deduction_id}/revert")]
pub async fn revert_deduction(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let admin_id = match get_admin_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let (_event_id, deduction_id) = path.into_inner();

    match db.web_db.revert_deduction(deduction_id, admin_id.to_string()).await {
        Ok(result) => HttpResponse::Ok().json(json!({
            "success": true,
            "deduction": result
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "code": "NOT_FOUND",
                    "message": "Deduction not found"
                }))
            } else if msg.contains("already reverted") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": "ALREADY_REVERTED",
                    "message": "Deduction is already reverted"
                }))
            } else {
                tracing::error!("Failed to revert deduction: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "code": "DB_ERROR",
                    "message": "Failed to revert deduction"
                }))
            }
        }
    }
}
