use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize)]
pub struct DiscordWebhookMessage<'a> {
    pub content: &'a str,
    pub username: &'a str,
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct RegistrationRequest {
    #[validate(length(min = 1, message = "Full name cannot be empty"))]
    pub full_name: String,
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 3,
        max = 20,
        message = "Password must be between 3 and 20 characters"
    ))]
    pub password: String,
    #[validate(length(equal = 6, message = "Pincode must be exactly 6 characters long"))]
    pub pincode: String,
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,
}

#[derive(Serialize)]
pub struct NotAuthenticated {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct AuthRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 3,
        max = 20,
        message = "Password must be between 3 and 20 characters"
    ))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct AuthResponse {
    pub is_authenticated: Option<i32>,
    pub user_num: Option<i64>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, utoipa::ToSchema)]
pub struct ProfileData {
    pub user_name: String,
    pub user_num: i64,
    pub user_email: String,
    pub user_available: u8,
    pub user_type: i32,
    pub web_points: f64,
    pub mileage_points: f64,
    pub created_at: DateTime<Local>,
    pub last_updated: DateTime<Local>,
    #[serde(default)]
    pub is_verified: bool,
}

impl ProfileData {
    pub fn user_role(&self) -> &'static str {
        match self.user_type {
            20 | 78 => "admin",
            _ => "player",
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "user_name": self.user_name,
            "user_num": self.user_num,
            "user_email": self.user_email,
            "user_available": self.user_available,
            "web_points": self.web_points,
            "mileage_points": self.mileage_points,
            "created_at": self.created_at,
            "last_updated": self.last_updated,
            "is_verified": self.is_verified,
            "user_role": self.user_role(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ChangePincodeRequest {
    #[validate(length(equal = 6, message = "Pincode must be exactly 6 characters long"))]
    pub old_pincode: String,
    #[validate(length(equal = 6, message = "Pincode must be exactly 6 characters long"))]
    pub new_pincode: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ChangeEmailRequest {
    pub email: String,
    #[validate(length(equal = 6, message = "Pincode must be exactly 6 characters long"))]
    pub pincode: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    #[validate(length(equal = 6, message = "Pincode must be exactly 6 characters long"))]
    pub pincode: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ValidateUsernameRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, utoipa::ToSchema)]
pub struct ValidateEmailRequest {
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationTokenClaims {
    pub sub: i64,
    pub email: String,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}
