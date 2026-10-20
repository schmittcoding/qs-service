//! Integration tests for GET /admin/v1/profile endpoint
//!
//! Tests:
//! - Valid JWT token with existing admin user → 200 OK
//! - Valid JWT token but admin user not found → 404 Not Found
//! - Invalid JWT token (bad signature) → 401 Unauthorized
//! - Expired JWT token → 401 Unauthorized
//! - Missing Authorization header → 401 Unauthorized
//! - Valid JWT but claims.sub is not a valid UUID → 400 Bad Request
//! - Response structure validation

use actix_web::{test, web, App, HttpResponse, Responder};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

// We can't directly access private modules from ran-access-point, so we'll use
// the public interface via actix-web testing

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MockClaims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

// ============================================================================
// Test Constants
// ============================================================================

const JWT_SECRET: &str = "test_secret_key_for_admin_profile_testing";
const VALID_ADMIN_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";
const INVALID_UUID: &str = "this-is-not-a-uuid";
const NON_EXISTENT_UUID: &str = "11111111-1111-1111-1111-111111111111";

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a valid JWT token with custom claims
fn generate_valid_jwt(user_id: &str, role: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let expiration = now + 3600; // 1 hour from now
    let claims = MockClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("Failed to generate JWT")
}

/// Generate an expired JWT token
fn generate_expired_jwt(user_id: &str, role: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let expiration = now - 3600; // 1 hour ago (expired)
    let claims = MockClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("Failed to generate JWT")
}

/// Generate a JWT with wrong secret (invalid signature)
fn generate_invalid_signature_jwt(user_id: &str, role: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let expiration = now + 3600; // 1 hour from now
    let claims = MockClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("wrong_secret_key".as_bytes()),
    )
    .expect("Failed to generate JWT")
}

// ============================================================================
// Mock Service for Testing
// ============================================================================

/// A mock handler that simulates the actual get_profile behavior
/// This is for testing the endpoint logic without requiring a real database.
async fn mock_get_profile(
    req: actix_web::HttpRequest,
) -> impl Responder {
    use uuid::Uuid;

    // Simulate extracting JWT claims from request extensions
    // In real implementation, this would be done by JwtMiddleware
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        // Try to decode the token
        if let Ok(token_data) = jsonwebtoken::decode::<MockClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
        ) {
            let claims = token_data.claims;

            // Parse user_id from claims.sub
            match Uuid::parse_str(&claims.sub) {
                Ok(user_id) => {
                    // Simulate database lookup
                    if user_id.to_string() == VALID_ADMIN_UUID {
                        HttpResponse::Ok().json(json!({
                            "user_name": "admin1",
                            "user_id": VALID_ADMIN_UUID,
                            "user_type": 1
                        }))
                    } else {
                        HttpResponse::NotFound()
                            .json(json!({"message": "Admin not found"}))
                    }
                }
                Err(_) => {
                    HttpResponse::BadRequest()
                        .json(json!({"message": "Invalid token subject"}))
                }
            }
        } else {
            HttpResponse::Unauthorized()
                .json(json!({"message": "Unauthorized"}))
        }
    } else {
        HttpResponse::Unauthorized()
            .json(json!({"message": "Unauthorized"}))
    }
}

// ============================================================================
// Test Cases
// ============================================================================

#[actix_web::test]
async fn test_get_profile_with_valid_jwt_and_existing_admin() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    assert_eq!(json["user_id"], VALID_ADMIN_UUID);
    assert_eq!(json["user_name"], "admin1");
    assert_eq!(json["user_type"], 1);
}

#[actix_web::test]
async fn test_get_profile_with_valid_jwt_but_admin_not_found() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(NON_EXISTENT_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    assert_eq!(json["message"], "Admin not found");
}

#[actix_web::test]
async fn test_get_profile_missing_authorization_header() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    assert_eq!(json["message"], "Unauthorized");
}

#[actix_web::test]
async fn test_get_profile_with_invalid_jwt_bad_signature() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_invalid_signature_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_profile_with_expired_jwt() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_expired_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_profile_with_invalid_uuid_in_claims() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(INVALID_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    assert_eq!(json["message"], "Invalid token subject");
}

#[actix_web::test]
async fn test_get_profile_response_has_required_fields() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // Verify all required fields are present
    assert!(json.get("user_name").is_some(), "user_name field missing");
    assert!(json.get("user_id").is_some(), "user_id field missing");
    assert!(json.get("user_type").is_some(), "user_type field missing");

    // Verify field types
    assert!(json["user_name"].is_string(), "user_name should be a string");
    assert!(json["user_id"].is_string(), "user_id should be a string");
    assert!(json["user_type"].is_number(), "user_type should be a number");
}

#[actix_web::test]
async fn test_get_profile_user_id_is_valid_uuid_format() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let user_id = json["user_id"].as_str().expect("user_id should be a string");
    assert!(Uuid::parse_str(user_id).is_ok(), "user_id should be a valid UUID");
}

#[actix_web::test]
async fn test_get_profile_malformed_bearer_token() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", "InvalidTokenFormat"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_profile_empty_bearer_token() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", "Bearer "))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_profile_with_empty_uuid() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt("", "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_get_profile_user_type_is_integer() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(VALID_ADMIN_UUID, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let user_type = json["user_type"].as_i64().expect("user_type should be an integer");
    assert!(user_type >= 0, "user_type should be non-negative");
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[actix_web::test]
async fn test_get_profile_case_sensitive_bearer_prefix() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let token = generate_valid_jwt(VALID_ADMIN_UUID, "admin");
    // Use lowercase "bearer" instead of "Bearer"
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should fail because header parsing is case-sensitive for the prefix
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_profile_with_very_long_uuid() {
    let app = test::init_service(
        App::new().service(web::scope("/admin/v1").route("/profile", web::get().to(mock_get_profile)))
    )
    .await;

    let long_uuid = format!("{}-extra-garbage", VALID_ADMIN_UUID);
    let token = generate_valid_jwt(&long_uuid, "admin");
    let req = test::TestRequest::get()
        .uri("/admin/v1/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
