use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum EventError {
    NotFound(String),
    Duplicate(String),
    ValidationFailed(String),
    RegistrationClosed,
    Unauthorized,
    Forbidden(String),
    DbError(String),
    CacheError(String),
    Internal(String),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::NotFound(msg) => write!(f, "Not found: {}", msg),
            EventError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            EventError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            EventError::RegistrationClosed => write!(f, "Registration is no longer open."),
            EventError::Unauthorized => write!(f, "Authentication required."),
            EventError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            EventError::DbError(msg) => write!(f, "Database error: {}", msg),
            EventError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            EventError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for EventError {
    fn error_response(&self) -> HttpResponse {
        let (status, code, message) = match self {
            EventError::NotFound(msg) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone(),
            ),
            EventError::Duplicate(msg) => (
                actix_web::http::StatusCode::CONFLICT,
                "DUPLICATE",
                msg.clone(),
            ),
            EventError::ValidationFailed(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                msg.clone(),
            ),
            EventError::RegistrationClosed => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "CLOSED",
                "Registration is no longer open.".to_string(),
            ),
            EventError::Unauthorized => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required.".to_string(),
            ),
            EventError::Forbidden(msg) => (
                actix_web::http::StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg.clone(),
            ),
            EventError::DbError(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Database error.".to_string(),
            ),
            EventError::CacheError(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR",
                "Cache error.".to_string(),
            ),
            EventError::Internal(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "Internal server error.".to_string(),
            ),
        };

        HttpResponse::build(status).json(json!({
            "success": false,
            "code": code,
            "message": message,
        }))
    }
}

impl From<tiberius::error::Error> for EventError {
    fn from(err: tiberius::error::Error) -> Self {
        EventError::DbError(err.to_string())
    }
}
