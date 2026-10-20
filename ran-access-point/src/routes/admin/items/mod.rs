use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use ran_db_core::{shopmanager::items::ItemTrait, gamemanager::admin::items::GameItemTrait};
use ran_models::admin::items::{ItemMapOptions, Item, ItemListQuery};
use serde_json::json;
use uuid::Uuid;

use crate::{constants::http_messages::http_messages::error, database::DBState};

pub mod dispatch;

#[get("")]
async fn list_items(
    query: web::Query<ItemListQuery>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let page = query.page.unwrap_or(1).max(1) as i16;
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100) as i16;

    match db
        .game_db
        .get_items(page, page_size, query.cha_num, query.item_main, query.item_sub, query.inven_type)
        .await
    {
        Ok(rows) => {
            let data: Vec<Item> = rows
                .iter()
                .map(|row| Item::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": page_size,
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch items: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[get("/{uuid}")]
async fn get_item_by_uuid(
    path: web::Path<Uuid>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let item_uuid = path.into_inner();

    match db.game_db.get_item_by_uuid(item_uuid).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": error::NOT_FOUND })),
        Err(e) => {
            log::error!("Failed to fetch item {}: {:?}", item_uuid, e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[get("/{cha_num}/character")]
async fn list_items_by_cha_num(
    path: web::Path<i32>,
    query: web::Query<ItemListQuery>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let cha_num = path.into_inner();
    let page = query.page.unwrap_or(1).max(1) as i16;
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100) as i16;

    match db.game_db.get_items_by_cha_num(page, page_size, cha_num, query.item_main, query.item_sub, query.inven_type).await {
        Ok(rows) => {
            let data: Vec<Item> = rows
                .iter()
                .map(|row| Item::from_row(row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": page_size,
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch items for character {}: {:?}", cha_num, e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[get("/item-options")]
async fn get_item_map_options(
    db: web::Data<Arc<DBState>>
) -> impl Responder {
    match db.shop_db.get_item_map_options().await {
        Ok(res) => {
            let data: Vec<ItemMapOptions> = res
                .into_iter()
                .map(|row| ItemMapOptions::from_row(&row))
                .collect();

            HttpResponse::Ok().json(data)
        },
        Err(e) => {
            log::error!("Failed to fetch the dispatch item options: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the dispatch item options." }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(get_item_map_options)
        .service(web::scope("/dispatch")
            .configure(dispatch::configure))
        .service(list_items_by_cha_num)
        .service(list_items)
        .service(get_item_by_uuid);
}
