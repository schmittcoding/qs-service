use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

use crate::database::DBState;
use ran_db_core::test_db_connection;

#[utoipa::path(
    get,
    path = "/api/v1/checks/health-check",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = serde_json::Value)
    )
)]
#[get("/health-check")]
pub async fn health_checker_handler() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "success", "message": "OK"}))
}

#[utoipa::path(
    get,
    path = "/api/v1/checks/database-check",
    tag = "Health",
    responses(
        (status = 200, description = "Database is healthy", body = serde_json::Value),
        (status = 503, description = "Database unavailable")
    )
)]
#[get("/database-check")]
pub async fn database_health_checker(
    data: actix_web::web::Data<std::sync::Arc<DBState>>,
) -> impl actix_web::Responder {
    let (user_db_status, web_db_status, log_db_status, game_db_status, shop_db_status) =
        tokio::try_join!(
            test_db_connection(&*data.user_db),
            test_db_connection(&*data.web_db),
            test_db_connection(&*data.log_db),
            test_db_connection(&*data.game_db),
            test_db_connection(&*data.shop_db)
        )
        .unwrap_or((false, false, false, false, false)); // Default to Ok(false) if any test fails

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "web_db": web_db_status,
        "user_db": user_db_status,
        "log_db": log_db_status,
        "game_db": game_db_status,
        "shop_db": shop_db_status
    }))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(health_checker_handler)
        .service(database_health_checker);
}
