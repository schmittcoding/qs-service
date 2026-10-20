use actix_governor::{governor, Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::{
    dev::ServiceRequest,
    http::StatusCode,
    HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde_json::json;
use std::fmt;
use tracing::warn;

/// Error type for IP extraction failures.
#[derive(Debug)]
pub struct IpExtractionError;

impl fmt::Display for IpExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to extract IP address")
    }
}

impl ResponseError for IpExtractionError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(json!({
            "message": "Failed to determine client IP"
        }))
    }
}

/// Custom key extractor for IP-based rate limiting.
/// Reads the IP from X-Forwarded-For (leftmost value), X-Real-IP, or falls back to peer address.
#[derive(Clone)]
pub struct RealIpKeyExtractor;

impl KeyExtractor for RealIpKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<String>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Try X-Forwarded-For header first (take leftmost IP)
        if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                let ip = forwarded_str
                    .split(',')
                    .next()
                    .map(|s| s.trim())
                    .unwrap_or("");
                if !ip.is_empty() {
                    return Ok(ip.to_string());
                }
            }
        }

        // Try X-Real-IP header
        if let Some(real_ip) = req.headers().get("X-Real-IP") {
            if let Ok(real_ip_str) = real_ip.to_str() {
                if !real_ip_str.is_empty() {
                    return Ok(real_ip_str.to_string());
                }
            }
        }

        // Fall back to peer address
        let ip = req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if ip == "unknown" {
            warn!("Failed to determine client IP, falling back to 'unknown'");
        }

        Ok(ip)
    }

    fn exceed_rate_limit_response(
        &self,
        _: &governor::NotUntil<governor::clock::QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        response.status(StatusCode::TOO_MANY_REQUESTS).json(json!({
            "message": "Too many requests. Please try again later."
        }))
    }
}

/// Creates a rate limiter for validation endpoints.
/// Configuration: 1 request per 6 seconds with a burst size of 10 (allowing up to 10 requests per minute per IP).
///
/// To reuse this for other endpoints, simply call this function in the scope wrapper:
/// ```ignore
/// cfg.service(
///     web::scope("/other-path")
///         .wrap(validation_rate_limiter())
///         .service(endpoint_handler),
/// );
/// ```
pub fn validation_rate_limiter() -> Governor<RealIpKeyExtractor, actix_governor::governor::middleware::NoOpMiddleware<actix_governor::governor::clock::QuantaInstant>> {
    let config = GovernorConfigBuilder::default()
        .seconds_per_request(6)
        .burst_size(10)
        .key_extractor(RealIpKeyExtractor)
        .finish()
        .expect("Failed to build rate limiter config");

    Governor::new(&config)
}

/// Creates a rate limiter for email resend endpoints.
/// Configuration: 1 request per 30 seconds with a burst size of 3 (allowing up to 3 requests per 90 seconds per IP).
pub fn resend_verification_rate_limiter() -> Governor<RealIpKeyExtractor, actix_governor::governor::middleware::NoOpMiddleware<actix_governor::governor::clock::QuantaInstant>> {
    let config = GovernorConfigBuilder::default()
        .seconds_per_request(30)
        .burst_size(3)
        .key_extractor(RealIpKeyExtractor)
        .finish()
        .expect("Failed to build rate limiter config");

    Governor::new(&config)
}
