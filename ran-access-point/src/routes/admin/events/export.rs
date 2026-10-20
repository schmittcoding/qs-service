use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use ran_models::events::ref_data::{event_class_name, school_name};
use serde_json::json;
use uuid::Uuid;

use crate::database::DBState;

use super::{get_admin_id, map_event_error};

fn format_filename_datetime(dt: chrono::NaiveDateTime) -> String {
    dt.format("%d%b%Y%H%M").to_string()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// GET /admin/events/{event_id}/export/characters
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/export/characters",
    operation_id = "admin_export_characters_csv",
    tag = "Admin - Export",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "CSV file with character participants", content_type = "text/csv"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/export/characters")]
pub async fn export_characters_csv(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }))
        }
        Err(e) => return map_event_error(e),
    };

    let rows = match db.web_db.list_participants_by_character(event_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export character participants: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to export participants"
            }));
        }
    };

    let mut csv = String::from("RegistrationID,ChaNum,ChaName,UserNum,GuildNum,GuildName,ChaSchool,ChaClass,ChaLevel,Status,RegisteredAt,UpdatedAt\n");
    for r in &rows {
        let registration_id = r.get::<uuid::Uuid, _>("RegistrationID")
            .map(|u| u.to_string())
            .unwrap_or_default();
        let cha_num = r.get::<i32, _>("ChaNum").unwrap_or_default();
        let cha_name = r.get::<&str, _>("ChaName").unwrap_or("").replace(',', " ");
        let user_num = r.get::<i32, _>("UserNum").unwrap_or_default();
        let guild_num = r.get::<i32, _>("GuildNum").map(|v| v.to_string()).unwrap_or_default();
        let guild_name = r.get::<&str, _>("GuildName").unwrap_or("").replace(',', " ");
        let cha_school = r.get::<i32, _>("ChaSchool").unwrap_or_default();
        let cha_class = r.get::<i32, _>("ChaClass").unwrap_or_default();
        let cha_level = r.get::<i32, _>("ChaLevel").unwrap_or_default();
        let status = r.get::<u8, _>("Status").unwrap_or(1);
        let registered_at = r.get::<chrono::NaiveDateTime, _>("RegisteredAt")
            .map(|d| d.to_string())
            .unwrap_or_default();
        let updated_at = r.get::<chrono::NaiveDateTime, _>("UpdatedAt")
            .map(|d| d.to_string())
            .unwrap_or_default();

        let class_name = event_class_name(cha_class);
        let school = school_name(cha_school);

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            registration_id, cha_num, cha_name, user_num, guild_num, guild_name,
            school, class_name, cha_level, status,
            registered_at, updated_at
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let filename = format!(
        "{}_characters_{}.csv",
        sanitize_filename(&event.event_name),
        format_filename_datetime(now)
    );

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/csv; charset=utf-8"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv)
}

/// GET /admin/events/{event_id}/export/guilds
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/export/guilds",
    operation_id = "admin_export_guilds_csv",
    tag = "Admin - Export",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID")
    ),
    responses(
        (status = 200, description = "CSV file with guild participants", content_type = "text/csv"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/export/guilds")]
pub async fn export_guilds_csv(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let event_id = path.into_inner();

    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }))
        }
        Err(e) => return map_event_error(e),
    };

    let rows = match db.web_db.list_participants_by_guild(event_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export guild participants: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to export participants"
            }));
        }
    };

    let mut csv = String::from("GuildNum,GuildName,ChaSchool,MemberCount,ActiveCount,WithdrawnCount,DisqualifiedCount\n");
    for r in &rows {
        let guild_num = r.get::<i32, _>("GuildNum").unwrap_or_default();
        let guild_name = r.get::<&str, _>("GuildName").unwrap_or("").replace(',', " ");
        let cha_school = r.get::<i32, _>("ChaSchool").map(|v| v.to_string()).unwrap_or_default();
        let member_count = r.get::<i32, _>("MemberCount").unwrap_or(0);
        let active_count = r.get::<i32, _>("ActiveCount").unwrap_or(0);
        let withdrawn_count = r.get::<i32, _>("WithdrawnCount").unwrap_or(0);
        let disqualified_count = r.get::<i32, _>("DisqualifiedCount").unwrap_or(0);

        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            guild_num, guild_name, cha_school, member_count,
            active_count, withdrawn_count, disqualified_count
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let filename = format!(
        "{}_guilds_{}.csv",
        sanitize_filename(&event.event_name),
        format_filename_datetime(now)
    );

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/csv; charset=utf-8"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv)
}

/// GET /admin/events/{event_id}/export/matches/{match_id}/guild
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/export/matches/{match_id}/guild",
    operation_id = "admin_export_guild_match_results_csv",
    tag = "Admin - Export",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "CSV file with guild match results", content_type = "text/csv"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/export/matches/{match_id}/guild")]
pub async fn export_guild_match_results_csv(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (event_id, match_id) = path.into_inner();

    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }))
        }
        Err(e) => return map_event_error(e),
    };

    let rows = match db.web_db.get_guild_match_results_export(event_id, match_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export guild match results: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to export guild match results"
            }));
        }
    };

    let mut csv = String::from("MatchID,MatchLabel,MatchDate,StartTime,EndTime,TallyStatus,RowsTallied,KillsCounted,KillsRejected,GuildNum,GuildName,School,MatchKills,MatchDeaths,MatchTower\n");
    for r in &rows {
        let guild_name = r.guild_name.replace(',', " ");
        let school = school_name(r.cha_school);
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.match_id,
            r.match_label.as_deref().unwrap_or("").replace(',', " "),
            r.match_date,
            r.start_time,
            r.end_time,
            r.tally_status,
            r.rows_tallied.map(|v| v.to_string()).unwrap_or_default(),
            r.kills_counted.map(|v| v.to_string()).unwrap_or_default(),
            r.kills_rejected.map(|v| v.to_string()).unwrap_or_default(),
            r.guild_num,
            guild_name,
            school,
            r.match_kills,
            r.match_deaths,
            r.match_tower,
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let filename = format!(
        "{}_guild_match_results_{}.csv",
        sanitize_filename(&event.event_name),
        format_filename_datetime(now)
    );

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/csv; charset=utf-8"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv)
}

/// GET /admin/events/{event_id}/export/matches/{match_id}/koth
#[utoipa::path(
    get,
    path = "/admin/v1/events/{event_id}/export/matches/{match_id}/koth",
    operation_id = "admin_export_koth_match_results_csv",
    tag = "Admin - Export",
    security(("bearer_jwt" = [])),
    params(
        ("event_id" = String, Path, description = "Event UUID"),
        ("match_id" = String, Path, description = "Match UUID")
    ),
    responses(
        (status = 200, description = "CSV file with KOTH match results", content_type = "text/csv"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/{event_id}/export/matches/{match_id}/koth")]
pub async fn export_koth_match_results_csv(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    if let Err(resp) = get_admin_id(&req) {
        return resp;
    }

    let (event_id, match_id) = path.into_inner();

    let event = match db.web_db.get_event_by_id(event_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "success": false, "code": "NOT_FOUND", "message": "Event not found"
            }))
        }
        Err(e) => return map_event_error(e),
    };

    let rows = match db.web_db.get_koth_match_results_export(event_id, match_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export KOTH match results: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false, "code": "DB_ERROR", "message": "Failed to export KOTH match results"
            }));
        }
    };

    let mut csv = String::from("MatchID,MatchLabel,MatchDate,StartTime,EndTime,TallyStatus,ChaNum,ChaName,ChaClass,GuildNum,GuildName,MatchKills,MatchDeaths\n");
    for r in &rows {
        let cha_name = r.cha_name.replace(',', " ");
        let class = event_class_name(r.cha_class);
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.match_id,
            r.match_label.as_deref().unwrap_or("").replace(',', " "),
            r.match_date,
            r.start_time,
            r.end_time,
            r.tally_status,
            r.cha_num,
            cha_name,
            class,
            r.guild_num.map(|v| v.to_string()).unwrap_or_default(),
            r.guild_name.as_deref().unwrap_or("").replace(',', " "),
            r.match_kills,
            r.match_deaths,
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let filename = format!(
        "{}_koth_match_results_{}.csv",
        sanitize_filename(&event.event_name),
        format_filename_datetime(now)
    );

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/csv; charset=utf-8"))
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv)
}
