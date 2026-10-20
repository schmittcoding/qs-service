use actix_web::{get, post, web, HttpResponse, Responder};
use ran_db_core::shopmanager::item_shop::ItemShopTrait;
use ran_db_core::shopmanager::orders::ShopOrdersTrait;
use ran_db_core::webmanager::user::WebUserTrait;
use ran_models::shop::PurchaseItem;
use serde_json::json;
use std::convert::Into;
use std::sync::Arc;

use crate::{database::DBState, middleware::session::get_user_session, states::AppState};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_items)
        .service(shop_purchase)
        .service(get_user_orders)
        .service(get_user_order_detail);
}

#[get("")]
pub async fn get_items(
    session: actix_session::Session,
    _states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            match db.shop_db.get_shop_items(profile.user_num).await {
                Ok(mut items) => {
                    for item in &mut items {
                        if let Ok(gallery) = db.shop_db.get_item_gallery(item.product_num).await {
                            item.item_gallery = gallery;
                        }
                    }
                    HttpResponse::Ok().json(items)
                }
                Err(err) => {
                    tracing::error!("❌ Failed to fetch item shop list: {:?}", err);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to fetch item shop list."
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
pub async fn shop_purchase(
    session: actix_session::Session,
    _states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    purchase_data: web::Json<Vec<PurchaseItem>>, // Expecting multiple items
) -> impl Responder {
    match get_user_session(session.clone()).await {
        Ok(profile) => {
            let user_name = profile.user_name.clone();
            let user_num = profile.user_num;

            // Serialize the purchase data into JSON.
            let purchase_items_json = match serde_json::to_string(&purchase_data.into_inner()) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!("❌ Failed to serialize purchase data: {:?}", err);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to process purchase data"
                    }));
                }
            };

            match db.shop_db.shop_purchase(user_num, purchase_items_json.clone()).await {
                Ok(row) => {
                    let order_id = row.try_get::<i64, _>("OrderID").ok().flatten().unwrap_or_default();
                    let total_cost = row.try_get::<f64, _>("TotalCost").ok().flatten().unwrap_or_default();
                    let mileage_earned = row.try_get::<f64, _>("MileageEarned").ok().flatten().unwrap_or(0.0);
                    let mileage_rate = row.try_get::<f64, _>("MileageRate").ok().flatten().unwrap_or(0.0);

                    // Refresh session with fresh profile data from DB
                    ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                    match db.web_db.get_profile_from_db(user_num).await {
                        Ok(Some(fresh_profile)) => {
                            if let Err(e) = session.insert("user_data", &fresh_profile) {
                                tracing::error!("❌ Failed to update session after purchase: {}", e);
                            }
                        }
                        Ok(None) => tracing::error!("❌ User profile not found after purchase"),
                        Err(e) => tracing::error!("❌ Failed to fetch fresh profile after purchase: {:?}", e),
                    }

                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "order_id": order_id,
                        "user_name": user_name,
                        "total_cost": total_cost,
                        "mileage_earned": mileage_earned,
                        "mileage_rate": mileage_rate,
                        "items_purchased": purchase_items_json,
                    }))
                }
                Err(err) => {
                    tracing::error!("❌ Stored procedure execution failed: {:?}", err);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Purchase failed"
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
pub async fn get_user_orders(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    params: web::Query<ran_models::PaginationParams>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            let page = params.page.unwrap_or(1) as i32;
            let limit = params.limit.unwrap_or(10) as i32;

            match db.shop_db.get_user_orders(profile.user_num as i32, page, limit).await {
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
                    tracing::error!("❌ Failed to fetch user orders: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to fetch order history."
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
pub async fn get_user_order_detail(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    path: web::Path<i64>,
) -> impl Responder {
    match get_user_session(session).await {
        Ok(profile) => {
            let order_id = path.into_inner();

            match db.shop_db.get_order_detail(order_id, Some(profile.user_num as i32)).await {
                Ok(detail) => HttpResponse::Ok().json(detail),
                Err(e) => {
                    tracing::error!("❌ Failed to fetch order detail {}: {:?}", order_id, e);
                    HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "Order not found."
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
