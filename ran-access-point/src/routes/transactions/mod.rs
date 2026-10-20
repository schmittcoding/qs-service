use std::{sync::Arc};

use actix_web::{get, web, HttpResponse, Responder};
use ran_db_core::webmanager::transactions::WebTransactionUserTrait;
use ran_models::{
    payments::{WebTransaction, WebTransactionResponse},
    PaginationParams,
};
use serde_json::json;

use crate::{constants::http_messages::http_messages::error, database::DBState, middleware::session::get_user_session};

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(get_transactions);
}


#[get("")]
async fn get_transactions(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    params: web::Query<PaginationParams>
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

    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    match db.web_db.get_user_transactions(page, limit, session.user_num).await {
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
