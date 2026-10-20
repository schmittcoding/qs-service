use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::webmanager::report::WebReportTrait;
use ran_models::{reports::transactions::WebReportTransaction, PaginationParams};
use serde_json::json;

use crate::{database::DBState};

#[get("/summary")]
async fn get_transaction_reports(
    db: web::Data<Arc<DBState>>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let status: Option<i16> = params.status;
    let r#type: Option<i16> = params.category;
    let search: Option<String> = params.search.clone();
    let start_date: Option<String> = params.start_date.clone();
    let end_date: Option<String> = params.end_date.clone();

    let result = db.web_db.get_transaction_report(page, limit, r#type, status, search, start_date, end_date).await;

    match result {
        Ok(res) => {
            let mut pending = 0.0;
            let mut approved = 0.0;
            let mut cancelled = 0.0;
            let mut rejected = 0.0;

            for row in &res.summary {
                let status: i16 = row.get("TransactionStatus").unwrap_or(0);
                let total: f64 = row.get("TotalAmount").unwrap_or(0.0);

                match status {
                    0 => pending = total,
                    1 => approved = total,
                    2 => rejected = total,
                    3 => cancelled = total,
                    _ => {}
                }
            }

            let transactions: Vec<WebReportTransaction> = res.transactions
                .into_iter()
                .map(|row| WebReportTransaction::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "summary": {
                    "pending": pending,
                    "approved": approved,
                    "cancelled": cancelled,
                    "rejected": rejected,
                },
                "page": page,
                "limit": limit,
                "total_items": res.total_records,
                "data": transactions,
            }))
        },
        Err(e) => {
            log::error!("Failed to fetch recharge logs: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to fetch recharge logs" }))
        }
    }
}


pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_transaction_reports);
}