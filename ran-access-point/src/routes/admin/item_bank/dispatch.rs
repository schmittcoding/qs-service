use std::sync::Arc;

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use ran_db_core::{
    gamemanager::admin::dispatch_filter::GameDispatchFilterTrait,
    shopmanager::item_bank::ItemBankDispatchTrait,
};
use ran_models::{PaginationParams, admin::item_bank::{
    DispatchCharacterFilterRequest, DispatchCharacterFilterResult,
    ItemDispatchDetailRequestPayload, ItemDispatchList,
}};
use serde_json::json;
use uuid::Uuid;

use crate::{constants::http_messages::http_messages::error, database::DBState, services::jwt::Claims};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(get_dispatch_list)
        .service(get_dispatch_by_id)
        .service(create_dispatch)
        .service(filter_characters_for_dispatch);
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
            tracing::error!("Failed to fetch the dispatch list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the dispatch list." }))
        }
    }
}

#[get("/{id}")]
async fn get_dispatch_by_id(
    path: web::Path<Uuid>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let dispatch_id: Uuid = path.into_inner();

    match db.shop_db.get_dispatch_item_by_id(dispatch_id).await {
        Ok(res) => HttpResponse::Ok().json(json!(res)),
        Err(e) => {
            tracing::error!("Failed to fetch dispatch item data: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch dispatch item data." }))
        }
    }
}

#[post("")]
async fn create_dispatch(
    req: HttpRequest,
    db: web::Data<Arc<DBState>>,
    body: web::Json<ItemDispatchDetailRequestPayload>,
) -> impl Responder {
    let data = body.into_inner();

    let items_json = match serde_json::to_string(&data.clone().items) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to serialize item data: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to process item data"
            }));
        }
    };

    let characters_json = match serde_json::to_string(&data.clone().characters) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to serialize character data: {:?}", err);
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
                        tracing::error!("Failed to dispatch item: {:?}", message);
                        HttpResponse::Forbidden()
                            .json(json!({ "error": error::FORBIDDEN }))
                    },
                    _ => {
                        tracing::error!("Failed to dispatch item: {:?}", message);
                        HttpResponse::InternalServerError()
                            .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
                    }
                }
            },
            Err(e) => {
                tracing::error!("Failed to dispatch item: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to dispatch item" }))
            }
        }
    } else {
        HttpResponse::Unauthorized().body("No valid JWT found")
    }
}

#[get("/characters/filter")]
async fn filter_characters_for_dispatch(
    db: web::Data<Arc<DBState>>,
    params: web::Query<DispatchCharacterFilterRequest>,
) -> impl Responder {
    let data = params.into_inner();
    let page = data.page.unwrap_or(0);
    let limit = data.limit.unwrap_or(50);

    let search_num: Option<i32> = data.search.as_ref().and_then(|s| s.parse::<i32>().ok());

    let login_from = match data.login_from.as_deref().map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d")) {
        Some(Ok(date)) => Some(date.and_hms_opt(0, 0, 0).unwrap()),
        Some(Err(_)) => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "Invalid login_from date format. Expected YYYY-MM-DD." }));
        }
        None => None,
    };

    let login_to = match data.login_to.as_deref().map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d")) {
        Some(Ok(date)) => Some(date.and_hms_opt(23, 59, 59).unwrap()),
        Some(Err(_)) => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "Invalid login_to date format. Expected YYYY-MM-DD." }));
        }
        None => None,
    };

    match db
        .game_db
        .filter_characters_for_dispatch(
            search_num,
            data.search,
            data.min_level,
            data.max_level,
            login_from,
            login_to,
            page,
            limit,
        )
        .await
    {
        Ok(res) => {
            let characters: Vec<DispatchCharacterFilterResult> = res
                .data
                .iter()
                .map(DispatchCharacterFilterResult::from_row)
                .collect();

            HttpResponse::Ok().json(json!({
                "data": characters,
                "page": page,
                "limit": limit,
                "total_items": res.total_records
            }))
        }
        Err(e) => {
            tracing::error!("Failed to filter characters for dispatch: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to filter characters." }))
        }
    }
}
