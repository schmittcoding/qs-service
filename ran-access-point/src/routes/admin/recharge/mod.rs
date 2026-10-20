use std::{ops::Deref, sync::Arc};

use actix_web::{HttpRequest, get, post, put, web, HttpResponse, Responder};
use ran_db_core::webmanager::transaction::WebTransactionTrait;
use ran_models::{
    admin::TransactionFilterParams,
    payments::{AddGatewayInfo, AddRechargeInfo, RechargeInfo, WebTransaction, WebTransactionResponse},
    PaginationParams,
};
use serde::Deserialize;
use serde_json::json;
use serde_qs::Config;
use uuid::Uuid;

use crate::{constants::http_messages::http_messages::error, database::DBState};
use ran_db_core::webmanager::user::cache::invalidate_profile;

// Constants for performance tuning
const MAX_QUERY_LIMIT: i64 = 100;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(add_recharge_info)
        .service(get_recharge_info)
        .service(get_gateway_info)
        .service(add_gateway_info)
        .service(update_recharge_info)
        .service(update_transaction)
        .service(recharge_logs)
        .service(update_gateway_info)
        .service(update_recharge_info)
        .service(get_transactions);
}

#[derive(Deserialize)]
struct UpdateStatusParams {
    status: i16,
}

/// Get all recharge logs with pagination and filtering support
///
/// # Query Parameters
/// * `page` - Page number (1-indexed, defaults to 1)
/// * `limit` - Number of items per page (defaults to 10, max 100)
/// * `status` - Filter by transaction status (0: Pending, 1: Approved, 2: Declined)
/// * `category` - Filter by transaction type (1: GCash, 2: PayPal)
///
/// # Response Format
/// ```json
/// {
///   "page": 1,
///   "limit": 10,
///   "total_pages": 5,
///   "total_items": 42,
///   "data": [
///     {
///       "transaction_id": "...",
///       "user_num": 123,
///       "user_name": "...",
///       // other transaction fields
///     }
///   ]
/// }
/// ```
#[get("/recharge-logs")]
async fn recharge_logs(
    db: web::Data<Arc<crate::database::DBState>>,
    pagination: web::Query<PaginationParams>,
) -> impl Responder {
    // Extract and validate pagination parameters
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10).min(MAX_QUERY_LIMIT as usize);

    // Log the filters if provided
    if let Some(status) = pagination.status {
        log::debug!("Filtering by status: {}", status);
    }
    if let Some(category) = pagination.category {
        log::debug!("Filtering by transaction type (category): {}", category);
    }

    // Decide which query to run based on the presence of the status filter
    let result = if let Some(status) = pagination.status {
        db.web_db.get_recharge_logs_filtered(status).await
    } else {
        db.web_db.get_recharge_logs_all().await
    };

    match result {
        Ok(rows) => {
            // Convert rows to WebTransaction objects
            let all_transactions: Vec<WebTransaction> = rows
                .into_iter()
                .map(|row| WebTransaction::from_row(&row))
                .collect();

            // Filter by transaction_type (category) if provided
            let filtered_transactions: Vec<&WebTransaction> = all_transactions
                .iter()
                .filter(|item| {
                    // Status is already filtered at DB level if provided
                    let type_ok = pagination
                        .category
                        .map(|t| item.transaction_type == t)
                        .unwrap_or(true);

                    let search_ok = pagination
                        .search
                        .as_ref()
                        .map(|t| {
                            item.user_name.contains(t) ||
                            &item.transaction_id.to_string() == t ||
                            item.reference_code.as_ref().map_or(false, |code| code == t)
                        })
                        .unwrap_or(true);

                    type_ok && search_ok
                })
                .collect();

            let total_count = filtered_transactions.len();
            let total_pages = if total_count == 0 {
                1
            } else {
                (total_count as f64 / limit as f64).ceil() as usize
            };

            // Calculate pagination bounds
            let start = (page - 1) * limit;
            let end = std::cmp::min(start + limit, total_count);

            // Create a paginated response with filtered transactions
            let paginated_responses: Vec<WebTransactionResponse> = if start < total_count {
                filtered_transactions[start..end]
                    .iter()
                    .map(|tx| WebTransactionResponse::from(*tx))
                    .collect()
            } else {
                Vec::new()
            };

            // Match the format of get-news endpoint - pagination info at root level
            HttpResponse::Ok().json(json!({
                "page": page,
                "limit": limit,
                "total_pages": total_pages,
                "total_items": total_count,
                "data": paginated_responses
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch recharge logs: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge logs" }))
        }
    }
}

#[get("")]
async fn get_transactions(
    db: web::Data<Arc<DBState>>,
    req: HttpRequest,
) -> impl Responder {
    let qs = req.query_string();
    let config = Config::new(5, false);

    let params: TransactionFilterParams = match config.deserialize_str(qs) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid query: {}", e)),
    };

    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let status = params.status.as_ref().filter(|v| !v.is_empty()).cloned();
    let r#type = params.method.as_ref().filter(|v| !v.is_empty()).cloned();
    let search: Option<String> = params.search.clone();

    match db.web_db.get_transactions(page, limit, r#type, status, search).await {
        Ok(res) => {
            let data: Vec<WebTransactionResponse> = res.data
                .into_iter()
                .map(|row| WebTransactionResponse::from_transactions(&WebTransaction::from_row(&row)))
                .collect();

            HttpResponse::Ok().json(
                json!({
                    "data": data,
                    "page": page,
                    "limit": limit,
                    "total_items": res.total_records
                })
            )
        }
        Err(e) => {
            log::error!("❌ Failed to get recharge transactions list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[put("/update-transaction/{id}")]
async fn update_transaction(
    path: web::Path<Uuid>,
    db: web::Data<Arc<crate::database::DBState>>,
    data: web::Query<UpdateStatusParams>,
) -> impl Responder {
    let trans_id = path.into_inner();

    // TODO : Add HWID of Administrator
    match db
        .web_db
        .update_transaction_status(trans_id, data.status)
        .await
    {
        Ok(user_num) => {
            invalidate_profile(user_num);
            let webhook_url = std::env::var("DISCORD_TRANSACTION_WEBHOOK").ok();
            tokio::spawn(async move {
                let webhook_url = match webhook_url {
                    Some(url) => url,
                    None => {
                        tracing::warn!("DISCORD_TRANSACTION_WEBHOOK not set, skipping notification");
                        return;
                    }
                };
                let client = reqwest::Client::new();
                let embed = json!({
                    "title": "Approve/Denied Notification",
                    "description": format!(
                        "Transaction  {} has been processed",
                        trans_id
                    ),
                    "color": 0x444444,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "footer": {
                        "text": "RAN Online GS"
                    }
                });
                let payload = json!({
                    "username": "RAN Online GS",
                    "embeds": [ embed ]
                });

                if let Err(err) = client.post(&webhook_url).json(&payload).send().await {
                    tracing::error!("Error sending Discord webhook: {:?}", err);
                }
            });
            HttpResponse::Ok().json(json!({ "message": "Transaction updated successfully" }))
        }
        Err(err) => {
            log::error!("Failed to update transaction: {:?}", err);
            HttpResponse::InternalServerError().json(json!({ "error": err.to_string() }))
        }
    }
}

#[put("/update-recharge-info/{id}")]
async fn update_recharge_info(
    path: web::Path<Uuid>,
    db: web::Data<Arc<crate::database::DBState>>,
    data: web::Json<AddRechargeInfo>,
) -> impl Responder {
    let recharge_id = path.into_inner();
    let update = data.into_inner();

    match db
        .web_db
        .update_recharge_info_by_id(recharge_id, update)
        .await
    {
        Ok(recharge_info) => HttpResponse::Ok().json(recharge_info), // ✅ Return the data
        Err(e) => {
            log::error!("❌ Failed to fetch recharge info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge info" }))
        }
    }
}

#[put("/update-gateway-info/{id}")]
async fn update_gateway_info(
    path: web::Path<Uuid>,
    db: web::Data<Arc<crate::database::DBState>>,
    data: web::Json<AddGatewayInfo>,
) -> impl Responder {
    let gateway_id = path.into_inner();
    let update = data.into_inner();

    match db
        .web_db
        .update_gateway_info_by_id(gateway_id, update)
        .await
    {
        Ok(recharge_info) => HttpResponse::Ok().json(recharge_info), // ✅ Return the data
        Err(e) => {
            log::error!("❌ Failed to fetch recharge info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge info" }))
        }
    }
}

#[get("/get-gateway-info")]
async fn get_gateway_info(db: web::Data<Arc<crate::database::DBState>>) -> impl Responder {
    match db.web_db.get_gateway_info().await {
        Ok(gateways) => HttpResponse::Ok().json(gateways), // ✅ Return the data
        Err(e) => {
            log::error!("❌ Failed to fetch gateway info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch gateway info" }))
        }
    }
}

#[post("/add-gateway-info")]
async fn add_gateway_info(
    db: web::Data<Arc<crate::database::DBState>>,
    gateway: web::Json<AddGatewayInfo>,
) -> impl Responder {
    let gateway_info = gateway.into_inner();
    match db.web_db.add_gateway_info(gateway_info).await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({"message": "Added new gateway info successfully" }))
        }
        Err(err) => {
            log::error!("❌ Failed to add gateway info: {:?}", err);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to add gateway info" }))
        }
    }
}

#[get("/get-recharge-info")]
async fn get_recharge_info(
    db: web::Data<Arc<crate::database::DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    let status = params.status.map(|p| p as u8);
    let r#type = params.category;

    let results = match db.web_db.get_recharge_info(status.clone(), r#type.clone()).await {
        Ok(data) => data,
        Err(err) => {
            log::error!("❌ Failed to fetch recharge info: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }));
        }
    };

    let total_items = results.len();
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

    let paginated: Vec<RechargeInfo> = results
        .into_iter()
        .skip(start)
        .take(limit)
        .collect();

    HttpResponse::Ok().json(json!({
        "data": paginated,
        "total_items": total_items
    }))
}

#[post("/add-recharge-info")]
async fn add_recharge_info(
    db: web::Data<Arc<crate::database::DBState>>,
    recharge_info: web::Json<AddRechargeInfo>,
) -> impl Responder {
    let data: AddRechargeInfo = recharge_info.into_inner();
    let query_str: &str;
    let mut params: Vec<&dyn tiberius::ToSql> = Vec::new();

    if let Some(discount) = &data.discount {
        query_str = r#"
            INSERT INTO RechargeInfo (RechargeType, RechargeAmount, RechargeDiscount, RechargePoints, RechargeStatus)
            VALUES (@P1, @P2, @P3, @P4, @P5);"#;
        params.push(&data.r#type);
        params.push(&data.amount);
        params.push(discount);
        params.push(&data.points);
        params.push(&data.status);
    } else {
        query_str = r#"
            INSERT INTO RechargeInfo (RechargeType, RechargeAmount, RechargePoints, RechargeStatus)
            VALUES (@P1, @P2, @P3, @P4);"#;
        params.push(&data.r#type);
        params.push(&data.amount);
        params.push(&data.points);
        params.push(&data.status);
    }
    let mut client = match db.web_db.pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("❌ Failed to get database connection: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Database connection error" }));
        }
    };

    match client.execute(query_str, params.deref()).await {
        Ok(rows_updated) => {
            if rows_updated.total() > 0 {
                HttpResponse::Ok()
                    .json(json!({ "message": "Added new recharge info successfully" }))
            } else {
                HttpResponse::NotFound().json(json!({ "error": "recharge info not found" }))
            }
        }
        Err(e) => {
            log::error!("❌ Failed to add recharge info: {:?}", e);
            HttpResponse::InternalServerError().json(json!({ "error": "Failed to update status" }))
        }
    }
}
