use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ran_db_core::webmanager::email::templates::WebEmailTemplateTrait;
use ran_models::{
    email::{
        CreateEmailTemplateRequest, EmailTemplateFilterParams, PreviewTemplateRequest,
        UpdateEmailTemplateRequest,
    },
    errors::flatten_validation_errors,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{database::DBState, services::jwt::Claims};

#[post("")]
async fn create_template(
    db: web::Data<Arc<DBState>>,
    req: HttpRequest,
    body: web::Json<CreateEmailTemplateRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"})),
    };

    let data = body.into_inner();
    if let Err(validation_errors) = data.validate() {
        return HttpResponse::BadRequest().json(flatten_validation_errors(&validation_errors));
    }

    match db
        .web_db
        .create_email_template(data, &claims.sub)
        .await
    {
        Ok(template) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": template
        })),
        Err(e) => {
            tracing::error!("Failed to create email template: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to create email template"}))
        }
    }
}

#[get("")]
async fn get_templates(
    db: web::Data<Arc<DBState>>,
    query: web::Query<EmailTemplateFilterParams>,
) -> impl Responder {
    let params = query.into_inner();
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    match db
        .web_db
        .get_email_templates(params.status, params.category, params.search, page, limit)
        .await
    {
        Ok(result) => HttpResponse::Ok().json(json!({
            "data": result.data,
            "page": page,
            "limit": limit,
            "total_items": result.total_records
        })),
        Err(e) => {
            tracing::error!("Failed to fetch email templates: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch email templates"}))
        }
    }
}

#[get("/{id}")]
async fn get_template_by_id(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let template_id = path.into_inner();

    match db.web_db.get_email_template_by_id(template_id).await {
        Ok(Some(template)) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": template
        })),
        Ok(None) => {
            HttpResponse::NotFound().json(json!({"error": "Template not found"}))
        }
        Err(e) => {
            tracing::error!("Failed to fetch email template: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch email template"}))
        }
    }
}

#[put("/{id}")]
async fn update_template(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateEmailTemplateRequest>,
) -> impl Responder {
    let template_id = path.into_inner();
    let data = body.into_inner();

    if let Err(validation_errors) = data.validate() {
        return HttpResponse::BadRequest().json(flatten_validation_errors(&validation_errors));
    }

    match db.web_db.update_email_template(template_id, data).await {
        Ok(template) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": template
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("NOT_FOUND") {
                return HttpResponse::NotFound()
                    .json(json!({"error": "Template not found"}));
            }
            tracing::error!("Failed to update email template: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to update email template"}))
        }
    }
}

#[delete("/{id}")]
async fn delete_template(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let template_id = path.into_inner();

    match db.web_db.delete_email_template(template_id).await {
        Ok(template) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": template,
            "message": "Template deactivated"
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("NOT_FOUND") {
                return HttpResponse::NotFound()
                    .json(json!({"error": "Template not found"}));
            }
            tracing::error!("Failed to delete email template: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to delete email template"}))
        }
    }
}

#[post("/{id}/preview")]
async fn preview_template(
    db: web::Data<Arc<DBState>>,
    path: web::Path<Uuid>,
    body: Option<web::Json<PreviewTemplateRequest>>,
) -> impl Responder {
    let template_id = path.into_inner();

    let template = match db.web_db.get_email_template_by_id(template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({"error": "Template not found"}));
        }
        Err(e) => {
            tracing::error!("Failed to fetch template for preview: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch template"}));
        }
    };

    let mut variables = crate::services::smtp::default_preview_variables();
    if let Some(body) = body {
        if let Some(custom_vars) = body.into_inner().variables {
            variables.extend(custom_vars);
        }
    }

    let (rendered_subject, rendered_html) =
        crate::services::smtp::render_template(&template.html_body, &template.subject_line, &variables);

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "subject": rendered_subject,
            "html": rendered_html
        }
    }))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_template)
        .service(get_templates)
        .service(get_template_by_id)
        .service(update_template)
        .service(delete_template)
        .service(preview_template);
}
