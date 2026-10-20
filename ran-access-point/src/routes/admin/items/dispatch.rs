use std::{sync::Arc};

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::{shopmanager::items::dispatch::ItemDispatchTrait};
use ran_models::{PaginationParams, admin::items::dispatch::{ItemDispatchDetailRequestPayload, ItemDispatchList}};
use serde_json::json;
use uuid::Uuid;

use crate::{constants::http_messages::http_messages::error, database::DBState, services::jwt::Claims};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(get_dispatch_list)
        .service(get_dispatch_by_id)
        .service(dispatch_item);
}

#[get("")]
async fn get_dispatch_list(
    db: web::Data<Arc<DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(0);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let search: Option<String> = params.search.clone();

    let result = db.shop_db.get_dispatch_list(page, limit, search).await;

    match result {
        Ok(res) => {
            let data: Vec<ItemDispatchList> = res.data
                .into_iter()
                .map(|row| ItemDispatchList::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": limit,
                "total_items": res.total_records
            }))
        },
        Err(e) => {
            log::error!("❌ Failed to fetch the dispatch list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the dispatch list." }))
        }
    }
}

#[get("/{id}")]
async fn get_dispatch_by_id(
    path: web::Path<Uuid>,
    db: web::Data<Arc<crate::database::DBState>>,
) -> impl Responder {
    let dispatch_id: Uuid = path.into_inner();

    match db.shop_db.get_dispatch_item_by_id(dispatch_id).await {
        Ok(res) => HttpResponse::Ok().json(json!(res)),
        Err(e) => {
            log::error!("Failed to fetch dispatch item data: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to dispatch item data." }))
        }
    }
}

#[post("")]
async fn dispatch_item(
    req: HttpRequest,
    db: web::Data<Arc<crate::database::DBState>>,
    body: web::Json<ItemDispatchDetailRequestPayload>,
) -> impl Responder {
    let data = body.into_inner();

    let items_json = match serde_json::to_string(&data.clone().items) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("❌ Failed to serialize item data: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to process item data"
            }));
        }
    };

    let characters_json = match serde_json::to_string(&data.clone().characters) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("❌ Failed to serialize character data: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to process character data"
            }));
        }
    };

    if let Some(claims) = req.extensions().get::<Claims>() {
        match db
            .shop_db
            .dispatch_item(items_json, characters_json, data.notes.clone(), claims.sub.clone())
            .await
        {
            Ok((message, id)) => {
                match message.as_str() {
                    "SUCCESS" => {
                        HttpResponse::Ok().json(json!({ "success": true, "dispatch_id": id }))
                    },
                    "UNAUTHORIZED" => {
                        log::error!("❌ Failed to dispatch item: {:?}", message);
                        HttpResponse::Forbidden()
                            .json(json!({ "error": error::FORBIDDEN }))
                    },
                    _ => {
                        log::error!("❌ Failed to dispatch item: {:?}", message);
                        HttpResponse::InternalServerError()
                            .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
                    }
                }
            },
            Err(e) => {
                log::error!("❌ Failed to dispatch item: {:?}", e);
                HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to dispatch item" }))
            }

        }
    } else {
        HttpResponse::Unauthorized().body("No valid JWT found")
    }
}
