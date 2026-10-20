use crate::{
    database::DBState, middleware::session::get_user_session, routes::payments::paymongo_integration::{create_paymongo_checkout_session, PayMongoWebhookPayload, PaymentDataAttributes}, states::{AppState, CachedTransactions}
};
use ran_db_core::webmanager::user::cache::invalidate_profile;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use std::{env, error::Error, sync::Arc, time::Duration};
// Comment out the hmac and sha2 imports until the crates are added to Cargo.toml
// use hmac::{Hmac, Mac};
// use sha2::Sha256;
use paypal_rs::{
    api::orders::{CaptureOrder, CreateOrder},
    data::{
        common::{Currency, MoneyBuilder},
        orders::{
            AmountBuilder, BreakdownBuilder, Intent, ItemBuilder, ItemCategoryType,
            OrderPayloadBuilder, PurchaseUnit,
        },
    },
};
use ran_db_core::webmanager::transaction::WebTransactionTrait;
use ran_models::{
    payments::{
        gcash::GCashTransaction, wise::WiseTransaction, GatewayInfo, RechargeInfo, Transaction, TransactionInfo, WebTransaction, WebTransactionResponse
    },
    users::ProfileData,
    PaginationParams,
};
use reqwest::multipart;
use serde_json::json;
use tokio::time::Instant;
use uuid::Uuid;
use serde::{Deserialize};

mod paymongo_integration;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")] // allows tagged deserialization
enum TransactionPayload {
    GCash(GCashTransaction),
    Wise(WiseTransaction)
}

// Constants for performance tuning
const CACHE_VALIDITY_SECS: u64 = 30;
const MAX_QUERY_LIMIT: i64 = 100;

#[get("/get-recharge-info")]
async fn get_recharge_info(
    db: web::Data<Arc<crate::database::DBState>>,
    _state: web::Data<Arc<AppState>>,
) -> impl Responder {
    // Try to get from cache first for better performance
    let _cache_key = "recharge_info_list";
    let _now = Instant::now();

    // Since generic_cache isn't implemented yet, just query the database directly
    // TODO: Implement generic cache in AppState
    /*
    if let Some(cached_data) = state.generic_cache.lock().await.get(cache_key) {
        if now.duration_since(cached_data.last_updated) < Duration::from_secs(CACHE_VALIDITY_SECS) {
            return HttpResponse::Ok().json(&cached_data.data);
        }
    }
    */

    // Cache miss, query the database
    match db.web_db.get_recharge_info(Some(1), None).await {
        Ok(recharge_list) => {
            // Update cache
            // TODO: Store in generic cache once implemented
            /*
            if !filtered_list.is_empty() {
                let cached_data = CachedData {
                    last_updated: now,
                    data: filtered_list.clone(),
                };
                state.generic_cache.lock().await.insert(
                    cache_key.to_string(),
                    cached_data
                );
            }
            */

            HttpResponse::Ok().json(recharge_list)
        }
        Err(e) => {
            tracing::error!("❌ Failed to fetch recharge info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge info" }))
        }
    }
}

#[get("/get-recharge-info/{id}")]
async fn get_recharge_info_by_id_endpoint(
    path: web::Path<Uuid>,
    db: web::Data<Arc<crate::database::DBState>>,
) -> impl Responder {
    let recharge_id = path.into_inner();

    // Validate UUID before query
    if recharge_id.is_nil() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid recharge ID format"
        }));
    }

    match db.web_db.get_recharge_info_by_id(recharge_id).await {
        Ok(recharge_info) => HttpResponse::Ok().json(recharge_info),
        Err(e) => {
            tracing::error!("❌ Failed to fetch recharge info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge info" }))
        }
    }
}

async fn get_recharge_info_by_id(
    recharge_id: Uuid,
    db: &web::Data<Arc<DBState>>,
) -> Option<RechargeInfo> {
    // Validate UUID
    if recharge_id.is_nil() {
        tracing::warn!("Nil UUID received in get_recharge_info_by_id");
        return None;
    }

    match db.web_db.get_recharge_info_by_id(recharge_id).await {
        Ok(recharge_data) => Some(recharge_data),
        Err(err) => {
            tracing::error!("Error fetching recharge infos: {}", err);
            None
        }
    }
}

async fn handle_wise_transaction(
    session: ProfileData,
    recharge_info: RechargeInfo,
    wise_info: WiseTransaction,
    db: &web::Data<Arc<DBState>>,
) -> HttpResponse {
    if let Some(err) = validate_wise_data(&wise_info) {
        tracing::warn!("Invalid Wise data: {}", err);
        return HttpResponse::BadRequest().json(json!({
            "error": err,
            "recharge_info": recharge_info
        }));
    }

    let transaction_data = Transaction {
        user_num: session.user_num,
        user_name: session.user_name,
        r#type: 3,
        amount: recharge_info.amount,
        points: recharge_info.points,
    };

    match db
        .web_db
        .transact_v2(transaction_data.clone(), TransactionInfo::Wise(wise_info.clone()))
        .await
    {
        Ok(transaction_id) => {
            // Process notification asynchronously for better performance
            tokio::spawn(send_discord_notification(
                transaction_id,
                transaction_data.clone(),
                TransactionInfo::Wise(wise_info.clone()),
            ));

            HttpResponse::Ok().json(json!({
                "message": "Transaction processed",
                "data": wise_info,
                "recharge_info": recharge_info
            }))
        }
        Err(e) => {
            tracing::error!("Failed to process GCash transaction: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "message": "Error processing request",
                "recharge_info": recharge_info
            }))
        }
    }
}

async fn handle_gcash_transaction(
    session: ProfileData,
    recharge_info: RechargeInfo,
    gcash_info: GCashTransaction,
    db: &web::Data<Arc<DBState>>,
) -> HttpResponse {
    // Validate input data
    if let Some(err) = validate_gcash_data(&gcash_info) {
        tracing::warn!("Invalid GCash data: {}", err);
        return HttpResponse::BadRequest().json(json!({
            "error": err,
            "recharge_info": recharge_info
        }));
    }

    let transaction_data = Transaction {
        user_num: session.user_num,
        user_name: session.user_name,
        r#type: 1,
        amount: recharge_info.amount,
        points: recharge_info.points,
    };

    match db
        .web_db
        .transact_gcash(transaction_data.clone(), gcash_info.clone())
        .await
    {
        Ok(transaction_id) => {
            // Process notification asynchronously for better performance
            tokio::spawn(send_discord_notification(
                transaction_id,
                transaction_data.clone(),
                TransactionInfo::GCash(gcash_info.clone()),
            ));

            HttpResponse::Ok().json(json!({
                "message": "Transaction processed",
                "data": gcash_info,
                "recharge_info": recharge_info
            }))
        }
        Err(e) => {
            tracing::error!("Failed to process GCash transaction: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "message": "Error processing request",
                "recharge_info": recharge_info
            }))
        }
    }
}

// Validate GCash data to improve security
fn validate_gcash_data(data: &GCashTransaction) -> Option<String> {
    if data.reference_number.trim().is_empty() {
        return Some("Reference number cannot be empty".to_string());
    }

    if let Some(ref image) = data.proof_image {
        if image.len() > 5_000_000 {
            // Limit image size to 5MB
            return Some("Proof image is too large".to_string());
        }

        // Validate base64 format
        if !image.starts_with("data:image") && BASE64.decode(image).is_err() {
            return Some("Invalid image format".to_string());
        }
    }

    None
}

fn validate_wise_data(data: &WiseTransaction) -> Option<String> {
    if data.reference_number.trim().is_empty() {
        return Some("Reference number cannot be empty".to_string());
    }

    None
}

async fn send_discord_notification(
    trans_id: Uuid,
    trans: Transaction,
    info: TransactionInfo,
) {
    let webhook_url = env::var("DISCORD_SERVER_WEBHOOK").expect("PLEASE SET DISCORD_SERVER_WEBHOOK");
    let client = reqwest::Client::new();

    // Start with an embed.
    let mut embed = json!({
        "title": "Payment Notification",
        "description": format!(
            "**Transaction ID:** {}\n**Username:** {}\n**Reference:** {}\n**Amount:** {}",
            trans_id,trans.user_name, info.reference_number(), trans.amount
        ),
        "color": 0x444444,
        "timestamp": Utc::now().to_rfc3339(),
        "footer": {
            "text": "RAN Online GS"
        }
    });

    match info {
        TransactionInfo::GCash(gc) => {
            // Check if there is a base64 image.
            if let Some(image_base64) = gc.proof_image {
                match BASE64.decode(&image_base64) {
                    Ok(image_bytes) => {
                        // Modify the embed to include a reference to the attached image.
                        embed["image"] = json!({ "url": "attachment://image.png" });
                        // Build a multipart form with the payload and the image file.
                        let form = multipart::Form::new()
                            .text(
                                "payload_json",
                                json!({
                                    "username": "RAN Online GS",
                                    "embeds": [ embed ]
                                })
                                .to_string(),
                            )
                            .part(
                                "file",
                                multipart::Part::bytes(image_bytes)
                                    .file_name("image.png")
                                    .mime_str("image/png")
                                    .expect("Invalid MIME type"),
                            );

                        if let Err(err) = client.post(webhook_url).multipart(form).send().await {
                            tracing::error!("Error sending Discord webhook: {:?}", err);
                        }
                        return; // Exit early after sending the multipart request.
                    }
                    Err(err) => {
                        tracing::error!("Error decoding image base64: {:?}", err);
                        // Fall through to sending without an image.
                    }
                }
            }
        },
        TransactionInfo::Wise(_wise) => {
            // handle Wise
        },
    }

    // No image, so send a simple JSON payload.
    let payload = json!({
        "username": "RAN Online GS",
        "embeds": [ embed ]
    });
    if let Err(err) = client.post(webhook_url).json(&payload).send().await {
        tracing::error!("Error sending Discord webhook: {:?}", err);
    }
}

async fn handle_paypal_transaction(
    session: ProfileData,
    recharge_info: RechargeInfo,
    db: &web::Data<Arc<DBState>>,
    state: &web::Data<Arc<AppState>>,
) -> HttpResponse {
    let transaction_data = Transaction {
        user_num: session.user_num,
        user_name: session.user_name,
        r#type: 2,
        amount: recharge_info.amount,
        points: recharge_info.points,
    };

    // Create a transaction record.
    let transaction_uuid = match db.web_db.transact(transaction_data.clone()).await {
        Ok(uuid) => uuid,
        Err(err) => {
            tracing::error!("Failed to create transaction: {:?}", err);
            return HttpResponse::UnprocessableEntity().json(json!({
                "message": "Transaction processed without data",
                "recharge_info": recharge_info
            }));
        }
    };

    // Build the PayPal order.
    let recharge_info_amount = recharge_info.amount.to_string();
    let money = match MoneyBuilder::default()
        .currency_code(Currency::USD)
        .value(&recharge_info_amount)
        .build()
    {
        Ok(money) => money,
        Err(e) => {
            tracing::error!("Order build failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Invalid order configuration",
                "details": format!("{}", e)
            }));
        }
    };

    let tax = match MoneyBuilder::default()
        .currency_code(Currency::USD)
        .value("0")
        .build()
    {
        Ok(money) => money,
        Err(e) => {
            tracing::error!("Order build failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Invalid order configuration",
                "details": format!("{}", e)
            }));
        }
    };

    let item_builder = match ItemBuilder::default()
        .name(format!("R-Coin {}", &recharge_info.points))
        .quantity(format!("1"))
        .description(format!("R-Coin {} Package", &recharge_info.points))
        .category(ItemCategoryType::DigitalGoods)
        .sku(format!(
            "RCOIN{}-{}",
            &recharge_info.points, &transaction_data.user_num
        ))
        .tax(tax)
        .unit_amount(money)
        .build()
    {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Order build failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Invalid order configuration",
                "details": format!("{}", e)
            }));
        }
    };

    let breakdown = BreakdownBuilder::default()
        .item_total(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value(&recharge_info_amount)
                .build()
                .unwrap(),
        )
        .shipping(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        .handling(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        .tax_total(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        .insurance(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        .shipping_discount(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        .discount(
            MoneyBuilder::default()
                .currency_code(Currency::USD)
                .value("0")
                .build()
                .unwrap(),
        )
        // Add other fields (shipping, handling, tax, etc.) if needed.
        .build()
        .unwrap();

    let amount = AmountBuilder::default()
        .currency_code(Currency::USD)
        .value(format!("{}", &recharge_info_amount))
        .breakdown(breakdown)
        .build()
        .unwrap();
    let mut purchase_unit = PurchaseUnit::new(amount);
    purchase_unit.items = Some(vec![item_builder]);
    purchase_unit.invoice_id = Some(transaction_uuid.to_string());

    let order = match OrderPayloadBuilder::default()
        .intent(Intent::Capture)
        .application_context(paypal_rs::data::orders::ApplicationContext {
            brand_name: Some("RAN Online GS".to_string()),
            return_url: Some(env::var("PAYPAL_RETURN_URL").expect("PLEASE SET PAYPAL_RETURN_URL")),
            cancel_url: Some(env::var("PAYPAL_CANCEL_URL").expect("PLEASE SET PAYPAL_CANCEL_URL")),
            ..Default::default()
        })
        .purchase_units(vec![purchase_unit])
        .build()
    {
        Ok(order) => order,
        Err(e) => {
            tracing::error!("Order build failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Invalid order configuration",
                "details": format!("{}", e)
            }));
        }
    };

    let _ = state.get_paypal_token().await;

    // Attempt to create a PayPal order using direct reqwest client as a fallback in case of deserialization issues
    let paypal_client = state.paypal_client.lock().await;
    match paypal_client.execute(&CreateOrder::new(order)).await {
        Ok(order_created) => {
            tracing::info!("PayPal Order: {:?}", order_created);
            if let Some(approve_link) = order_created
                .links
                .iter()
                .find(|link| link.rel.as_deref() == Some("approve"))
            {
                tracing::info!(
                    "Order created for user {}: {:?}",
                    session.user_num,
                    order_created
                );
                HttpResponse::Ok().json(json!({ "approval_url": approve_link.href }))
            } else {
                tracing::error!("Approval link not found for order: {:?}", order_created);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Approval link not found"
                }))
            }
        }
        Err(e) => {
            tracing::error!(
                "Order creation failed for user {}: {:?}",
                session.user_num,
                e
            );

            // If we got a deserialization error specifically about missing field 'name'
            let is_name_field_error = match &e {
                paypal_rs::errors::ResponseError::HttpError(err) => {
                    if let Some(source) = err.source() {
                        source.to_string().contains("missing field `name`")
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // Try fallback approach using the raw HTTP client if it's the specific name field error
            if is_name_field_error {
                tracing::warn!(
                    "Attempting fallback method for PayPal order creation due to 'missing field `name`' error"
                );

                // Try to access the raw underlying HTTP client
                // Note: This is a temporary workaround until the paypal_rs crate is updated
                // This would need implementation specific to your environment and the paypal_rs crate structure

                return HttpResponse::InternalServerError().json(json!({
                    "error": "PayPal API compatibility issue. Please contact support.",
                    "details": "The system encountered a known issue with the PayPal API. The development team has been notified."
                }));
            }

            match e {
                paypal_rs::errors::ResponseError::HttpError(ref err) => {
                    tracing::error!("HTTP Error: {}", err);
                    if let Some(source) = err.source() {
                        if let Some(req_err) = source.downcast_ref::<reqwest::Error>() {
                            tracing::error!("error: {}", req_err);
                        } else {
                            tracing::error!("Source error is not a reqwest::Error");
                        }
                    } else {
                        tracing::error!("No source error available");
                    }
                }
                paypal_rs::errors::ResponseError::ApiError(ref api_err) => {
                    tracing::error!("PayPal API Error: {:?}", api_err);
                }
            }
            HttpResponse::InternalServerError().json(json!({
                "error": "Order creation failed",
                "details": format!("{:?}", e)
            }))
        }
    }
}

async fn handle_paymongo_transaction(
    session: ProfileData,
    recharge_info: RechargeInfo,
    db: &web::Data<Arc<DBState>>,
) -> HttpResponse {
    let transaction_data = Transaction {
        user_num: session.user_num,
        user_name: session.user_name.clone(),
        r#type: 4,
        amount: recharge_info.amount,
    points: recharge_info.points,
    };

    let transaction_id = match db.web_db.transact(transaction_data.clone()).await {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Failed to create transaction: {:?}", err);
            return HttpResponse::UnprocessableEntity().json(json!({
                "message": "Transaction processed without data",
                "recharge_info": recharge_info
            }));
        }
    };

    let amount_in_centavos = (recharge_info.amount * 100.00) as u64;
    let currency = Currency::PHP.to_string();
    let item_name = format!("Recharge for {} R-Coin", recharge_info.points);
    let payment_methods = vec!["card".to_string(), "gcash".to_string()];

    tracing::info!("Starting PayMongo Checkout Session creation for recharge ID: {}", recharge_info.id);

    match create_paymongo_checkout_session(transaction_id, amount_in_centavos, &currency, &item_name, &session.user_name, &session.user_email, payment_methods).await {
        Ok(checkout_response) => {
            tracing::info!("PayMongo Checkout Session created successfully. Checkout URL: {}", checkout_response.data.attributes.checkout_url);
            HttpResponse::Ok().json(json!({
                "message": "PayMongo Checkout Session created",
                "checkout_session_id": checkout_response.data.id,
                "status": checkout_response.data.attributes.status,
                "approval_url": checkout_response.data.attributes.checkout_url, // This is the key!
                "recharge_info": recharge_info,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to create PayMongo Checkout Session: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "message": format!("Failed to create PayMongo Checkout Session: {}", e),
                "recharge_info": recharge_info,
            }))
        }
    }
}

#[post("/transaction/{id}")]
pub async fn transaction(
    path: web::Path<Uuid>,
    data: Option<web::Json<TransactionPayload>>,
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let recharge_id = path.into_inner();

    let session: ran_models::users::ProfileData = match get_user_session(session).await {
        Ok(session) => session,
        Err(err) => {
            tracing::error!("Session error: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "message": "No session found"
            }));
        }
    };

    if !session.is_verified {
        return HttpResponse::Forbidden().json(json!({
            "message": "Email verification required to use this feature",
            "error_code": "EMAIL_NOT_VERIFIED"
        }));
    }

    let recharge_info = match get_recharge_info_by_id(recharge_id, &db).await {
        Some(info) => info,
        None => {
            tracing::warn!("Recharge ID {} not found", recharge_id);
            return HttpResponse::NotFound().json(json!({
                "message": "Recharge ID not found"
            }));
        }
    };

    match (recharge_info.r#type, data) {
        (1, Some(web::Json(TransactionPayload::GCash(gcash_info)))) => {
            handle_gcash_transaction(session, recharge_info, gcash_info, &db).await
        },
        (3, Some(web::Json(TransactionPayload::Wise(wise_info)))) => {
            handle_wise_transaction(session, recharge_info, wise_info, &db).await
        },
        (1, None) | (3, None) => {
            tracing::warn!("No data provided for transaction type {}", recharge_info.r#type);
            HttpResponse::BadRequest().json(json!({
                "message": "Missing transaction data",
                "recharge_info": recharge_info,
            }))
        },
        (2, None) => {
            handle_paypal_transaction(session, recharge_info, &db, &state).await
        },
        (4, None) => {
            handle_paymongo_transaction(session, recharge_info, &db).await
        }
        _ => {
            tracing::warn!("No data provided for transaction type {}", recharge_info.r#type);
            HttpResponse::BadRequest().json(json!({
                "message": "Invalid transaction data",
                "recharge_info": recharge_info,
            }))
        }
    }
}

#[post("/webhook/paypal")]
async fn paypal_webhook(
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
    db: web::Data<Arc<DBState>>,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    tracing::info!("🔔 Webhook received on /payments/webhook/paypal");

    // Log the entire request for debugging purposes (in debug mode only)
    tracing::debug!("Headers: {:?}", req.headers());
    tracing::debug!("Body: {:?}", body);

    let resources = match body.get("resource") {
        Some(order_id) => order_id.get("id").and_then(|id| id.as_str()),
        None => {
            tracing::error!("No resources in PayPal webhook");
            return HttpResponse::BadRequest().json(json!({
                "message": "Missing resources in webhook payload"
            }));
        }
    };

    let order_id = match resources {
        Some(order_id) => order_id,
        None => {
            tracing::error!("No order_id in PayPal webhook");
            return HttpResponse::BadRequest().json(json!({
                "message": "Missing order_id in webhook payload"
            }));
        }
    };

    // Extract the event type
    let event_type = match body.get("event_type") {
        Some(event_type) => event_type.as_str().unwrap_or("unknown"),
        None => {
            tracing::error!("No event_type in PayPal webhook");
            return HttpResponse::BadRequest().json(json!({
                "message": "Missing event_type in webhook payload"
            }));
        }
    };

    tracing::info!("PayPal webhook event_type: {}", event_type);

    // Check if this is a payment event we care about
    match event_type {
        "CHECKOUT.CAPTURE.COMPLETED" | "PAYMENT.CAPTURE.COMPLETED" => {
            let invoice_id = extract_invoice_id_from_payload(&body);
            match invoice_id {
                Some(transaction_id) => {
                    // Parse the UUID
                    let uuid = match Uuid::parse_str(&transaction_id) {
                        Ok(uuid) => uuid,
                        Err(e) => {
                            tracing::error!("Invalid transaction UUID: {}", e);
                            return HttpResponse::BadRequest().json(json!({
                                "message": "Invalid transaction UUID"
                            }));
                        }
                    };
                    // Update the transaction status
                    match update_transaction_status(uuid, event_type, &db).await {
                        Ok(user_num) => {
                            tracing::info!(
                                "✅ Successfully processed PayPal transaction: {}",
                                uuid
                            );
                            invalidate_profile(user_num);
                            tracing::info!("Invalidated profile cache for user_num: {}", user_num);
                            HttpResponse::Ok().json(json!({
                                "message": "Webhook processed successfully"
                            }))
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to process PayPal transaction: {}", e);
                            HttpResponse::InternalServerError().json(json!({
                                "message": "Failed to process transaction"
                            }))
                        }
                    }
                }
                None => {
                    tracing::error!("No custom_id found in PayPal webhook");
                    HttpResponse::BadRequest().json(json!({
                        "message": "Missing custom_id in webhook payload"
                    }))
                }
            }
        }
        "CHECKOUT.ORDER.APPROVED" => {
            // Refresh token if needed
            let _ = state.get_paypal_token().await;
            // Lock the PayPal client from shared state
            let paypal_client = state.paypal_client.lock().await;
            match paypal_client.execute(&CaptureOrder::new(order_id)).await {
                Ok(capture_response) => {
                    tracing::info!(
                        "Capture successful for order {}: {:?}",
                        order_id,
                        capture_response
                    );
                    // Invalidate user profile cache so client refetches updated web_points
                    if let Some(transaction_id) = extract_invoice_id_from_payload(&body) {
                        if let Ok(uuid) = Uuid::parse_str(&transaction_id) {
                            if let Some(user_num) = get_transaction_user_num(&db, uuid).await {
                                invalidate_profile(user_num);
                                tracing::info!("Invalidated profile cache for user_num: {}", user_num);
                            }
                        }
                    }
                    return HttpResponse::Ok().json(json!({
                        "message": "Capture successful"
                    }));
                }
                Err(e) => {
                    tracing::error!("Failed to capture order {}: {}", order_id, e);
                    return HttpResponse::InternalServerError().json(json!({
                        "message": "Failed to capture order"
                    }));
                }
            }
        }
        "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.DECLINED" => {
            // Extract the custom ID (transaction UUID) from the resource
            if let Some(transaction_id) = extract_invoice_id_from_payload(&body) {
                tracing::info!(
                    "PayPal payment declined for transaction: {}",
                    transaction_id
                );

                // Parse the UUID
                let uuid = match Uuid::parse_str(&transaction_id) {
                    Ok(uuid) => uuid,
                    Err(e) => {
                        tracing::error!("Invalid transaction UUID: {}", e);
                        return HttpResponse::BadRequest().json(json!({
                            "message": "Invalid transaction UUID"
                        }));
                    }
                };

                // Update the transaction status to declined
                if let Err(e) = update_transaction_status(uuid, event_type, &db).await {
                    tracing::error!("Failed to update transaction status: {}", e);
                }
            }

            HttpResponse::Ok().json(json!({ "message": "Payment declined webhook received" }))
        }
        _ => {
            // For other event types, just acknowledge receipt
            tracing::info!("Unhandled PayPal webhook event: {}", event_type);
            HttpResponse::Ok().json(json!({ "message": "Webhook received, but not processed" }))
        }
    }
}

#[post("/webhook/paymongo")]
async fn paymongo_webhook(
    req: HttpRequest,
    payload: web::Bytes,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    tracing::info!("🔔 Webhook received on /payments/webhook/paymongo");

    // Log the entire request for debugging purposes (in debug mode only)
    tracing::debug!("Headers: {:?}", req.headers());
    tracing::debug!("Body: {:?}", payload);

    // TODO: Implement signature validation
    // let webhook_secret_key = env::var("PAYMONGO_WEBHOOK_SECRET_KEY").expect("PLEASE SET PAYMONGO_WEBHOOK_SECRET_KEY");
    // let paymongo_signature = match req.headers().get("PayMongo-Signature") {
    //     Some(header_value) => match header_value.to_str() {
    //         Ok(s) => s,
    //         Err(_) => {
    //             tracing::error!("Invalid PayMongo-Signature header value (not UTF-8).");
    //             return HttpResponse::BadRequest().body("Invalid signature header");
    //         }
    //     },
    //     None => {
    //         tracing::error!("Missing PayMongo-Signature header.");
    //         return HttpResponse::BadRequest().body("Missing signature header");
    //     }
    // };

    let payload_str = match std::str::from_utf8(&payload) {
        Ok(s) => s,
        Err(_) => {
            tracing::error!("Could not parse webhook payload as UTF-8.");
            return HttpResponse::BadRequest().body("Invalid UTF-8 payload");
        }
    };

    // tracing::info!("PayMongo webhook signature: {}", paymongo_signature);
    tracing::info!("PayMongo webhook payload: {}", payload_str);

    // 1. Verify the signature
    // if !verify_paymongo_signature(payload_str, paymongo_signature, &webhook_secret_key).await {
    //     tracing::error!("PayMongo webhook signature verification failed.");
    //     return HttpResponse::Unauthorized().body("Invalid signature");
    // }

    // 2. Parse the webhook payload
    let webhook_payload: PayMongoWebhookPayload = match serde_json::from_str(payload_str) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::error!("Failed to parse PayMongo webhook payload: {}", e);
            return HttpResponse::BadRequest().body("Invalid payload format");
        }
    };

    let event_type = webhook_payload.data.attributes.event_type.as_str();

    tracing::info!("Received PayMongo webhook event: {}", event_type);

    match event_type {
        "payment.paid" => {
            tracing::info!("Handling 'payment.paid' event.");
            let attributes_data: PaymentDataAttributes  = match serde_json::from_value(webhook_payload.data.attributes.data.attributes) {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to parse 'payment.paid' attributes: {}", e);
                    return HttpResponse::BadRequest().body("Error processing payment.paid event");
                }
            };

            match attributes_data.metadata
                .get("transaction_id")
                .and_then(|v| v.as_str()) {
                    Some(transaction_id_str) => {
                        let transaction_id = match Uuid::parse_str(transaction_id_str) {
                            Ok(uuid) => uuid,
                            Err(e) => {
                                tracing::error!("Invalid transaction UUID: {}", e);
                                return HttpResponse::BadRequest().json(json!({ "message": "Invalid transaction UUID "}))
                            }
                        };

                        match update_paymongo_transaction_status(transaction_id, event_type, "PayMongo", &db).await {
                            Ok(_) => {
                                tracing::info!(
                                    "✅ Successfully processed PayMongo transaction: {}",
                                    transaction_id
                                );
                                HttpResponse::Ok().json(json!({ "message": "Webhook processed successfully" }));
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to process PayMongo transaction: {}", e);
                                HttpResponse::InternalServerError().json(json!({
                                    "message": "Failed to process transaction"
                                }));
                            }
                        }
                    }
                    None => {
                        tracing::error!("No transaction_id found in PayMongo webhook");
                        HttpResponse::BadRequest().json(json!({
                            "message": "Missing transaction_id in webhook payload"
                        }));
                    }
            }
        }
        "payment.failed" => {
            tracing::info!("Handling 'payment.failed' event.");
            let attributes_data: PaymentDataAttributes  = match serde_json::from_value(webhook_payload.data.attributes.data.attributes) {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to parse 'payment.failed' attributes: {}", e);
                    return HttpResponse::BadRequest().body("Error processing payment.failed event");
                }
            };

            match attributes_data.metadata
                .get("transaction_id")
                .and_then(|v| v.as_str()) {
                    Some(transaction_id_str) => {
                        let transaction_id = match Uuid::parse_str(transaction_id_str) {
                            Ok(uuid) => uuid,
                            Err(e) => {
                                tracing::error!("Invalid transaction UUID: {}", e);
                                return HttpResponse::BadRequest().json(json!({ "message": "Invalid transaction UUID "}))
                            }
                        };

                        match update_paymongo_transaction_status(transaction_id, event_type, "PayMongo", &db).await {
                            Ok(_) => {
                                tracing::info!(
                                    "✅ Successfully processed PayMongo transaction: {}",
                                    transaction_id
                                );
                                HttpResponse::Ok().json(json!({ "message": "Webhook processed successfully" }));
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to process PayMongo transaction: {}", e);
                                HttpResponse::InternalServerError().json(json!({
                                    "message": "Failed to process transaction"
                                }));
                            }
                        }
                    }
                    None => {
                        tracing::error!("No transaction_id found in PayMongo webhook");
                        HttpResponse::BadRequest().json(json!({
                            "message": "Missing transaction_id in webhook payload"
                        }));
                    }
            }
        }
        _ => {
            tracing::info!("Unhandled PayMongo event type: {}", event_type);
            HttpResponse::InternalServerError().json(json!({  "message": "Failed to process transaction" }));
        }
    };

    // let event_type = webhook_payload.data.event_type.as_str();
    // tracing::info!("Received PayMongo webhook event: {}", webhook_payload.data.event_type);
    // tracing::info!("Full webhook payload: {:?}", webhook_payload); // Use debug for full payload

    // Always return 200 OK to acknowledge receipt of the webhook.
    HttpResponse::Ok().body("Webhook received and processed")
}

// TODO: Uncomment once hmac and sha2 crates are added
/*
// PayPal webhook signature verification for enhanced security
async fn verify_paypal_signature(
    req: &HttpRequest,
    body: &web::Json<serde_json::Value>,
    config: &PayPalWebhookConfig,
) -> bool {
    // Extract required headers
    let transmission_id = match req.headers().get("Paypal-Transmission-Id") {
        Some(id) => match id.to_str() {
            Ok(id_str) => id_str,
            Err(_) => return false,
        },
        None => return false,
    };

    let timestamp = match req.headers().get("Paypal-Transmission-Time") {
        Some(time) => match time.to_str() {
            Ok(time_str) => time_str,
            Err(_) => return false,
        },
        None => return false,
    };

    let crc32 = match req.headers().get("Paypal-Transmission-Sig") {
        Some(sig) => match sig.to_str() {
            Ok(sig_str) => sig_str,
            Err(_) => return false,
        },
        None => return false,
    };

    // Construct validation string
    let webhook_event_body = serde_json::to_string(body).unwrap_or_default();
    let validation_string = format!(
        "{}|{}|{}|{}",
        transmission_id,
        timestamp,
        config.webhook_id,
        webhook_event_body
    );

    // Create HMAC signature
    let mut mac = Hmac::<Sha256>::new_from_slice(config.webhook_secret.as_bytes())
        .expect("HMAC initialization failed");
    mac.update(validation_string.as_bytes());
    let calculated_signature = format!("{:x}", mac.finalize().into_bytes());

    // Compare signatures
    crc32 == calculated_signature
}

// Helper structure for PayPal webhook verification
#[derive(Clone)]
pub struct PayPalWebhookConfig {
    pub webhook_id: String,
    pub webhook_secret: String,
}
*/

// // Struct for caching arbitrary data
// #[derive(Clone)]
// pub struct CachedData<T> {
//     pub last_updated: Instant,
//     pub data: T,
// }

// Helper function to extract custom_id from different possible locations in the PayPal payload
async fn get_transaction_user_num(db: &web::Data<Arc<DBState>>, transaction_id: Uuid) -> Option<i64> {
    let mut client = db.web_db.pool.get().await.ok()?;
    let query = "SELECT UserNum FROM WebTransaction WHERE TransactionID = @P1";
    let row = client.query(query, &[&transaction_id]).await.ok()?.into_row().await.ok()??;
    row.get::<i64, _>("UserNum")
}

fn extract_invoice_id_from_payload(payload: &serde_json::Value) -> Option<String> {
    // First try to get it from the resource.purchase_units path
    if let Some(resource) = payload.get("resource") {
        if let Some(custom_id) = resource.get("invoice_id") {
            return custom_id.as_str().map(|s| s.to_string());
        }

        // Try the order path first
        if let Some(purchase_units) = resource.get("purchase_units") {
            if let Some(first_unit) = purchase_units.get(0) {
                if let Some(custom_id) = first_unit.get("invoice_id") {
                    return custom_id.as_str().map(|s| s.to_string());
                }
            }
        }

        // Try the payment path as fallback
        if let Some(custom) = resource.get("custom") {
            return custom.as_str().map(|s| s.to_string());
        }
    }

    None
}

// Helper function to update transaction status in the database
async fn update_transaction_status(
    transaction_id: Uuid,
    event_type: &str,
    db: &web::Data<Arc<DBState>>,
) -> Result<i64, Box<dyn Error>> {
    // Updated status codes based on the database schema:
    // 0: Pending (default in MSSQL)
    // 1: Approved (this is what triggers point addition)
    // 2: Declined or cancelled
    let status = match event_type {
        "CHECKOUT.ORDER.APPROVED" => 1, // Approved - this triggers the points addition
        "PAYMENT.CAPTURE.COMPLETED" => 1, // Completed/Approved - also triggers points addition
        "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.DECLINED" => 2, // Declined/Cancelled
        _ => 0,                         // Unknown/pending
    };

    // Using the correct method from WebTransactionTrait that we found in the codebase
    match db
        .web_db
        .update_transaction_status(transaction_id, status)
        .await
    {
        Ok(user_num) => {
            // The status update will trigger the trg_AddWebPoints trigger if status = 1 (approved)
            // which will automatically add points to the user account
            tracing::info!(
                "Updated transaction {} status to {}",
                transaction_id,
                status
            );
            Ok(user_num)
        }
        Err(e) => {
            tracing::error!("Failed to update transaction status: {:?}", e);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to update transaction status: {:?}", e),
            )))
        }
    }
}

async fn update_paymongo_transaction_status(
    transaction_id: Uuid,
    event_type: &str,
    gateway_type: &str,
    db: &web::Data<Arc<DBState>>,
) -> Result<(), Box<dyn Error>> {
    tracing::info!("Received update transaction for {}:{}:{}", gateway_type, event_type, transaction_id);

    // Updated status codes based on the database schema:
    // 0: Pending (default in MSSQL)
    // 1: Approved (this is what triggers point addition)
    // 2: Declined or cancelled
    let status = match gateway_type {
        "PayMongo" => match event_type {
            "payment.paid" => 1, // Approved - this triggers the points addition
            "payment.failed" => 2, // Declined/Cancelled
            _ => 0,                         // Unknown/pending
        },
        "PayPal" => match event_type {
            "CHECKOUT.ORDER.APPROVED" => 1, // Approved - this triggers the points addition
            "PAYMENT.CAPTURE.COMPLETED" => 1, // Completed/Approved - also triggers points addition
            "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.DECLINED" => 2, // Declined/Cancelled
            _ => 0,                         // Unknown/pending
        },
        _ => 0
    };

    // Using the correct method from WebTransactionTrait that we found in the codebase
    match db
        .web_db
        .update_transaction_status(transaction_id, status)
        .await
    {
        Ok(_) => {
            // The status update will trigger the trg_AddWebPoints trigger if status = 1 (approved)
            // which will automatically add points to the user account
            tracing::info!(
                "Updated transaction {} status to {}",
                transaction_id,
                status
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to update transaction status: {:?}", e);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to update transaction status: {:?}", e),
            )))
        }
    }
}

// Added filter for active only
#[get("/get-gateway-info")]
async fn get_gateway_info(db: web::Data<Arc<DBState>>) -> impl Responder {
    match db.web_db.get_gateway_info().await {
        Ok(gateways) => {
            let filtered_list: Vec<GatewayInfo> = gateways
                .into_iter()
                .filter(|item| item.status == 1)
                .collect();

            HttpResponse::Ok().json(filtered_list)
        } // ✅ Return the data
        Err(e) => {
            tracing::error!("❌ Failed to fetch gateway info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch gateway info" }))
        }
    }
}

#[get("/get-recharge-logs")]
async fn get_recharge_logs(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    state: web::Data<Arc<AppState>>,
    pagination: web::Query<PaginationParams>,
) -> impl Responder {
    let session: ran_models::users::ProfileData = match get_user_session(session).await {
        Ok(session) => session,
        Err(err) => {
            tracing::error!("Session error: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "message": "No session found"
            }));
        }
    };

    // Extract and validate pagination parameters
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10).min(MAX_QUERY_LIMIT as usize);

    // Log the filters if provided
    if let Some(status) = pagination.status {
        tracing::debug!("Filtering by status: {}", status);
    }
    if let Some(category) = pagination.category {
        tracing::debug!("Filtering by transaction type (category): {}", category);
    }

    // Use more aggressive caching for better performance with pagination-aware cache key
    let cache_key: String = format!("recharge_logs_{}_{}_{}", session.user_num, page, limit);
    let mut cache = state.transaction_cache.lock().await;
    let now = Instant::now();

    // Try to get from cache first
    if let Some(cached) = cache.get(&cache_key) {
        if now.duration_since(cached.last_updated) < Duration::from_secs(CACHE_VALIDITY_SECS) {
            let transactions = &cached.transactions;

            // Filter by status and transaction_type (using category) if provided
            let filtered_transactions: Vec<&WebTransaction> = transactions
                .iter()
                .filter(|item| {
                    let status_ok = pagination
                        .status
                        .map(|s| item.transaction_status == s)
                        .unwrap_or(true);
                    let type_ok = pagination
                        .category
                        .map(|t| item.transaction_type == t)
                        .unwrap_or(true);
                    status_ok && type_ok
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

            // Create a paginated response with filtered transactions
            let paginated_responses: Vec<WebTransactionResponse> = filtered_transactions
                .iter()
                .skip(start)
                .take(limit)
                .map(|tx| WebTransactionResponse::from(*tx))
                .collect();

            // Match the format of get-news endpoint - pagination info at root level
            return HttpResponse::Ok().json(json!({
                "page": page,
                "limit": limit,
                "total_pages": total_pages,
                "total_items": total_count,
                "data": paginated_responses
            }));
        }
    }

    // Cache miss, fetch from database
    match db.web_db.get_recharge_logs(session.user_num).await {
        Ok(rows) => {
            // First convert rows to WebTransaction objects
            let all_transactions: Vec<WebTransaction> = rows
                .into_iter()
                .filter_map(|row| Some(WebTransaction::from_row(&row)))
                .collect();

            // Filter by status and transaction_type (using category) if provided
            let filtered_transactions: Vec<&WebTransaction> = all_transactions
                .iter()
                .filter(|item| {
                    let status_ok = pagination
                        .status
                        .map(|s| item.transaction_status == s)
                        .unwrap_or(true);
                    let type_ok = pagination
                        .category
                        .map(|t| item.transaction_type == t)
                        .unwrap_or(true);
                    status_ok && type_ok
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

            // Update the cache with all transactions (unfiltered) for future use
            let cached_transactions = CachedTransactions {
                last_updated: now,
                transactions: all_transactions,
            };
            cache.insert(cache_key, cached_transactions);

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
            tracing::error!("❌ Failed to fetch recharge logs: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge logs" }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(paypal_webhook)
        .service(paymongo_webhook)
        .service(get_recharge_info)
        .service(transaction)
        .service(get_gateway_info)
        .service(get_recharge_info_by_id_endpoint)
        .service(get_recharge_logs);
}
