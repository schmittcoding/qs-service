use actix_web::{get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::webmanager::download::WebDownloadTrait;
use ran_models::{download::{DownloadItem, DownloadItemPayload}, PaginationParams};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{constants::http_messages::http_messages::{error, success}, database::DBState, services::jwt::Claims, states::AppState};

#[get("")]
pub async fn get_download_items(
    db: web::Data<Arc<DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    let status = params.status.map(|p| p as u8);
    let r#type = params.category;

    let links = match db.web_db.get_download_links(status.clone(), r#type.clone()).await {
        Ok(data) => data,
        Err(err) => {
            log::error!("❌ Failed to download links from database: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }));
        }
    };

    let total_items = links.len();
    let total_pages = if total_items == 0 {
        1
    } else {
        (total_items as f64 / limit as f64).ceil() as usize
    };

    // Adjust page if the requested page is out-of-range.
    let current_page = if page > total_pages {
        total_pages
    } else {
        page
    };
    let start = (current_page - 1) * limit;

    let paginated: Vec<DownloadItem> = links
        .into_iter()
        .skip(start)
        .take(limit)
        .collect();

    HttpResponse::Ok().json(json!({
        "data": paginated,
        "total_items": total_items
    }))
}

#[post("")]
pub async fn add_download_item(
    req: HttpRequest,
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    body: web::Json<DownloadItemPayload>,
) -> impl Responder {
    let data = body.into_inner();

    if let Some(claims) = req.extensions().get::<Claims>() {
        match db.web_db.add_download_link(data.clone(), claims.sub.clone()).await {
            Ok((message, id)) => {
                match message.as_str() {
                    "SUCCESS" => {
                        let cache_instance = states.download_links_cache.lock().await;
                        let mut data_cache = cache_instance.data.lock().await;

                        let data_vec = data_cache.get_or_insert(Vec::new());

                        // TODO: Add admin logs
                        data_vec.push(DownloadItem {
                            id,
                            link: data.link.clone(),
                            provider: data.provider.clone(),
                            status: data.status.clone(),
                            version: data.version.clone(),
                            r#type: data.r#type.clone()
                        });

                        data_vec.sort_by(|a, b| b.version.cmp(&a.version));

                        drop(data_cache);
                        drop(cache_instance);

                        HttpResponse::Ok().json(json!({ "message": success::ADD }))
                    },
                    "UNAUTHORIZED" => {
                        log::error!("❌ Failed to add web download data: {:?}", message);
                        HttpResponse::Forbidden()
                            .json(json!({ "error": error::FORBIDDEN }))
                    },
                    "DUPLICATE_DATA" => {
                        log::error!("❌ Failed to add web download data: {:?}", message);
                        HttpResponse::Conflict()
                            .json(json!({ "error": error::DUPLICATE_DATA }))
                    },
                    _ => {
                        log::error!("❌ Failed to add web download data: {:?}", message);
                        HttpResponse::InternalServerError()
                            .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
                    }
                }
            },
            Err(e) => {
                log::error!("❌ Failed to add web download data: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
            }
        }
    } else {
        HttpResponse::Unauthorized()
            .json(json!({ "error": error::UNAUTHORIZED }))
    }
}

#[put("/{id}")]
pub async fn update_download_item(
    path: web::Path<Uuid>,
    req: HttpRequest,
    states: web::Data<Arc<AppState>>,
    db: web::Data<Arc<DBState>>,
    body: web::Json<DownloadItemPayload>,
) -> impl Responder {
    let id = path.into_inner();
    let data = body.into_inner();

    if let Some(claims) = req.extensions().get::<Claims>() {
        match db.web_db.edit_download_link(id.clone(), data.clone(), claims.sub.clone()).await {
            Ok(message) => {
                match message.as_str() {
                    "SUCCESS" => {
                        let cache_instance = states.download_links_cache.lock().await;
                        let mut data_cache = cache_instance.data.lock().await;
                        
                        let data_vec = data_cache.get_or_insert_with(Vec::new);

                        let new_item_data = DownloadItem {
                            id,
                            link: data.link.clone(),
                            provider: data.provider.clone(),
                            status: data.status.clone(),
                            version: data.version.clone(),
                            r#type: data.r#type.clone()
                        };

                        // TODO: Add admin logs
                        let mut new_vec: Vec<DownloadItem> = data_vec
                            .iter()
                            .map(|d| {
                                if d.id == id {
                                    new_item_data.clone()
                                } else {
                                    d.clone()
                                }
                            })
                            .collect();

                        new_vec.sort_by(|a, b| b.version.cmp(&a.version));
                        *data_cache = Some(new_vec);

                        drop(data_cache);
                        drop(cache_instance);

                        HttpResponse::Ok().json(json!({ "message": success::EDIT }))
                    },
                    "UNAUTHORIZED" => {
                        log::error!("❌ Failed to edit web download data: {:?}", message);
                        HttpResponse::Forbidden()
                            .json(json!({ "error": error::FORBIDDEN }))
                    },
                    "NOT_FOUND" => {
                        log::error!("❌ Failed to edit web download data: {:?}", message);
                        HttpResponse::NotFound()
                            .json(json!({ "error": error::NOT_FOUND }))
                    },
                    _ => {
                        log::error!("❌ Failed to edit web download data: {:?}", message);
                        HttpResponse::InternalServerError()
                            .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
                    }
                }
            },
            Err(e) => {
                log::error!("❌ Failed to edit web download data: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
            }
        }
    } else {
        HttpResponse::Unauthorized()
            .json(json!({ "error": error::UNAUTHORIZED }))
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(add_download_item)
        .service(update_download_item)
        .service(get_download_items);
}
