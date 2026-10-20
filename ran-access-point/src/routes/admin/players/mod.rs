use std::{collections::HashMap, sync::Arc};

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::{gamemanager::admin::characters::GameCharacterTrait, webmanager::admin::players::WebPlayerTrait};
use ran_models::{admin::{players::{BanAccountRequestPayload, UserAccountInfo, UserCharacterList}, AdminPaginationParams, AdminPlayersFilterParams, BasicPaginationParams}, reports::transactions::WebReportTransaction, PaginationParams};
use serde_json::json;
use serde_qs::Config;
use tiberius::Row;
use tracing::error;

use crate::{constants::http_messages::http_messages::error, database::DBState, services::jwt::Claims};

#[utoipa::path(
    get,
    path = "/admin/v1/players",
    operation_id = "admin_get_players",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page"),
        ("category" = Option<Vec<i16>>, Query, description = "Filter by user type (category)"),
        ("status" = Option<Vec<i16>>, Query, description = "Filter by user block status"),
        ("search" = Option<String>, Query, description = "Search by username")
    ),
    responses(
        (status = 200, description = "List of players", body = serde_json::Value),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("")]
pub async fn get_players(
    db: web::Data<Arc<DBState>>,
    req: HttpRequest,
) -> impl Responder {
    let qs = req.query_string();
    let config = Config::new(5, false);

    let params: AdminPlayersFilterParams = match config.deserialize_str(qs) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid query: {}", e)),
    };

    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let search = params.search.clone();
    let category = params.category.as_ref().filter(|v| !v.is_empty()).cloned();
    let status = params.status.as_ref().filter(|v| !v.is_empty()).cloned();

    let result = db.web_db.get_players(page, limit, category, status, search).await;

    match result {
        Ok(res) => {
            let data: Vec<UserAccountInfo> = res.data
                .into_iter()
                .map(|row| {
                    UserAccountInfo::from_row(&row)
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": limit,
                "total_items": res.total_records
            }))
        },
        Err(e) => {
            log::error!("Failed to fetch players list: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch players list." }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/players/{id}",
    operation_id = "admin_get_player_by_id",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i64, Path, description = "Player user ID")
    ),
    responses(
        (status = 200, description = "Player details", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}")]
pub async fn get_player_by_id(
    path: web::Path<i64>,
    db: web::Data<Arc<crate::database::DBState>>,
) -> impl Responder {
    let user_num = path.into_inner();

    let result = db.web_db.get_player_by_id(user_num).await;

    match result {
        Ok(res) => {
            HttpResponse::Ok().json(json!(res))
        },
        Err(e) => {
            log::error!("Failed to fetch player data: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch player data." }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/admin/v1/players/{id}/ban-account",
    operation_id = "admin_ban_player_account",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i64, Path, description = "Player user ID")
    ),
    request_body = BanAccountRequestPayload,
    responses(
        (status = 200, description = "Account banned successfully", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/{id}/ban-account")]
pub async fn ban_player_account(
    path: web::Path<i64>,
    req: HttpRequest,
    db: web::Data<Arc<crate::database::DBState>>,
    body: web::Json<BanAccountRequestPayload>,
) -> impl Responder {
    let user_num = path.into_inner();
    let data = body.into_inner();

    if let Some(claims) = req.extensions().get::<Claims>() {
        match db
            .user_db
            .ban_unban_account(
                user_num,
                claims.sub.clone(),
                data.status,
                data.block_date,
                data.notes
            )
            .await
        {
            Ok(row) => {
                let message = row
                    .try_get::<&str, _>("Message")
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                match message {
                    "SUCCESS" => {
                        HttpResponse::Ok().json(json!({ "success": true }))
                    },
                    "UNAUTHORIZED" => {
                        log::error!("❌ Failed to process account ban {:?}", message);
                        HttpResponse::Unauthorized()
                            .json(json!({ "error": "You don't have permission to perform this transaction" }))
                    },
                    "USER_NOT_FOUND" => {
                        log::error!("❌ Failed to process account ban {:?}", message);
                        HttpResponse::NotFound()
                            .json(json!({ "error": "User not found" }))
                    },
                    _ => {
                        log::error!("❌ Failed to process account ban: {:?}", message);
                        HttpResponse::InternalServerError()
                            .json(json!({ "error": "Failed to process account ban" }))
                    }
                }

            },
            Err(e) => {
                log::error!("❌ Failed to process account ban: {:?}", e);
                HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to process account ban" }))
            }
        }
    } else {
        HttpResponse::Unauthorized().body("No valid JWT found")
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/players/{id}/ban-history",
    operation_id = "admin_ban_account_history",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i64, Path, description = "Player user ID"),
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Ban history", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}/ban-history")]
pub async fn ban_account_history(
    path: web::Path<i64>,
    db: web::Data<Arc<crate::database::DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    match db.user_db.ban_account_history(user_num, page, limit).await {
        Ok(res) => {
            HttpResponse::Ok().json(json!({
                "page": page,
                "limit": limit,
                "total_items": res.total_records,
                "data": res.data,
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch account ban history: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch account ban history" }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/players/{id}/transaction-history",
    operation_id = "admin_account_transaction_history",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i64, Path, description = "Player user ID"),
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page"),
        ("start_date" = Option<String>, Query, description = "Filter by start date"),
        ("end_date" = Option<String>, Query, description = "Filter by end date")
    ),
    responses(
        (status = 200, description = "Transaction history with summary", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}/transaction-history")]
pub async fn account_transaction_history(
    path: web::Path<i64>,
    db: web::Data<Arc<crate::database::DBState>>,
    params: web::Query<AdminPaginationParams>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);
    let start_date: Option<String> = params.start_date.clone();
    let end_date: Option<String> = params.end_date.clone();

    match db
        .web_db
        .get_account_transaction(
            page,
            limit,
            user_num,
            start_date,
            end_date
        )
        .await
    {
        Ok(res) => {
            let transactions: Vec<WebReportTransaction> = res.transactions
                .into_iter()
                .map(|row| WebReportTransaction::from_row(&row))
                .collect();

            let mut grouped: HashMap<String, Vec<&Row>> = HashMap::new();

            for row in &res.summary {
                let r#type: i16 = row.get("TransactionType").unwrap_or(0);
                grouped.entry(r#type.to_string()).or_default().push(row);
            }

            let mut summary: HashMap<String, HashMap<String, f64>> = HashMap::new();

            for (r#type, group) in grouped {
                let mut status_map: HashMap<String, f64> = HashMap::new();

                for row in &group {
                    let status: i16 = row.get("TransactionStatus").unwrap_or(0);
                    let total: f64 = row.get("TotalAmount").unwrap_or(0.0);

                    match status {
                        0 => *status_map.entry("pending".to_string()).or_insert(0.0) += total,
                        1 => *status_map.entry("approved".to_string()).or_insert(0.0) += total,
                        2 => *status_map.entry("cancelled".to_string()).or_insert(0.0) += total,
                        3 => *status_map.entry("rejected".to_string()).or_insert(0.0) += total,
                        _ => {}
                    }
                }

                summary.insert(r#type.clone(), status_map.clone());
            }

            HttpResponse::Ok().json(json!({
                "page": page,
                "limit": limit,
                "total_items": res.total_records,
                "data": transactions,
                "summary": summary
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch account ban history: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch account ban history" }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/players/{id}/characters",
    operation_id = "admin_get_characters_by_account",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i32, Path, description = "Player user ID"),
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of characters owned by player", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}/characters")]
pub async fn get_characters_by_account(
    path: web::Path<i32>,
    db: web::Data<Arc<crate::database::DBState>>,
    params: web::Query<BasicPaginationParams>,
) -> impl Responder {
    let user_num = path.into_inner();
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    match db
        .game_db
        .get_characters_by_user(page, limit, user_num)
        .await
    {
        Ok(res) => {
            let data: Vec<UserCharacterList> = res.data
                .into_iter()
                .map(|row| {
                    UserCharacterList::from_row(&row)
                })
                .collect();

            HttpResponse::Ok()
                .json(json!({
                    "page": page,
                    "limit": limit,
                    "total_items": res.total_records,
                    "data": data,
                }))
        }
        Err(e) => {
            log::error!("Failed to fetch characters by user: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/v1/players/{id}/login-history",
    operation_id = "admin_get_user_login_history",
    tag = "Admin - Players",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i64, Path, description = "Player user ID"),
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "User login history", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}/login-history")]
pub async fn get_login_history(
    path: web::Path<i64>,
    db: web::Data<Arc<DBState>>,
    params: web::Query<BasicPaginationParams>,
) -> impl Responder {
    let user_id = path.into_inner();
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1).max(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10).min(100);

    match db.user_db.get_login_history(user_id, page, limit).await {
        Ok(res) => {
            HttpResponse::Ok().json(json!({
                "page": page,
                "limit": limit,
                "total_items": res.total_records,
                "data": res.data,
            }))
        }
        Err(e) => {
            error!("Failed to fetch user login history: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch login history" }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_players)
        .service(get_player_by_id)
        .service(ban_player_account)
        .service(ban_account_history)
        .service(account_transaction_history)
        .service(get_characters_by_account)
        .service(get_login_history);
}
