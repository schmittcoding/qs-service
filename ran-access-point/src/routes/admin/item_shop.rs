use std::sync::Arc;

use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use anyhow::Result;
use ran_db_core::shopmanager::item_shop::ItemShopTrait;
use ran_models::{admin::AdminItemShopFilterParams, item_shop::{GetItemShopData, ItemShopBase, ItemShopData, UpdateItemShopRequest}};
use serde_json::json;
use serde_qs::Config;

use crate::{constants::http_messages::http_messages::{error, success}, database::DBState, routes::utils::upload_image_to_r2};

const MAX_GALLERY_IMAGES: usize = 5;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_shop_items)
        .service(get_shop_item)
        .service(edit_shop_item)
        .service(add_shop_item);
}

#[derive(Debug, MultipartForm)]
struct AddForm {
    #[multipart(limit = "5MB")]
    file: TempFile,
    #[multipart(limit = "5MB")]
    gallery: Vec<TempFile>,
    data: MpJson<ItemShopBase>
}

#[derive(Debug, MultipartForm)]
struct EditForm {
    #[multipart(limit = "5MB")]
    file: Option<TempFile>,
    #[multipart(limit = "5MB")]
    gallery: Vec<TempFile>,
    data: MpJson<UpdateItemShopRequest>
}

async fn upload_image(file: TempFile, db: web::Data<Arc<DBState>>) -> Result<String> {
    let image_bytes = match tokio::fs::read(file.file.path()).await {
        Ok(bytes) => bytes,
        Err(e) => anyhow::bail!("❌ Failed to read uploaded file: {:?}", e)
    };

    let url =
        upload_image_to_r2(&db.r2_client, &db.r2_config, "shop", image_bytes)
        .await
        .unwrap();

    Ok(url)
}

async fn upload_gallery_images(
    files: Vec<TempFile>,
    db: &web::Data<Arc<DBState>>,
    product_num: i32,
) -> Result<()> {
    let files: Vec<TempFile> = files.into_iter().take(MAX_GALLERY_IMAGES).collect();

    let mut gallery_entries: Vec<serde_json::Value> = Vec::new();

    for (i, file) in files.into_iter().enumerate() {
        let image_bytes = tokio::fs::read(file.file.path()).await?;
        let url = upload_image_to_r2(&db.r2_client, &db.r2_config, "shop", image_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("❌ Failed to upload gallery image: {:?}", e))?;

        gallery_entries.push(json!({
            "url": url,
            "sort_order": i as i16
        }));
    }

    if !gallery_entries.is_empty() {
        let images_json = serde_json::to_string(&gallery_entries)?;
        db.shop_db.add_item_gallery(product_num, images_json).await?;
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/admin/v1/item-shop",
    operation_id = "admin_get_item_shop",
    tag = "Admin - Item Shop",
    security(("bearer_jwt" = [])),
    params(
        ("page" = Option<i32>, Query, description = "Page number"),
        ("limit" = Option<i32>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search by item name"),
        ("category" = Option<Vec<i32>>, Query, description = "Filter by category IDs"),
        ("status" = Option<Vec<i32>>, Query, description = "Filter by status values"),
        ("purchase_limit" = Option<Vec<i32>>, Query, description = "Filter by purchase limit thresholds")
    ),
    responses(
        (status = 200, description = "List of items with pagination"),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("")]
pub async fn get_shop_items(
    db: web::Data<Arc<crate::DBState>>,
    req: HttpRequest,
) -> impl Responder {
    let qs = req.query_string();
    let config = Config::new(5, false);

    let params: AdminItemShopFilterParams = match config.deserialize_str(qs) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid query: {}", e)),
    };

    let page: i32 = params.page.map(|p| p as i32).unwrap_or(0);
    let limit: i32 = params.limit.map(|p| p as i32).unwrap_or(10);
    let shop_type: i16 = params.shop_type.unwrap_or(1);

    let search = params.search.clone();
    let category = params.category.as_ref().filter(|v| !v.is_empty()).cloned();
    let status = params.status.as_ref().filter(|v| !v.is_empty()).cloned();
    let purchase_limit = params.purchase_limit.as_ref().filter(|v| !v.is_empty()).cloned();

    match db.shop_db.get_admin_shop_items(page, limit, search, category, status, purchase_limit, shop_type).await {
        Ok(res) => {
            let data: Vec<GetItemShopData> = res.data
                .into_iter()
                .map(|row| GetItemShopData::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": limit,
                "total_items": res.total_records
            }))
        },
        Err(e) => {
            log::error!("❌ Failed to fetch the item shop list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the item shop list." }))
        }
    }
}

#[get("/{id}")]
pub async fn get_shop_item(
    path: web::Path<i32>,
    db: web::Data<Arc<crate::DBState>>,
) -> impl Responder {
    let product_num: i32 = path.into_inner();

    match db.shop_db.get_shop_item_by_id(product_num).await {
        Ok(res) => {
            let mut item = ItemShopData::from_row(&res);

            match db.shop_db.get_item_gallery(product_num).await {
                Ok(gallery) => item.item_gallery = gallery,
                Err(e) => log::error!("❌ Failed to fetch gallery images: {:?}", e),
            }

            HttpResponse::Ok().json(item)
        },
        Err(e) => {
            log::error!("❌ Failed to fetch the item shop list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch the item shop list." }))
        }
    }
}

#[put("/{id}")]
pub async fn edit_shop_item(
    path: web::Path<i32>,
    db: web::Data<Arc<DBState>>,
    MultipartForm(form): MultipartForm<EditForm>,
) -> impl Responder {
    let data = form.data.0.clone();
    let product_num = path.into_inner();

    let mut url: String = data.item_image.unwrap_or("".to_string());

    if let Some(file) = form.file {
        url = upload_image(file, db.clone()).await.ok().unwrap_or_default();
    }

    // Delete removed gallery images
    if let Some(removed_ids) = data.removed_gallery_ids {
        if !removed_ids.is_empty() {
            if let Err(e) = db.shop_db.delete_item_gallery(product_num, removed_ids).await {
                log::error!("❌ Failed to delete gallery images: {:?}", e);
            }
        }
    }

    // Upload new gallery images
    if !form.gallery.is_empty() {
        if let Err(e) = upload_gallery_images(form.gallery, &db, product_num).await {
            log::error!("❌ Failed to upload gallery images: {:?}", e);
        }
    }

    match db.shop_db.edit_shop_item(product_num, data.base, url).await {
        Ok(mut res) => {
            match db.shop_db.get_item_gallery(product_num).await {
                Ok(gallery) => res.item_gallery = gallery,
                Err(e) => log::error!("❌ Failed to fetch gallery images: {:?}", e),
            }

            HttpResponse::Ok()
                .json(json!({ "data": res, "message": success::EDIT }))
        },
        Err(e) => {
            log::error!("❌ Failed to update shop item details: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[post("")]
pub async fn add_shop_item(
    db: web::Data<Arc<DBState>>,
    MultipartForm(form): MultipartForm<AddForm>,
) -> impl Responder {
    let data = form.data.0.clone();

    let url: String = upload_image(form.file, db.clone()).await.ok().unwrap_or_default();

    match db.shop_db.add_shop_item(data, url).await {
        Ok(mut res) => {
            // Upload gallery images if provided
            if !form.gallery.is_empty() {
                if let Err(e) = upload_gallery_images(form.gallery, &db, res.product_num).await {
                    log::error!("❌ Failed to upload gallery images: {:?}", e);
                }

                match db.shop_db.get_item_gallery(res.product_num).await {
                    Ok(gallery) => res.item_gallery = gallery,
                    Err(e) => log::error!("❌ Failed to fetch gallery images: {:?}", e),
                }
            }

            HttpResponse::Ok()
                .json(json!({ "data": res, "message": success::ADD }))
        },
        Err(e) => {
            log::error!("❌ Failed to add shop item: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}
