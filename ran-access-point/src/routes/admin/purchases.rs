use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use ran_db_core::shopmanager::purchases::PurchasesTrait;
use ran_models::{PaginationParams, admin::{item_bank::ItemBankEntry, purchases::ShopPurchaseBase}};
use serde_json::json;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_shop_purchases)
        .service(get_shop_purchase_by_id)
        .service(get_user_purchases);
}

#[get("")]
pub async fn get_shop_purchases(
    db: web::Data<Arc<crate::DBState>>,
    params: web::Query<PaginationParams>
) -> impl Responder {
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let status: Option<i16> = params.status.clone();
    let search: Option<String> = params.search.clone();

    match db.shop_db.get_shop_purchases(page, limit, search, status).await {
        Ok(res) => {
            let data: Vec<ItemBankEntry> = res.data
                .iter()
                .map(|row| ItemBankEntry::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": limit,
                "total_items": res.total_records,
            }))
        },
        Err(e) => {
            log::error!("❌ Failed to fetch shop purchases: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch shop purchases." }))
        }
    }
}

#[get("/{id}")]
pub async fn get_shop_purchase_by_id(
    path: web::Path<i32>,
    db: web::Data<Arc<crate::DBState>>,
) -> impl Responder {
    let purchase_id = path.into_inner();
    match db.shop_db.get_shop_purchase_by_id(purchase_id).await {
        Ok(res) => HttpResponse::Ok().json(ShopPurchaseBase::from_row(&res)),
        Err(e) => {
            log::error!("❌ Failed to fetch the shop purchase by ID: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the shop purchase by ID." }))
        }
    }
}

#[get("/{user_num}/user")]
pub async fn get_user_purchases(
    path: web::Path<i32>,
    params: web::Query<PaginationParams>,
    db: web::Data<Arc<crate::DBState>>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let status: Option<i16> = params.status.clone();

    match db.shop_db.get_user_shop_purchases(user_num, page, limit, status).await {
        Ok(res) => {
            let data: Vec<ShopPurchaseBase> = res.data
                .into_iter()
                .map(|row| ShopPurchaseBase::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        },
        Err(e) => {
            log::error!("❌ Failed to fetch user purchases: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch user purchases." }))
        }
    }
}
