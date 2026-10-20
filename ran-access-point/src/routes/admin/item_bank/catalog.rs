use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use ran_db_core::shopmanager::item_bank::ItemBankCatalogTrait;
use ran_models::admin::item_bank::ItemCatalogOption;
use serde_json::json;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_catalog_items);
}

#[get("")]
pub async fn get_catalog_items(
    db: web::Data<Arc<crate::DBState>>,
) -> impl Responder {
    match db.shop_db.get_item_map_options().await {
        Ok(res) => {
            let data: Vec<ItemCatalogOption> = res
                .into_iter()
                .map(|row| ItemCatalogOption::from_row(&row))
                .collect();

            HttpResponse::Ok().json(data)
        },
        Err(e) => {
            tracing::error!("Failed to fetch item catalog options: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch item catalog options." }))
        }
    }
}
