use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::webmanager::admin::auth::WebAdminTrait;
use crate::database::DBState;
use crate::services::jwt::Claims;
use tracing::error;
use uuid::Uuid;
use serde_json::json;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/admin/v1/profile",
    operation_id = "get_admin_profile",
    tag = "Admin - Auth",
    security(("bearer_jwt" = [])),
    responses(
        (status = 200, description = "Admin profile retrieved successfully", body = AuthAdminProfileData),
        (status = 400, description = "Invalid token subject"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 404, description = "Admin not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("")]
pub async fn get_profile(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    // Extract Claims from request extensions
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(json!({"message": "Unauthorized"}));
        }
    };

    // Parse user_id from claims.sub
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(json!({"message": "Invalid token subject"}));
        }
    };

    // Call database trait method
    match db.web_db.get_admin_profile(user_id).await {
        Ok(Some(profile)) => HttpResponse::Ok().json(profile),
        Ok(None) => {
            HttpResponse::NotFound()
                .json(json!({"message": "Admin not found"}))
        }
        Err(e) => {
            error!("Failed to get admin profile: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"message": "Internal server error"}))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_profile);
}
