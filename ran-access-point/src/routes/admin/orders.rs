use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use ran_db_core::shopmanager::orders::ShopOrdersTrait;
use ran_models::{PaginationParams, shop::orders::{ShopOrder, ShopOrderItemWithDate}};
use serde_json::json;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_admin_orders)
        .service(get_admin_order_detail)
        .service(get_orders_by_user_num)
        .service(get_order_items_by_user_num);
}

#[get("")]
pub async fn get_admin_orders(
    db: web::Data<Arc<crate::DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1) as i32;
    let limit = params.limit.unwrap_or(10) as i32;
    let search = params.search.clone();
    let status = params.status.map(|s| s as u8);

    match db.shop_db.get_admin_orders(page, limit, search, status).await {
        Ok(res) => {
            let data: Vec<ShopOrder> = res.data
                .iter()
                .map(|row| ShopOrder::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to fetch admin orders: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch orders." }))
        }
    }
}

#[get("/{order_id}")]
pub async fn get_admin_order_detail(
    db: web::Data<Arc<crate::DBState>>,
    path: web::Path<i64>,
) -> impl Responder {
    let order_id = path.into_inner();

    match db.shop_db.get_order_detail(order_id, None).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => {
            tracing::error!("❌ Failed to fetch order detail {}: {:?}", order_id, e);
            HttpResponse::NotFound()
                .json(json!({ "error": "Order not found." }))
        }
    }
}

#[get("/user/{user_num}")]
pub async fn get_orders_by_user_num(
    db: web::Data<Arc<crate::DBState>>,
    path: web::Path<i32>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page = params.page.unwrap_or(1) as i32;
    let limit = params.limit.unwrap_or(10) as i32;

    match db.shop_db.get_orders_by_user_num(user_num, page, limit).await {
        Ok(res) => {
            let data: Vec<ShopOrder> = res.data
                .iter()
                .map(|row| ShopOrder::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to fetch orders for user {}: {:?}", user_num, e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch user orders." }))
        }
    }
}

#[get("/user/{user_num}/items")]
pub async fn get_order_items_by_user_num(
    db: web::Data<Arc<crate::DBState>>,
    path: web::Path<i32>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page = params.page.unwrap_or(1) as i32;
    let limit = params.limit.unwrap_or(10) as i32;

    match db.shop_db.get_order_items_by_user_num(user_num, page, limit).await {
        Ok(res) => {
            let data: Vec<ShopOrderItemWithDate> = res.data
                .iter()
                .map(|row| ShopOrderItemWithDate::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to fetch order items for user {}: {:?}", user_num, e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch user order items." }))
        }
    }
}
