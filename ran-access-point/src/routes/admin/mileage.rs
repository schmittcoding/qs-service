use std::sync::Arc;

use actix_web::{get, put, web, HttpRequest, HttpMessage, HttpResponse, Responder};
use ran_db_core::shopmanager::mileage_config::MileageConfigTrait;
use ran_db_core::shopmanager::mileage_orders::MileageOrdersTrait;
use ran_models::shop::mileage::UpdateMileageConfigRequest;
use serde_json::json;

use crate::{database::DBState, services::jwt::Claims};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_mileage_config)
        .service(update_mileage_config)
        .service(get_admin_mileage_orders)
        .service(get_admin_mileage_order_detail);
}

#[get("/config")]
pub async fn get_mileage_config(
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    match db.shop_db.get_mileage_config().await {
        Ok(Some(config)) => HttpResponse::Ok().json(config),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "No active mileage configuration found."
        })),
        Err(e) => {
            tracing::error!("Failed to fetch mileage config: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch mileage configuration."
            }))
        }
    }
}

#[put("/config")]
pub async fn update_mileage_config(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    body: web::Json<UpdateMileageConfigRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Unauthorized"
            }));
        }
    };

    if body.mileage_rate < 0.0 || body.mileage_rate > 100.0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Mileage rate must be between 0 and 100."
        }));
    }

    match db.shop_db.update_mileage_config(
        body.mileage_rate,
        body.effective_from.as_deref(),
        body.effective_to.as_deref(),
        body.description.as_deref(),
        &claims.sub,
    ).await {
        Ok(config) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Mileage configuration updated.",
            "data": config,
        })),
        Err(e) => {
            tracing::error!("Failed to update mileage config: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update mileage configuration."
            }))
        }
    }
}

#[get("/orders")]
pub async fn get_admin_mileage_orders(
    db: web::Data<Arc<DBState>>,
    params: web::Query<ran_models::PaginationParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1) as i32;
    let limit = params.limit.unwrap_or(10) as i32;
    let search = params.search.clone();
    let status = params.status.map(|s| s as u8);

    match db.shop_db.get_admin_mileage_orders(page, limit, search, status).await {
        Ok(res) => {
            let data: Vec<ran_models::shop::orders::ShopOrder> = res.data
                .iter()
                .map(|row| ran_models::shop::orders::ShopOrder::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch admin mileage orders: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch mileage orders."
            }))
        }
    }
}

#[get("/orders/{order_id}")]
pub async fn get_admin_mileage_order_detail(
    db: web::Data<Arc<DBState>>,
    path: web::Path<i64>,
) -> impl Responder {
    let order_id = path.into_inner();

    match db.shop_db.get_mileage_order_detail(order_id, None).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => {
            tracing::error!("Failed to fetch mileage order detail {}: {:?}", order_id, e);
            HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Mileage order not found."
            }))
        }
    }
}
