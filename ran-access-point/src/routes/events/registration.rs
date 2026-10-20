use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use ran_db_core::gamemanager::admin::characters::GameCharacterTrait;
use ran_models::events::category::EventCategory;
use ran_models::events::ref_data::map_game_class_to_event_class;
use ran_models::events::registration::{GvgRegisterRequest, RegisterRequest};
use serde_json::json;
use std::sync::Arc;

use crate::{database::DBState, middleware::session::get_user_session};

/// GET /events/{event_slug}/characters — eligible characters for this user
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/characters",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "List of eligible characters", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/characters")]
pub async fn get_eligible_characters(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<String>,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let event_slug = path.into_inner();

    // V1: Event exists
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

    let user_num = profile.user_num as i32;

    // Fetch all characters for this user
    match db.game_db.get_characters_by_user(1, 1000, user_num).await {
        Ok(res) => {
            let characters: Vec<serde_json::Value> = res
                .data
                .iter()
                .map(|row| {
                    let cha_num = row.get::<i32, _>("ChaNum").unwrap_or_default();
                    let cha_name = row
                        .get::<&str, _>("ChaName")
                        .map_or_else(String::new, |s| s.to_string());
                    let raw_cha_class = row.get::<i32, _>("ChaClass").unwrap_or_default();
                    let event_class = map_game_class_to_event_class(raw_cha_class).unwrap_or(0);
                    let cha_school = row.get::<i32, _>("ChaSchool").unwrap_or_default();
                    let cha_level = row.get::<i32, _>("ChaLevel").unwrap_or(1);
                    let guild_num = row.get::<i32, _>("GuNum").unwrap_or(0);
                    let guild_name: Option<&str> = row.get("GuName");
                    let guild_leader_name: Option<&str> = row.get("GuildLeaderChaName");
                    let guild_leader_school: Option<i32> = row.get("GuildLeaderChaSchool");
                    let deleted = row.get::<i32, _>("ChaDeleted").unwrap_or(0);

                    let meets_level = cha_level >= event.min_level;
                    let has_guild = guild_num > 0;

                    let eligible = match event.event_category {
                        EventCategory::Gvg | EventCategory::GvgKoth => meets_level && has_guild && deleted == 0,
                        _ => meets_level && deleted == 0,
                    };

                    json!({
                        "cha_num": cha_num,
                        "cha_name": cha_name,
                        "cha_class": event_class,
                        "cha_class_name": ran_models::events::ref_data::event_class_name(event_class),
                        "cha_school": cha_school,
                        "cha_level": cha_level,
                        "guild_num": guild_num,
                        "guild_name": guild_name,
                        "guild_leader_name": guild_leader_name,
                        "guild_leader_school": guild_leader_school,
                        "eligible": eligible,
                    })
                })
                .filter(|c| c["cha_level"].as_i64().unwrap_or(0) >= 1)
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "event_id": event_id,
                "min_level": event.min_level,
                "registration_open": event.registration_open,
                "registration_close": event.registration_close,
                "characters": characters
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch characters: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch characters"
            }))
        }
    }
}

/// POST /events/{event_slug}/register — register a character
#[utoipa::path(
    post,
    path = "/api/v1/events/{event_slug}/register",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = serde_json::Value),
        (status = 400, description = "Validation error or registration not open"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event or character not found"),
        (status = 409, description = "Character already registered"),
        (status = 500, description = "Database error")
    )
)]
#[post("/register")]
pub async fn register_character(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<String>,
    body: web::Json<RegisterRequest>,
    req: HttpRequest,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let event_slug = path.into_inner();

    let user_num = profile.user_num as i32;
    let req_body = body.into_inner();

    // V1: Event exists and registration window is open
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

    let now = chrono::Local::now().naive_local();
    if !event.is_registration_open(now) {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "CLOSED", "message": "Registration is no longer open."
        }));
    }

    // V2: Character belongs to this user
    let char_info = match db.game_db.get_character_by_id(req_body.cha_num).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Character not found"
            }));
        }
    };

    if char_info.user_num != user_num {
        return HttpResponse::Forbidden().json(json!({
            "success": false, "code": "UNAUTHORIZED",
            "message": "This character does not belong to your account."
        }));
    }

    // V3: Level requirement
    if char_info.cha_level < event.min_level {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "LEVEL_REQ",
            "message": format!("Character must be at least level {}.", event.min_level)
        }));
    }

    // V4: If GvG event, must have a guild
    if matches!(event.event_category, EventCategory::Gvg | EventCategory::GvgKoth) && char_info.cha_guild_num <= 0 {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "NO_GUILD",
            "message": "You must be in a guild to join GvG."
        }));
    }

    // V5: No duplicate registration
    match db
        .web_db
        .get_registration_by_event_and_char(event_id, req_body.cha_num)
        .await
    {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(json!({
                "success": false, "code": "DUPLICATE",
                "message": "This character is already registered for this event."
            }));
        }
        Err(e) => {
            log::error!("Failed to check duplicate: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Database error"
            }));
        }
        Ok(None) => {}
    }

    // V6: Account limit (max 3 characters per account)
    match db
        .web_db
        .count_active_registrations(event_id, user_num)
        .await
    {
        Ok(count) if count >= 3 => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "ACCT_LIMIT",
                "message": "You can register at most 3 characters."
            }));
        }
        Err(e) => {
            log::error!("Failed to count registrations: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Database error"
            }));
        }
        _ => {}
    }

    // V10: If GvG event, guild leader's school must match the registree's school
    if matches!(event.event_category, EventCategory::Gvg | EventCategory::GvgKoth) && char_info.cha_guild_num > 0 {
        match db
            .game_db
            .get_guild_leader_school(char_info.cha_guild_num)
            .await
        {
            Ok(Some((leader_school, leader_name))) if leader_school != char_info.cha_school => {
                let leader_school_name =
                    ran_models::events::ref_data::school_name(leader_school);
                let char_school_name =
                    ran_models::events::ref_data::school_name(char_info.cha_school);
                return HttpResponse::BadRequest().json(json!({
                    "success": false, "code": "LEADER_SCHOOL_MISMATCH",
                    "message": format!(
                        "Your guild leader ({}) is from {}. Your character is in {}. Guild leader and member must be in the same school for GvG.",
                        leader_name, leader_school_name, char_school_name
                    )
                }));
            }
            Err(e) => {
                log::error!("Failed to check guild leader school: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false, "code": "DB_ERROR", "message": "Database error"
                }));
            }
            _ => {}
        }
    }

    // V9: If GvG event, all guild members must be in the same school
    if matches!(event.event_category, EventCategory::Gvg | EventCategory::GvgKoth) && char_info.cha_guild_num > 0 {
        match db
            .web_db
            .get_guild_registered_school(event_id, char_info.cha_guild_num)
            .await
        {
            Ok(Some(existing_school)) if existing_school != char_info.cha_school => {
                let existing_name =
                    ran_models::events::ref_data::school_name(existing_school);
                let char_school_name =
                    ran_models::events::ref_data::school_name(char_info.cha_school);
                return HttpResponse::BadRequest().json(json!({
                    "success": false, "code": "SCHOOL_MISMATCH",
                    "message": format!(
                        "Your guild already has members registered from {}. All guild members must be in the same school for GvG. Your character is in {}.",
                        existing_name, char_school_name
                    )
                }));
            }
            Err(e) => {
                log::error!("Failed to check guild school: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false, "code": "DB_ERROR", "message": "Database error"
                }));
            }
            _ => {}
        }
    }

    // Guild is locked at registration time
    let guild_num = if char_info.cha_guild_num > 0 {
        Some(char_info.cha_guild_num)
    } else {
        None
    };

    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    // Map raw game class to simplified event class
    let event_class = match map_game_class_to_event_class(char_info.cha_class) {
        Some(c) => c,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "success": false, "code": "VALIDATION_FAILED",
                "message": format!("Unrecognized character class: {}", char_info.cha_class)
            }));
        }
    };

    // Insert registration
    match db
        .web_db
        .insert_event_registration(
            event_id,
            req_body.cha_num,
            user_num,
            guild_num,
            Some(char_info.cha_school),
            event_class,
            char_info.cha_level,
            ip_address,
        )
        .await
    {
        Ok(reg) => HttpResponse::Created().json(json!({
            "success": true,
            "message": "Registration successful",
            "registration": reg
        })),
        Err(e) => {
            let err_str = e.to_string();
            // Handle UK violation as fallback for race condition
            if err_str.contains("UQ_Event_Character") || err_str.contains("UNIQUE") {
                return HttpResponse::Conflict().json(json!({
                    "success": false, "code": "DUPLICATE",
                    "message": "This character is already registered for this event."
                }));
            }
            log::error!("Failed to register: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to register"
            }))
        }
    }
}

/// GET /events/{event_slug}/registration-status — check registration for current user
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/registration-status",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "User registrations", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/registration-status")]
pub async fn get_registration_status(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<String>,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

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

    let user_num = profile.user_num as i32;

    match db
        .web_db
        .get_registrations_by_user(event_id, user_num)
        .await
    {
        Ok(registrations) => HttpResponse::Ok().json(json!({
            "success": true,
            "event_id": event_id,
            "registrations": registrations
        })),
        Err(e) => {
            log::error!("Failed to fetch registrations: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch registration status"
            }))
        }
    }
}

/// DELETE /events/{event_slug}/register/{cha_num} — withdraw
#[utoipa::path(
    delete,
    path = "/api/v1/events/{event_slug}/register/{cha_num}",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug"),
        ("cha_num" = i32, Path, description = "Character number")
    ),
    responses(
        (status = 200, description = "Registration withdrawn", body = serde_json::Value),
        (status = 400, description = "No active registration"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event or registration not found"),
        (status = 500, description = "Database error")
    )
)]
#[delete("/register/{cha_num}")]
pub async fn withdraw_registration(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<(String, i32)>,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let (event_slug, cha_num) = path.into_inner();

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

    let user_num = profile.user_num as i32;

    // Verify ownership
    match db
        .web_db
        .get_registration_by_event_and_char(event_id, cha_num)
        .await
    {
        Ok(Some(r)) if r.user_num != user_num => {
            return HttpResponse::Forbidden().json(json!({
                "success": false, "code": "UNAUTHORIZED",
                "message": "This character does not belong to your account."
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Registration not found"
            }));
        }
        Err(e) => {
            log::error!("Failed to fetch registration: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Database error"
            }));
        }
        _ => {}
    }

    match db
        .web_db
        .withdraw_registration(event_id, cha_num, user_num)
        .await
    {
        Ok(true) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Registration withdrawn"
        })),
        Ok(false) => HttpResponse::BadRequest().json(json!({
            "success": false, "code": "NOT_FOUND",
            "message": "No active registration found to withdraw"
        })),
        Err(e) => {
            log::error!("Failed to withdraw: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to withdraw"
            }))
        }
    }
}

/// POST /events/{event_slug}/register/guild — GvG guild leader bulk registration
#[utoipa::path(
    post,
    path = "/api/v1/events/{event_slug}/register/guild",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    request_body = GvgRegisterRequest,
    responses(
        (status = 201, description = "Guild members registered", body = serde_json::Value),
        (status = 400, description = "Validation error or not GvG event"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild master"),
        (status = 404, description = "Event not found"),
        (status = 409, description = "Members already registered"),
        (status = 500, description = "Database error")
    )
)]
#[post("/register/guild")]
pub async fn register_guild(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<String>,
    body: web::Json<GvgRegisterRequest>,
    req: HttpRequest,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let event_slug = path.into_inner();

    let req_body = body.into_inner();
    let user_num = profile.user_num as i32;

    // V1: Event exists and is GvG category
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

    if !matches!(event.event_category, EventCategory::Gvg | EventCategory::GvgKoth) {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "WRONG_CATEGORY",
            "message": "Guild registration is only available for GvG events"
        }));
    }

    // V2: Registration window is open
    let now = chrono::Local::now().naive_local();
    if !event.is_registration_open(now) {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "CLOSED", "message": "Registration is no longer open."
        }));
    }

    // V3: Must provide at least one member
    if req_body.member_cha_nums.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false, "code": "VALIDATION_FAILED",
            "message": "Must provide at least one member to register"
        }));
    }

    // V4: Caller must own a character that is the guild master of the specified guild
    let user_chars = match db.game_db.get_characters_by_user(1, 1000, user_num).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to fetch user characters: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to verify guild leader"
            }));
        }
    };

    // Find the caller's character that is guild master of this guild
    let leader_char = user_chars.data.iter().find(|row| {
        let guild_num = row.get::<i32, _>("GuNum").unwrap_or(0);
        let is_master = row.get::<i32, _>("GuMasterFlag").unwrap_or(0);
        guild_num == req_body.guild_num && is_master == 1
    });

    let leader_row = match leader_char {
        Some(r) => r,
        None => {
            return HttpResponse::Forbidden().json(json!({
                "success": false, "code": "NOT_GUILD_MASTER",
                "message": "You must be the guild master to register guild members"
            }));
        }
    };

    let leader_cha_num = leader_row.get::<i32, _>("ChaNum").unwrap_or(0);
    let leader_school = leader_row.get::<i32, _>("ChaSchool").unwrap_or(0);

    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    // Insert bulk registrations
    match db
        .web_db
        .insert_gvg_bulk_registration(
            event_id,
            req_body.guild_num,
            leader_cha_num,
            &req_body.member_cha_nums,
            leader_school,
            ip_address.as_deref(),
        )
        .await
    {
        Ok(registrations) => HttpResponse::Created().json(json!({
            "success": true,
            "message": format!("{} members registered for GvG", registrations.len()),
            "registrations": registrations
        })),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("UQ_Event_Character") || err_str.contains("UNIQUE") {
                return HttpResponse::Conflict().json(json!({
                    "success": false, "code": "DUPLICATE",
                    "message": "One or more characters are already registered for this event."
                }));
            }
            log::error!("Failed to bulk register: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to register guild members"
            }))
        }
    }
}

/// GET /events/{event_slug}/guild-members — eligible guild members for GvG
#[utoipa::path(
    get,
    path = "/api/v1/events/{event_slug}/guild-members",
    tag = "Events - Registration",
    security(("cookie_session" = [])),
    params(
        ("event_slug" = String, Path, description = "Event slug")
    ),
    responses(
        (status = 200, description = "List of eligible guild members", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild master"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/guild-members")]
pub async fn get_guild_members(
    db: web::Data<Arc<DBState>>,
    session: actix_session::Session,
    path: web::Path<String>,
) -> impl Responder {
    let profile = match get_user_session(session).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let event_slug = path.into_inner();

    let user_num = profile.user_num as i32;

    // V1: Event exists
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

    // Find caller's guild master character
    let user_chars = match db.game_db.get_characters_by_user(1, 1000, user_num).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to fetch user characters: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch characters"
            }));
        }
    };

    let leader_char = user_chars.data.iter().find(|row| {
        let is_master = row.get::<i32, _>("GuMasterFlag").unwrap_or(0);
        is_master == 1
    });

    let leader_row = match leader_char {
        Some(r) => r,
        None => {
            return HttpResponse::Forbidden().json(json!({
                "success": false, "code": "NOT_GUILD_MASTER",
                "message": "You must be a guild master to view eligible guild members"
            }));
        }
    };

    let guild_num = leader_row.get::<i32, _>("GuNum").unwrap_or(0);
    let leader_school = leader_row.get::<i32, _>("ChaSchool").unwrap_or(0);

    match db
        .web_db
        .get_guild_eligible_members(event_id, guild_num, leader_school, event.min_level)
        .await
    {
        Ok(rows) => {
            let members: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "cha_num": row.get::<i32, _>("ChaNum").unwrap_or_default(),
                        "cha_name": row.get::<&str, _>("ChaName").unwrap_or_default(),
                        "cha_class": row.get::<i32, _>("ChaClass").unwrap_or_default(),
                        "cha_school": row.get::<i32, _>("ChaSchool").unwrap_or_default(),
                        "cha_level": row.get::<i32, _>("ChaLevel").unwrap_or_default(),
                        "guild_name": row.get::<&str, _>("GuildName").unwrap_or_default(),
                        "already_registered": row.get::<i32, _>("AlreadyRegistered").unwrap_or(0) == 1,
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "success": true,
                "event_id": event_id,
                "guild_num": guild_num,
                "min_level": event.min_level,
                "members": members
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch guild members: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to fetch guild members"
            }))
        }
    }
}
