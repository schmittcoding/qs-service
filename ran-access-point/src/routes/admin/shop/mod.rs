use std::sync::Arc;

use actix_multipart::form::tempfile::TempFile;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use anyhow::Result;
use base64::Engine;
use ran_db_core::shopmanager::ShopDBTrait;
use ran_models::{
    errors::flatten_validation_errors,
    shop::{AddShopData, AdminShopDataResponse, AdminShopQuery, EditShopData, ShopDataResponse, ShopMigrate},
};
use serde_json::json;
use validator::Validate;

use crate::{database::DBState, routes::utils::upload_image_to_r2, states::AppState};

#[get("/{id}")]
async fn get_shop_item_by_id(
    path: web::Path<i32>,
    db: web::Data<Arc<crate::database::DBState>>,
) -> impl Responder {
    let product_num: i32 = path.into_inner();

    match db.shop_db.get_shop_item_by_id(product_num).await {
        Ok(item) => match item.to_admin_response() {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => {
                tracing::error!("❌ Failed to convert item to admin response: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to process item response" }))
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to fetch item info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch item info" }))
        }
    }
}

#[delete("/delete-item/{id}")]
async fn delete_shop_item_by_id(
    path: web::Path<i32>,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    let product_num = path.into_inner();

    match db.shop_db.delete_shop_item_by_id(product_num).await {
        Ok(shop_data) => match shop_data.to_admin_response() {
            Ok(response) => {
                let mut cache = states.shop_data.lock().await;
                match db.shop_db.get_shop_data().await {
                    Ok(updated_data) => {
                        let shop_data: Vec<ShopDataResponse> = updated_data
                            .into_iter()
                            .map(|item| item.to_response())
                            .filter_map(Result::ok)
                            .collect();

                        cache.insert(0, shop_data);
                        tracing::info!("✅ Cache successfully updated after item addition.");
                    }
                    Err(err) => {
                        tracing::error!("❌ Failed to refresh cache after adding item: {:?}", err);
                    }
                }
                HttpResponse::Ok().json(response)
            }
            Err(e) => {
                tracing::error!("❌ Failed to convert item to admin response: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to process item response" }))
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to fetch item info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch item info" }))
        }
    }
}

#[put("/{id}")]
async fn edit_shop_item_by_id(
    path: web::Path<i32>,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
    item_data: web::Json<EditShopData>,
) -> impl Responder {
    let product_num = path.into_inner();
    let item_data = item_data.into_inner();

    match db
        .shop_db
        .edit_shop_item_by_id(product_num, item_data)
        .await
    {
        Ok(shop_data) => match shop_data.to_admin_response() {
            Ok(response) => {
                let mut cache = states.shop_data.lock().await;
                match db.shop_db.get_shop_data().await {
                    Ok(updated_data) => {
                        let shop_data: Vec<ShopDataResponse> = updated_data
                            .into_iter()
                            .map(|item| item.to_response())
                            .filter_map(Result::ok)
                            .collect();

                        cache.insert(0, shop_data);
                        tracing::info!("✅ Cache successfully updated after item addition.");
                    }
                    Err(err) => {
                        tracing::error!("❌ Failed to refresh cache after adding item: {:?}", err);
                    }
                }
                HttpResponse::Ok().json(response)
            }
            Err(e) => {
                tracing::error!("❌ Failed to convert item to admin response: {:?}", e);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to process item response" }))
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to fetch item info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch item info" }))
        }
    }
}

#[get("")]
pub async fn get_shop_items(
    db: web::Data<Arc<crate::DBState>>,
    query: web::Query<AdminShopQuery>,
) -> impl Responder {
    match db.shop_db.get_shop_data().await {
        Ok(data) => {
            let filtered_data: Vec<AdminShopDataResponse> = data
                .into_iter()
                .filter(|item| {
                    query.category.is_empty() || query.category.contains(&item.item_category)
                })
                .map(|item| item.to_admin_response())
                .filter_map(Result::ok)
                .collect();

            // Apply pagination
            let page = query.page.unwrap_or(1);
            let limit = query.limit.unwrap_or(10);
            let total_items = filtered_data.len();
            let total_pages = if total_items == 0 {
                1
            } else {
                (total_items as f64 / limit as f64).ceil() as usize
            };
            let current_page = page.min(total_pages);
            let start = (current_page - 1) * limit;

            let paginated_data = filtered_data
                .into_iter()
                .skip(start)
                .take(limit)
                .collect::<Vec<_>>();

            HttpResponse::Ok().json(json!({
                "page": current_page,
                "limit": limit,
                "total_pages": total_pages,
                "total_items": total_items,
                "data": paginated_data
            }))
        }
        Err(err) => {
            log::error!("❌ Failed to fetch shop data from database: {:?}", err);
            HttpResponse::InternalServerError().json(json!({ "error": "Failed to load shop data" }))
        }
    }
}

#[post("")]
pub async fn add_item_data(
    states: web::Data<Arc<AppState>>,
    item_data: web::Json<AddShopData>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    if let Err(validation_errors) = item_data.validate() {
        let errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(errors);
    }

    let data = item_data.into_inner();

    let image_bytes: Vec<u8> = if let Some(encoded) = data.item_image {
        match base64::engine::general_purpose::STANDARD.decode(encoded.trim()) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to decode image data: {:?}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let query = r#"
        INSERT INTO dbo.YXShopItemMap
            (ItemMain, ItemSub, ItemName,
            ItemPrice, ItemCategory, ItemDescription,
            ItemDiscount, ItemDiscountType, ItemImage, ItemStatus,
            ItemStock, PurchaseLimitPerAccount, PurchaseLimitType)
        VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13)
    "#;

    let mut client = match db.shop_db.pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("❌ Failed to get database connection: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Database connection error" }));
        }
    };

    match client
        .execute(
            query,
            &[
                &data.item_main,
                &data.item_sub,
                &data.item_name,
                &data.item_price,
                &data.item_category,
                &data.item_description,
                &data.item_discount,
                &data.item_discount_type,
                &image_bytes,
                &data.item_status,
                &data.item_stock,
                &data.item_purchase_limit,
                &data.item_purchase_limit_type
            ],
        )
        .await
    {
        Ok(_) => {
            let mut cache = states.shop_data.lock().await;
            match db.shop_db.get_shop_data().await {
                Ok(updated_data) => {
                    let shop_data: Vec<ShopDataResponse> = updated_data
                        .into_iter()
                        .map(|item| item.to_response())
                        .filter_map(Result::ok)
                        .collect();

                    cache.insert(0, shop_data);
                    tracing::info!("✅ Cache successfully updated after item addition.");
                }
                Err(err) => {
                    tracing::error!("❌ Failed to refresh cache after adding item: {:?}", err);
                }
            }
            HttpResponse::Ok().json(json!({ "message": "Item added successfully" }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to add item data: {:?}", e);
            HttpResponse::InternalServerError().json(json!({ "error": "Failed to add item data" }))
        }
    }
}

#[post("/migrate-images")]
pub async fn migrate_images(db: web::Data<Arc<DBState>>) -> impl Responder {
    match db.shop_db.get_shop_data().await {
        Ok(data) => {
            let filtered_data: Vec<ShopMigrate> = data
                .into_iter()
                .map(|item| ShopMigrate {
                    product_num: item.product_num,
                    item_image: item.item_image
                })
                .collect();

            let query = r#"
                UPDATE dbo.YXShopItemMap
                SET NewItemImage = @P1
                WHERE ProductNum = @P2
            "#;

            let mut client = match db.shop_db.pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("❌ Failed to get database connection: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .json(json!({ "error": "Database connection error" }));
                }
            };

            for item in filtered_data {
                let url = upload_image_to_r2(&db.r2_client, &db.r2_config, "shop", item.item_image).await.unwrap();

                if let Err(e) = client
                    .execute(query, &[&url, &item.product_num])
                    .await {
                        tracing::error!("❌ Failed to update item {}: {:?}", item.product_num, e);
                        return HttpResponse::InternalServerError()
                            .json(json!({ "error": "Failed to update item data" }));
                    }
            }

            HttpResponse::Ok().json(true)
        }
        Err(err) => {
            log::error!("❌ Failed to fetch shop data from database: {:?}", err);
            HttpResponse::InternalServerError().json(json!({ "error": "Failed to load shop data" }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_shop_items)
        .service(get_shop_item_by_id)
        .service(edit_shop_item_by_id)
        .service(add_item_data)
        .service(migrate_images);
}
