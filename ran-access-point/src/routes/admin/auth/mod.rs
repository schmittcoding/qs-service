use actix_web::{post, web, HttpResponse, Responder};
use ran_db_core::webmanager::admin::auth::WebAdminTrait;
use ran_models::{admin::auth::AuthAdminRequest, errors::flatten_validation_errors};
use serde_json::json;
use tracing::error;
use validator::Validate;

use crate::{database::DBState, services::jwt::{generate_jwt, JwtConfig}};

#[utoipa::path(
    post,
    path = "/admin/v1/auth/authenticate",
    operation_id = "admin_authenticate",
    tag = "Admin - Auth",
    request_body = AuthAdminRequest,
    responses(
        (status = 200, description = "JWT token returned in response body", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/authenticate")]
pub async fn authenticate(
    request: web::Json<AuthAdminRequest>,
    jwt_config: web::Data<JwtConfig>,
    db: web::Data<std::sync::Arc<DBState>>
) -> impl Responder {
    if let Err(validation_errors) = request.validate() {
        let errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(errors);
    }

    match db
        .web_db
        .authenticate_admin(&request.username, &request.password)
        .await {
            Ok(res) => {
                match res {
                    Some(profile) => {
                        match generate_jwt(&profile.user_id, &profile.user_type.to_string(), &jwt_config) {
                            Ok(token) => return HttpResponse::Ok().json(json!({ "token": token, "user_type": profile.user_type })),
                            Err(_) => return HttpResponse::InternalServerError().finish()
                        }
                    }
                    None => {
                        error!("Invalid user credentials");
                        HttpResponse::Unauthorized().json(json!("Invalid user credentials"))
                    }
                }
            }
            Err(e) => {
                error!("Error during authentication: {}", e);
                HttpResponse::InternalServerError().into()
            }
        }
}


pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(authenticate);
}
