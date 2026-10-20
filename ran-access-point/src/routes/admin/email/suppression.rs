use std::{collections::HashSet, sync::Arc};

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use ran_db_core::webmanager::email::suppression::WebEmailSuppressionTrait;
use ran_models::{
    email::{RemoveSuppressionRequest, SuppressionListParams},
    errors::flatten_validation_errors,
};
use serde_json::json;
use tracing::error;
use validator::Validate;

use crate::{database::DBState, states::AppState};

/// GET /admin/v1/email/suppression — List suppression entries with pagination and optional filtering
#[get("")]
async fn get_suppression_list(
    db: web::Data<Arc<DBState>>,
    query: web::Query<SuppressionListParams>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let reason = query.reason.as_deref();

    match db.web_db.get_suppression_list(page, limit, reason).await {
        Ok(result) => HttpResponse::Ok().json(json!({
            "data": result.data,
            "total_records": result.total_records,
            "page": page,
            "limit": limit,
        })),
        Err(e) => {
            error!("Failed to get suppression list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"message": "Failed to get suppression list"}))
        }
    }
}

/// DELETE /admin/v1/email/suppression — Remove an email address from the suppression list
#[delete("")]
async fn remove_suppression(
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
    body: web::Json<RemoveSuppressionRequest>,
) -> impl Responder {
    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(flatten_validation_errors(&validation_errors));
    }

    let email = body.email.to_lowercase();

    match db.web_db.remove_email_suppression(&email, "admin").await {
        Ok(()) => {
            // Update cache
            let mut cache = states.suppression_cache.write().await;
            cache.remove(&email);

            HttpResponse::Ok().json(json!({"message": "Email removed from suppression list"}))
        }
        Err(e) => {
            error!("Failed to remove email suppression: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"message": "Failed to remove suppression"}))
        }
    }
}

/// POST /admin/v1/email/suppression/seed — Seed suppression list from email delivery failures (one-time operation)
#[post("/seed")]
async fn seed_suppression(
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    match db.web_db.seed_suppression_from_failures().await {
        Ok(count) => {
            // Refresh cache after seeding
            if let Ok(emails) = db.web_db.get_all_suppressed_emails().await {
                let set: HashSet<String> = emails.into_iter().collect();
                *states.suppression_cache.write().await = set;
            }

            HttpResponse::Ok().json(json!({
                "message": format!("{} addresses seeded into suppression list", count),
                "seeded_count": count,
            }))
        }
        Err(e) => {
            error!("Failed to seed suppression from failures: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"message": "Failed to seed suppression list"}))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_suppression_list)
        .service(remove_suppression)
        .service(seed_suppression);
}
