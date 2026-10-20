use anyhow::{Context, Result};
use ran_models::users::VerificationTokenClaims;

pub fn create_verification_token(user_num: i64, email: &str) -> Result<String> {
    let claims = VerificationTokenClaims {
        sub: user_num,
        email: email.to_string(),
        purpose: "email_verification".to_string(),
        iat: chrono::Utc::now().timestamp() as usize,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = std::env::var("VERIFICATION_JWT_SECRET").context("VERIFICATION_JWT_SECRET not set")?;
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode verification JWT")
}

pub fn decode_verification_token(token: &str) -> Result<VerificationTokenClaims> {
    let secret = std::env::var("VERIFICATION_JWT_SECRET").context("VERIFICATION_JWT_SECRET not set")?;
    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = true;
    let data = jsonwebtoken::decode::<VerificationTokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("Invalid or expired verification token")?;
    if data.claims.purpose != "email_verification" {
        anyhow::bail!("Invalid token purpose");
    }
    Ok(data.claims)
}
