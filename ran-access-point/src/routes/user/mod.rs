use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::webmanager::user::WebUserTrait;
use std::sync::Arc;

use crate::{database::DBState, middleware::session::get_user_session};

#[utoipa::path(
    get,
    path = "/api/v1/user/profile",
    tag = "User",
    security(("cookie_session" = [])),
    responses(
        (status = 200, description = "User profile with user_role field", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Database error")
    )
)]
#[get("/profile")]
pub async fn profile(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let session_profile = match get_user_session(session.clone()).await {
        Ok(profile) => profile,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "No session found",
                "message": "User not logged in"
            }));
        }
    };

    let user_num = session_profile.user_num;

    match db.web_db.get_profile_from_db(user_num).await {
        Ok(Some(profile_data)) => {
            // Keep session in sync with latest profile data
            if let Err(e) = session.insert("user_data", &profile_data) {
                tracing::error!("Failed to update session with fresh profile: {}", e);
            }

            HttpResponse::Ok().json(profile_data.to_json())
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Profile not found",
            "message": "No profile data found"
        })),
        Err(e) => {
            tracing::error!("Error fetching profile from DB: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": "Failed to fetch profile data"
            }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(profile);
}
