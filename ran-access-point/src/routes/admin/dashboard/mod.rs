use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::{
    usermanager::UserAdminMetricsTrait,
    webmanager::admin::dashboard::WebDashboardTrait,
};
use serde::{Deserialize};
use serde_json::json;
use tracing::error;

use crate::database::DBState;

#[derive(Deserialize)]
pub struct TransactionTypeQuery {
    #[serde(rename = "type")]
    r#type: String,
}

#[derive(Deserialize)]
pub struct PeriodTypeQuery {
    #[serde(rename = "type")]
    r#type: String,
}

#[utoipa::path(
    get,
    path = "/admin/v1/dashboard/online",
    operation_id = "get_online_count",
    tag = "Admin - Dashboard",
    security(("bearer_jwt" = [])),
    responses(
        (status = 200, description = "Online player count", body = i32),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/online")]
pub async fn get_online_count(db: web::Data<Arc<DBState>>) -> impl Responder {
    match db.user_db.get_online_player_count().await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            error!("Failed to get online player count: {}", err);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Failed to get online player count"}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/dashboard/registered",
    operation_id = "get_registered_count",
    tag = "Admin - Dashboard",
    security(("bearer_jwt" = [])),
    responses(
        (status = 200, description = "Registered user count", body = i32),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/registered")]
pub async fn get_registered_count(db: web::Data<Arc<DBState>>) -> impl Responder {
    match db.web_db.get_registered_user_count().await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            error!("Failed to get registered user count: {}", err);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Failed to get registered user count"}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/dashboard/transactions",
    operation_id = "get_transaction_count",
    tag = "Admin - Dashboard",
    security(("bearer_jwt" = [])),
    params(
        ("type" = String, Query, description = "Transaction type: pending, approved, declined")
    ),
    responses(
        (status = 200, description = "Transaction count by status", body = i32),
        (status = 400, description = "Invalid type parameter"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/transactions")]
pub async fn get_transaction_count(
    db: web::Data<Arc<DBState>>,
    query: web::Query<TransactionTypeQuery>,
) -> impl Responder {
    let status = match query.r#type.as_str() {
        "pending" => 0i16,
        "approved" => 1i16,
        "declined" => 2i16,
        _ => {
            return HttpResponse::BadRequest()
                .json(json!({"status": "error", "message": "Invalid type parameter"}))
        }
    };

    match db.web_db.get_transaction_count_by_status(status).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            error!("Failed to get transaction count: {}", err);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Failed to get transaction count"}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/dashboard/transactions/total",
    operation_id = "get_recharge_total",
    tag = "Admin - Dashboard",
    security(("bearer_jwt" = [])),
    params(
        ("type" = String, Query, description = "Period type: today, month, year")
    ),
    responses(
        (status = 200, description = "Total recharge amount", body = f64),
        (status = 400, description = "Invalid type parameter"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/transactions/total")]
pub async fn get_recharge_total(
    db: web::Data<Arc<DBState>>,
    query: web::Query<PeriodTypeQuery>,
) -> impl Responder {
    match query.r#type.as_str() {
        "today" | "month" | "year" => {}
        _ => {
            return HttpResponse::BadRequest()
                .json(json!({"status": "error", "message": "Invalid type parameter"}))
        }
    }

    match db.web_db.get_total_recharge_amount(&query.r#type).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            error!("Failed to get total recharge amount: {}", err);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Failed to get total recharge amount"}))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_online_count)
        .service(get_registered_count)
        .service(get_transaction_count)
        .service(get_recharge_total);
}
