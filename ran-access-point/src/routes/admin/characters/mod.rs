use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use ran_db_core::gamemanager::admin::characters::GameCharacterTrait;
use ran_models::{
    admin::{CharacterPaginationParams, characters::CharacterInfoList},
    errors::flatten_validation_errors,
};
use serde_json::json;
use serde_qs::Config;
use validator::Validate;

use crate::{constants::http_messages::http_messages::error, database::DBState};

#[utoipa::path(
    get,
    path = "/admin/v1/characters",
    operation_id = "admin_get_characters",
    tag = "Admin - Characters",
    security(("bearer_jwt" = [])),
    params(
        ("page" = Option<usize>, Query, description = "Page number"),
        ("limit" = Option<usize>, Query, description = "Items per page"),
        ("class" = Option<Vec<i32>>, Query, description = "Filter by one or more character classes"),
        ("school" = Option<Vec<i32>>, Query, description = "Filter by schools"),
        ("status" = Option<i16>, Query, description = "Filter by character status"),
        ("search" = Option<String>, Query, description = "Search by character name"),
        ("sort_by" = Option<String>, Query, description = "Sort by field: 'contribution_point', 'level', or 'money'"),
        ("sort_order" = Option<String>, Query, description = "Sort order: 'asc' or 'desc'"),
        ("cha_online" = Option<Vec<i16>>, Query, description = "Filter by one or more online statuses: 0 (offline), 1 (online)")
    ),
    responses(
        (status = 200, description = "List of characters", body = serde_json::Value),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("")]
pub async fn get_characters(
    db: web::Data<Arc<DBState>>,
    req: HttpRequest
) -> impl Responder {
    let qs = req.query_string();
    let config = Config::new(5, false);

    let params: CharacterPaginationParams = match config.deserialize_str(qs) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid query: {}", e)),
    };

    if let Err(validation_errors) = params.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    let page: i16 = params.page.map(|p| p as i16).unwrap_or(1);
    let limit: i16 = params.limit.map(|p| p as i16).unwrap_or(10);

    let sort_by = params.sort_by.as_ref().map(|sb| sb.as_db_str()).unwrap_or("");
    let sort_order = params.sort_order.as_ref().map(|so| so.as_db_str()).unwrap_or("desc");

    match db
        .game_db
        .get_characters(
            page,
            limit,
            params.class.clone(),
            params.school.clone(),
            params.status,
            params.search.clone(),
            sort_by,
            sort_order,
            params.cha_online.clone()
        )
        .await
    {
        Ok(res) => {
            let data: Vec<CharacterInfoList> = res.data
                .into_iter()
                .map(|row| CharacterInfoList::from_row(&row))
                .collect();

            HttpResponse::Ok().json(json!({
                "data": data,
                "page": page,
                "limit": limit,
                "total_items": res.total_records
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch characters: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "error": error::INTERNAL_SERVER_ERROR }))
        }
    }

}

#[utoipa::path(
    get,
    path = "/admin/v1/characters/{id}",
    operation_id = "admin_get_character_by_id",
    tag = "Admin - Characters",
    security(("bearer_jwt" = [])),
    params(
        ("id" = i32, Path, description = "Character ID")
    ),
    responses(
        (status = 200, description = "Character details", body = serde_json::Value),
        (status = 404, description = "Character not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{id}")]
pub async fn get_character_by_id(
    db: web::Data<Arc<DBState>>,
    path: web::Path<i32>,
) -> impl Responder {
    let cha_num = path.into_inner();

    match db.game_db.get_character_by_id(cha_num).await {
        Ok(character) => HttpResponse::Ok().json(character),
        Err(e) => {
            tracing::error!("Failed to fetch character {}: {:?}", cha_num, e);
            HttpResponse::NotFound().json(json!({ "error": "Character not found" }))
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_characters)
        .service(get_character_by_id);
}
