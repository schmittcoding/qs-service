use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::DBState;

#[derive(Deserialize)]
pub struct SnapshotKothQuery {
    pub cha_class: Option<i32>,
}

/// GET /events/{event_slug}/snapshots — list all snapshots with their match schedules
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/snapshots",
    tag = "Events - Snapshots",
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "List of snapshots with match schedule metadata", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/snapshots")]
pub async fn list_snapshots(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
) -> impl Responder {
    let event_slug = path.into_inner();

    let event = match db.web_db.get_event_by_slug(&event_slug).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch event"
            }));
        }
    };
    let event_id = event.event_id;

    match db.web_db.get_snapshots(event_id).await {
        Ok(snapshots) => HttpResponse::Ok().json(json!({
            "success": true,
            "snapshots": snapshots
        })),
        Err(e) => {
            log::error!("Failed to list snapshots: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to list snapshots"
            }))
        }
    }
}

/// GET /events/{event_slug}/snapshots/{snapshot_id}/guild — frozen guild standings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/snapshots/{snapshot_id}/guild",
    tag = "Events - Snapshots",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("snapshot_id" = String, Path, description = "Snapshot UUID")
    ),
    responses(
        (status = 200, description = "Frozen guild standings", body = serde_json::Value),
        (status = 400, description = "Invalid snapshot ID or event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/snapshots/{snapshot_id}/guild")]
pub async fn snapshot_guild_scores(
    db: web::Data<Arc<DBState>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (event_slug, snapshot_id_str) = path.into_inner();

    // Validate event exists
    if db.web_db.get_event_by_slug(&event_slug).await.ok().flatten().is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
        }));
    }
    let snapshot_id = match Uuid::parse_str(&snapshot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Invalid snapshot ID"
            }));
        }
    };

    match db.web_db.get_snapshot_guild_scores(snapshot_id).await {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch snapshot guild scores: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch snapshot data"
            }))
        }
    }
}

/// GET /events/{event_slug}/snapshots/{snapshot_id}/koth — frozen KOTH standings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/snapshots/{snapshot_id}/koth",
    tag = "Events - Snapshots",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("snapshot_id" = String, Path, description = "Snapshot UUID"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "Frozen KOTH standings", body = serde_json::Value),
        (status = 400, description = "Invalid snapshot ID or event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/snapshots/{snapshot_id}/koth")]
pub async fn snapshot_koth_scores(
    db: web::Data<Arc<DBState>>,
    path: web::Path<(String, String)>,
    params: web::Query<SnapshotKothQuery>,
) -> impl Responder {
    let (event_slug, snapshot_id_str) = path.into_inner();

    // Validate event exists
    if db.web_db.get_event_by_slug(&event_slug).await.ok().flatten().is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
        }));
    }
    let snapshot_id = match Uuid::parse_str(&snapshot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Invalid snapshot ID"
            }));
        }
    };

    match db
        .web_db
        .get_snapshot_koth_scores(snapshot_id, params.cha_class)
        .await
    {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "cha_class": params.cha_class,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch snapshot KOTH scores: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch snapshot data"
            }))
        }
    }
}

/// GET /events/{event_slug}/snapshots/{snapshot_id}/level-cap-race — frozen LevelCapRace standings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/snapshots/{snapshot_id}/level-cap-race",
    tag = "Events - Snapshots",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("snapshot_id" = String, Path, description = "Snapshot UUID")
    ),
    responses(
        (status = 200, description = "Frozen level cap race standings", body = serde_json::Value),
        (status = 400, description = "Invalid snapshot ID or event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/snapshots/{snapshot_id}/level-cap-race")]
pub async fn snapshot_level_cap_race_scores(
    db: web::Data<Arc<DBState>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (event_slug, snapshot_id_str) = path.into_inner();

    // Validate event exists
    if db.web_db.get_event_by_slug(&event_slug).await.ok().flatten().is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
        }));
    }
    let snapshot_id = match Uuid::parse_str(&snapshot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Invalid snapshot ID"
            }));
        }
    };

    match db.web_db.get_snapshot_level_cap_race_scores(snapshot_id).await {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch snapshot LevelCapRace scores: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch snapshot data"
            }))
        }
    }
}

/// GET /events/{event_slug}/snapshots/{snapshot_id}/school — frozen school standings
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/snapshots/{snapshot_id}/school",
    tag = "Events - Snapshots",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("snapshot_id" = String, Path, description = "Snapshot UUID")
    ),
    responses(
        (status = 200, description = "Frozen school standings", body = serde_json::Value),
        (status = 400, description = "Invalid snapshot ID or event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/snapshots/{snapshot_id}/school")]
pub async fn snapshot_school_scores(
    db: web::Data<Arc<DBState>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (event_slug, snapshot_id_str) = path.into_inner();

    // Validate event exists
    if db.web_db.get_event_by_slug(&event_slug).await.ok().flatten().is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED", "message": "Event not found"
        }));
    }
    let snapshot_id = match Uuid::parse_str(&snapshot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "VALIDATION_FAILED", "message": "Invalid snapshot ID"
            }));
        }
    };

    match db.web_db.get_snapshot_school_scores(snapshot_id).await {
        Ok(scores) => HttpResponse::Ok().json(json!({
            "success": true,
            "rankings": scores
        })),
        Err(e) => {
            log::error!("Failed to fetch snapshot school scores: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch snapshot data"
            }))
        }
    }
}
