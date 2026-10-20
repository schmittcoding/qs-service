use actix_web::{get, post, web, HttpResponse, Responder};
use ran_db_core::shopmanager::item_shop::ItemShopTrait;
use ran_db_core::shopmanager::mileage_config::MileageConfigTrait;
use ran_db_core::shopmanager::mileage_orders::MileageOrdersTrait;
use ran_db_core::shopmanager::mileage_shop::MileageShopTrait;
use ran_db_core::webmanager::user::WebUserTrait;
use ran_models::shop::PurchaseItem;
use serde_json::json;
use std::sync::Arc;

use crate::{database::DBState, middleware::session::get_user_session, states::AppState};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_mileage_items)
        .service(mileage_purchase)
        .service(get_user_mileage_orders)
        .service(get_user_mileage_order_detail)
        .service(get_mileage_config);
}

#[get("")]
pub async fn get_mileage_items(
    session: actix_session::Session,
    _states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            match db.shop_db.get_mileage_shop_items(profile.user_num).await {
                Ok(mut items) => {
                    for item in &mut items {
                        if let Ok(gallery) = db.shop_db.get_item_gallery(item.product_num).await {
                            item.item_gallery = gallery;
                        }
                    }
                    HttpResponse::Ok().json(items)
                }
                Err(err) => {
                    tracing::error!("Failed to fetch mileage shop list: {:?}", err);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to fetch mileage shop list."
                    }))
                }
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Unauthorized"
        })),
    }
}

#[post("/purchase")]
pub async fn mileage_purchase(
    session: actix_session::Session,
    _states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    purchase_data: web::Json<Vec<PurchaseItem>>,
) -> impl Responder {
    match get_user_session(session.clone()).await {
        Ok(profile) => {
            let user_name = profile.user_name.clone();
            let user_num = profile.user_num;

            let purchase_items_json = match serde_json::to_string(&purchase_data.into_inner()) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!("Failed to serialize purchase data: {:?}", err);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to process purchase data"
                    }));
                }
            };

            match db.shop_db.mileage_purchase(user_num, purchase_items_json.clone()).await {
                Ok(row) => {
                    let order_id = row.try_get::<i64, _>("OrderID").ok().flatten().unwrap_or_default();
                    let total_cost = row.try_get::<f64, _>("TotalCost").ok().flatten().unwrap_or_default();

                    ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                    match db.web_db.get_profile_from_db(user_num).await {
                        Ok(Some(fresh_profile)) => {
                            if let Err(e) = session.insert("user_data", &fresh_profile) {
                                tracing::error!("Failed to update session after mileage purchase: {}", e);
                            }
                        }
                        Ok(None) => tracing::error!("User profile not found after mileage purchase"),
                        Err(e) => tracing::error!("Failed to fetch fresh profile after mileage purchase: {:?}", e),
                    }

                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "order_id": order_id,
                        "user_name": user_name,
                        "total_cost": total_cost,
                        "items_purchased": purchase_items_json,
                    }))
                }
                Err(err) => {
                    tracing::error!("Mileage purchase failed: {:?}", err);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Mileage purchase failed"
                    }))
                }
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Unauthorized"
        })),
    }
}

#[get("/orders")]
pub async fn get_user_mileage_orders(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    params: web::Query<ran_models::PaginationParams>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            let page = params.page.unwrap_or(1) as i32;
            let limit = params.limit.unwrap_or(10) as i32;

            match db.shop_db.get_user_mileage_orders(profile.user_num as i32, page, limit).await {
                Ok(res) => {
                    let data: Vec<ran_models::shop::orders::ShopOrderSummary> = res.data
                        .iter()
                        .map(|row| ran_models::shop::orders::ShopOrderSummary::from_row(row))
                        .collect();

                    HttpResponse::Ok().json(json!({
                        "data": data,
                        "total_items": res.total_records,
                        "page": page,
                        "limit": limit,
                    }))
                }
                Err(e) => {
                    tracing::error!("Failed to fetch mileage orders: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to fetch mileage order history."
                    }))
                }
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Unauthorized"
        })),
    }
}

#[get("/orders/{order_id}")]
pub async fn get_user_mileage_order_detail(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    path: web::Path<i64>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            let order_id = path.into_inner();

            match db.shop_db.get_mileage_order_detail(order_id, Some(profile.user_num as i32)).await {
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
        Err(_) => HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Unauthorized"
        })),
    }
}

#[get("/config")]
pub async fn get_mileage_config(
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    match db.shop_db.get_mileage_config().await {
        Ok(Some(config)) => HttpResponse::Ok().json(json!({
            "mileage_rate": config.mileage_rate,
            "description": config.description,
            "effective_from": config.effective_from,
            "effective_to": config.effective_to,
        })),
        Ok(None) => HttpResponse::Ok().json(json!({
            "mileage_rate": 0,
            "description": null,
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
