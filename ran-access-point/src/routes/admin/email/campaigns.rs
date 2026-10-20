use std::sync::Arc;

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::webmanager::email::campaigns::WebEmailCampaignTrait;
use ran_models::{
    email::{CampaignFilterParams, CreateEmailCampaignRequest},
    errors::flatten_validation_errors,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{database::DBState, services::jwt::Claims};

#[post("")]
async fn create_campaign(
    db: web::Data<Arc<DBState>>,
    req: HttpRequest,
    body: web::Json<CreateEmailCampaignRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"})),
    };

    let data = body.into_inner();
    if let Err(validation_errors) = data.validate() {
        return HttpResponse::BadRequest().json(flatten_validation_errors(&validation_errors));
    }

    // segment_type must be 0, 1, or 2
    if ![0i16, 1, 2].contains(&data.segment_type) {
        return HttpResponse::BadRequest()
            .json(json!({"error": "segment_type must be 0 (All), 1 (Verified), or 2 (Online within N days)"}));
    }

    // segment_param required when segment_type = 2
    if data.segment_type == 2 && data.segment_param.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "segment_param (days) is required for segment_type 2"}));
    }

    match db.web_db.create_email_campaign(data, &claims.sub).await {
        Ok(campaign) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": campaign
        })),
        Err(e) => {
            tracing::error!("Failed to create campaign: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to create campaign"}))
        }
    }
}

#[get("")]
async fn get_campaigns(
    db: web::Data<Arc<DBState>>,
    query: web::Query<CampaignFilterParams>,
) -> impl Responder {
    let params = query.into_inner();
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    match db.web_db.get_email_campaigns(params.status, page, limit).await {
        Ok(result) => HttpResponse::Ok().json(json!({
            "data": result.data,
            "page": page,
            "limit": limit,
            "total_items": result.total_records
        })),
        Err(e) => {
            tracing::error!("Failed to fetch campaigns: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaigns"}))
        }
    }
}

#[get("/{id}")]
async fn get_campaign_by_id(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let campaign_id = path.into_inner();

    match db.web_db.get_email_campaign_by_id(campaign_id).await {
        Ok(Some(campaign)) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": campaign
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Campaign not found"})),
        Err(e) => {
            tracing::error!("Failed to fetch campaign: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaign"}))
        }
    }
}

/// Snapshots recipients and transitions campaign to Queued.
/// Returns immediately — background worker handles sending.
#[post("/{id}/queue")]
async fn queue_campaign(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let campaign_id = path.into_inner();

    let campaign = match db.web_db.get_email_campaign_by_id(campaign_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Campaign not found"})),
        Err(e) => {
            tracing::error!("Failed to fetch campaign for queue: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaign"}));
        }
    };

    // Only Draft (0) or Scheduled (1) campaigns can be queued
    if ![0i16, 1].contains(&campaign.status) {
        return HttpResponse::BadRequest().json(json!({
            "error": format!(
                "Cannot queue campaign with status {}. Only Draft or Scheduled campaigns can be queued.",
                campaign.status
            )
        }));
    }

    // Snapshot recipients
    let recipient_count = match db
        .web_db
        .populate_campaign_recipients(campaign_id, campaign.segment_type, campaign.segment_param)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to populate recipients: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to populate recipients"}));
        }
    };

    if recipient_count == 0 {
        return HttpResponse::BadRequest().json(json!({
            "error": "No recipients found for the selected segment. Campaign not queued."
        }));
    }

    // Transition to Queued (2)
    if let Err(e) = db
        .web_db
        .update_campaign_status(campaign_id, 2, Some(recipient_count), None, None)
        .await
    {
        tracing::error!("Failed to set campaign to Queued: {:?}", e);
        return HttpResponse::InternalServerError()
            .json(json!({"error": "Failed to queue campaign"}));
    }

    tracing::info!(
        "Campaign '{}' ({}) queued with {} recipients",
        campaign.campaign_name,
        campaign_id,
        recipient_count
    );

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Campaign queued with {} recipients. Sending will begin shortly.", recipient_count),
        "total_recipients": recipient_count
    }))
}

/// Cancel a Scheduled (1) or Queued (2) campaign.
#[post("/{id}/cancel")]
async fn cancel_campaign(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let campaign_id = path.into_inner();

    let campaign = match db.web_db.get_email_campaign_by_id(campaign_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Campaign not found"})),
        Err(e) => {
            tracing::error!("Failed to fetch campaign for cancel: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaign"}));
        }
    };

    if ![0i16, 1, 2].contains(&campaign.status) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Only Draft, Scheduled, or Queued campaigns can be cancelled."
        }));
    }

    if let Err(e) = db
        .web_db
        .update_campaign_status(campaign_id, 6, None, None, None)
        .await
    {
        tracing::error!("Failed to cancel campaign: {:?}", e);
        return HttpResponse::InternalServerError()
            .json(json!({"error": "Failed to cancel campaign"}));
    }

    HttpResponse::Ok().json(json!({"success": true, "message": "Campaign cancelled"}))
}

/// Resets a Failed campaign back to Queued so the worker picks it up again.
/// Permanently-failed recipients are reset to Pending with one attempt remaining.
/// Already-sent recipients are untouched.
#[post("/{id}/retry")]
async fn retry_campaign(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let campaign_id = path.into_inner();

    let campaign = match db.web_db.get_email_campaign_by_id(campaign_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Campaign not found"})),
        Err(e) => {
            tracing::error!("Failed to fetch campaign for retry: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaign"}));
        }
    };

    if campaign.status != 5 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Only Failed campaigns can be retried."
        }));
    }

    match db.web_db.retry_campaign(campaign_id).await {
        Ok(reset_count) => {
            tracing::info!(
                "Campaign '{}' ({}) retried — {} failed recipients reset to pending",
                campaign.campaign_name,
                campaign_id,
                reset_count
            );
            HttpResponse::Ok().json(json!({
                "success": true,
                "reset_recipients": reset_count
            }))
        }
        Err(e) => {
            tracing::error!("Failed to retry campaign {}: {:?}", campaign_id, e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to retry campaign"}))
        }
    }
}

/// HTTP fallback for campaign progress (for non-WebSocket clients).
#[get("/{id}/progress")]
async fn get_campaign_progress(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let campaign_id = path.into_inner();

    match db.web_db.get_campaign_progress(campaign_id).await {
        Ok(Some(progress)) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": progress
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Campaign not found"})),
        Err(e) => {
            tracing::error!("Failed to fetch campaign progress: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch campaign progress"}))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_campaign)
        .service(get_campaigns)
        .service(get_campaign_by_id)
        .service(queue_campaign)
        .service(cancel_campaign)
        .service(retry_campaign)
        .service(get_campaign_progress);
}
