use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use ran_db_core::shopmanager::item_bank::ItemBankEntriesTrait;
use ran_models::{PaginationParams, admin::item_bank::ItemBankEntry};
use serde_json::json;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_user_entries)
        .service(get_entries)
        .service(get_entry_by_id);
}

#[get("")]
pub async fn get_entries(
    db: web::Data<Arc<crate::DBState>>,
    params: web::Query<PaginationParams>,
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
            tracing::error!("Failed to fetch item bank entries: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch item bank entries." }))
        }
    }
}

#[get("/{id}")]
pub async fn get_entry_by_id(
    path: web::Path<i32>,
    db: web::Data<Arc<crate::DBState>>,
) -> impl Responder {
    let purchase_id = path.into_inner();
    match db.shop_db.get_shop_purchase_by_id(purchase_id).await {
        Ok(res) => HttpResponse::Ok().json(ItemBankEntry::from_row(&res)),
        Err(e) => {
            tracing::error!("Failed to fetch item bank entry by ID: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the item bank entry." }))
        }
    }
}

#[get("/user/{user_num}")]
pub async fn get_user_entries(
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
            let data: Vec<ItemBankEntry> = res.data
                .into_iter()
                .map(|row| ItemBankEntry::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "total_items": res.total_records,
                "page": page,
                "limit": limit,
            }))
        },
        Err(e) => {
            tracing::error!("Failed to fetch user item bank entries: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch user item bank entries." }))
        }
    }
}
