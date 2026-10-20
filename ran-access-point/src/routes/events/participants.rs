use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::database::DBState;

#[derive(Deserialize)]
pub struct ParticipantQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub cha_class: Option<i32>,
}

/// GET /events/{event_slug}/participants/characters — public character participant list
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/participants/characters",
    tag = "Events - Participants",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("limit" = Option<u32>, Query, description = "Items per page (default 20, max 100)"),
        ("cha_class" = Option<i32>, Query, description = "Character class filter")
    ),
    responses(
        (status = 200, description = "List of character participants", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/participants/characters")]
pub async fn participant_characters(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
    params: web::Query<ParticipantQuery>,
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

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.limit.unwrap_or(20).min(100);

    match db
        .web_db
        .get_public_participants_by_character(event_id, params.cha_class, page, page_size)
        .await
    {
        Ok((rows, total)) => {
            let participants: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "rank": r.get::<i64, _>("Rank").unwrap_or(0),
                        "cha_num": r.get::<i32, _>("ChaNum").unwrap_or_default(),
                        "cha_name": r.get::<&str, _>("ChaName").unwrap_or(""),
                        "cha_class": r.get::<i32, _>("ChaClass").unwrap_or_default(),
                        "cha_school": r.get::<i32, _>("ChaSchool"),
                        "cha_level": r.get::<i32, _>("ChaLevel").unwrap_or_default(),
                        "guild_num": r.get::<i32, _>("GuildNum"),
                        "guild_name": r.get::<&str, _>("GuildName"),
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total_items": total
                },
                "cha_class": params.cha_class,
                "participants": participants
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch character participants: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch participants"
            }))
        }
    }
}

/// GET /events/{event_slug}/participants/guilds — public guild participant list
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/participants/guilds",
    tag = "Events - Participants",
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("limit" = Option<u32>, Query, description = "Items per page (default 20, max 100)")
    ),
    responses(
        (status = 200, description = "List of guild participants", body = serde_json::Value),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/participants/guilds")]
pub async fn participant_guilds(
    db: web::Data<Arc<DBState>>,
    path: web::Path<String>,
    params: web::Query<ParticipantQuery>,
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

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.limit.unwrap_or(20).min(100);

    match db
        .web_db
        .get_public_participants_by_guild(event_id, page, page_size)
        .await
    {
        Ok((rows, total)) => {
            let guilds: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "rank": r.get::<i64, _>("Rank").unwrap_or(0),
                        "guild_num": r.get::<i32, _>("GuildNum").unwrap_or_default(),
                        "guild_name": r.get::<&str, _>("GuildName"),
                        "cha_school": r.get::<i32, _>("ChaSchool"),
                        "member_count": r.get::<i32, _>("MemberCount").unwrap_or(0),
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total_items": total
                },
                "participants": guilds
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch guild participants: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch participants"
            }))
        }
    }
}
