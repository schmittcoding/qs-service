use crate::{database::DBState, services, states::AppState};
use actix_web::{get, post, web, HttpResponse, Responder};
use ran_db_core::webmanager::{auth::WebAuthTrait, user::WebUserTrait};
use ran_models::{
    errors::flatten_validation_errors,
    users::{
        AuthRequest, ChangeEmailRequest, ChangePasswordRequest, ForgotPasswordRequest, ProfileData,
        RegistrationRequest, ValidateEmailRequest, ValidateUsernameRequest, VerifyEmailRequest,
    },
};
use rand::{rng, Rng};
use serde_json::json;
use std::sync::Arc;
use tracing::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegistrationRequest,
    responses(
        (status = 200, description = "User registered successfully", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Username already taken"),
        (status = 500, description = "Registration failed")
    )
)]
#[post("/register")]
pub async fn register(
    auth_request: web::Json<RegistrationRequest>,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Err(validation_errors) = auth_request.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    let registration_data = auth_request.into_inner();

    if registration_data.full_name.is_empty()
        || registration_data.password.is_empty()
        || registration_data.email.is_empty()
        || registration_data.username.is_empty()
    {
        return HttpResponse::BadRequest().json("Error: Missing fields");
    }

    // let email = registration_data.email.clone();
    match db.web_db.register_user(registration_data).await {
        Ok(_user_num) => {
            // Send verification email (non-fatal — user can resend later)
            // match services::verification::create_verification_token(user_num, &email) {
            //     Ok(jwt) => {
            //         if let Ok((user_email, username)) =
            //             db.web_db.save_verification_token(user_num, &jwt).await
            //         {
            //             let is_suppressed = {
            //                 let cache = states.suppression_cache.read().await;
            //                 cache.contains(&user_email.to_lowercase())
            //             };
            //             if is_suppressed {
            //                 tracing::info!("Skipping verification email to suppressed address: {}", user_email);
            //                 let _ = db.web_db.log_email_sent(user_num, "verification", &user_email, "suppressed", None).await;
            //             } else {
            //                 let config = services::smtp::SmtpConfig::from_env().ok();
            //                 let domain = config
            //                     .map(|c| c.web_domain)
            //                     .unwrap_or_else(|| "https://www.ranonlinegs.com".to_string());
            //                 let link = format!("{}/verify-email?token={}", domain, jwt);
            //                 let send_result = services::smtp::send_verification_email(
            //                     &username,
            //                     &user_email,
            //                     &link,
            //                 )
            //                 .await;
            //                 let (status, err_msg) = match &send_result {
            //                     Ok(_) => ("sent", None),
            //                     Err(e) => {
            //                         error!("Failed to send verification email for user {}: {:?}", user_num, e);
            //                         ("failed", Some(e.to_string()))
            //                     }
            //                 };
            //                 let _ = db
            //                     .web_db
            //                     .log_email_sent(
            //                         user_num,
            //                         "verification",
            //                         &user_email,
            //                         status,
            //                         err_msg.as_deref(),
            //                     )
            //                     .await;
            //             }
            //         }
            //     }
            //     Err(e) => {
            //         error!("Failed to create verification token for user {}: {:?}", user_num, e);
            //     }
            // }

            HttpResponse::Ok().json(json!({"message": "User has been registered successfully"}))
        }
        Err(err) => {
            log::error!("Registration error: {}", err.to_string());
            if err.to_string().contains("ERROR_DUPLICATE_USERNAME") {
                HttpResponse::Conflict().json(json!({"message": "Username is already taken"}))
            } else {
                HttpResponse::InternalServerError().json(json!({"message": "Registration failed"}))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    security(("cookie_session" = [])),
    responses(
        (status = 200, description = "Logout successful", body = serde_json::Value),
        (status = 401, description = "Not authenticated")
    )
)]
#[post("/logout")]
pub async fn logout(session: actix_session::Session) -> impl Responder {
    if let Ok(Some(profile)) = session.get::<ProfileData>("user_data") {
        ran_db_core::webmanager::user::cache::invalidate_profile(profile.user_num);
    }

    // Purge fully invalidates the session and clears the session cookie via the middleware.
    session.purge();
    HttpResponse::Ok().finish()
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/authenticate",
    tag = "Auth",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Authentication successful", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/authenticate")]
pub async fn authenticate(
    auth_request: web::Json<AuthRequest>,
    session: actix_session::Session,
    db: web::Data<std::sync::Arc<DBState>>,
) -> impl Responder {
    if let Err(validation_errors) = auth_request.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    match db
        .web_db
        .authenticate_user(&auth_request.username, &auth_request.password)
        .await
    {
        Ok(auth_response) => {
            if auth_response.is_authenticated == Some(1) {
                if let Some(user_num) = auth_response.user_num {
                    match db.web_db.get_profile_from_db(user_num).await {
                        Ok(Some(user_profile)) => {
                            // Regenerate session ID to prevent session fixation attacks.
                            session.renew();
                            // Insert the user profile into the session.
                            if let Err(e) = session.insert("user_data", &user_profile) {
                                error!("Failed to insert user_profile into session: {}", e);
                                return HttpResponse::InternalServerError().into();
                            }

                            // Return the user profile as JSON.
                            HttpResponse::Ok().json(user_profile.to_json())
                        }
                        Ok(None) => {
                            error!("No profile found for authenticated user {}", user_num);
                            HttpResponse::InternalServerError().into()
                        }
                        Err(e) => {
                            error!("Failed to fetch user profile: {}", e);
                            HttpResponse::InternalServerError().into()
                        }
                    }
                } else {
                    error!("Authentication succeeded, but no user number was returned");
                    HttpResponse::Unauthorized().into()
                }
            } else {
                HttpResponse::Unauthorized().into()
            }
        }
        Err(e) => {
            error!("Error during authentication: {}", e);
            HttpResponse::InternalServerError().into()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/validate",
    tag = "Auth",
    security(("cookie_session" = [])),
    responses(
        (status = 200, description = "Session is valid", body = serde_json::Value),
        (status = 401, description = "Not authenticated")
    )
)]
#[get("/validate")]
pub async fn validate(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let session_profile = match crate::middleware::session::get_user_session(session.clone()).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    let user_num = session_profile.user_num;

    match db.web_db.get_profile_from_db(user_num).await {
        Ok(Some(fresh_profile)) => {
            if let Err(e) = session.insert("user_data", &fresh_profile) {
                error!("Failed to update session with fresh profile: {}", e);
            }

            let expires_at = (chrono::Local::now() + chrono::Duration::days(1)).to_rfc3339();
            HttpResponse::Ok().json(json!({
                "user": fresh_profile.to_json(),
                "expires_at": expires_at,
            }))
        }
        Ok(None) => {
            error!("No profile found for validated user {}", user_num);
            HttpResponse::InternalServerError().json(json!({"message": "Profile not found"}))
        }
        Err(e) => {
            error!("Failed to fetch fresh profile for user {}: {}", user_num, e);
            let expires_at = (chrono::Local::now() + chrono::Duration::days(1)).to_rfc3339();
            HttpResponse::Ok().json(json!({
                "user": session_profile.to_json(),
                "expires_at": expires_at,
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent if account exists", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Failed to send email")
    )
)]
#[post("/forgot-password")]
pub async fn forgot_password(
    data: web::Json<ForgotPasswordRequest>,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Err(validation_errors) = data.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    let email = data.email.trim();
    let username = &data.username;

    // Get all profiles matching the email
    let profiles = match db.user_db.get_user_using_email(email).await {
        Ok(profiles) => profiles,
        Err(e) => {
            error!("Error looking up user by email {}: {:?}", email, e);
            return HttpResponse::InternalServerError()
                .json(json!({ "message": "Failed to update password" }));
        }
    };

    let matched_profile = profiles
        .iter()
        .cloned()
        .find(|profile| username.eq_ignore_ascii_case(&profile.user_name));

    if let Some(profile) = matched_profile {
        let is_suppressed = {
            let cache = states.suppression_cache.read().await;
            cache.contains(&profile.user_email.to_lowercase())
        };
        if is_suppressed {
            tracing::info!("Skipping password reset email to suppressed address: {}", profile.user_email);
            return HttpResponse::Ok().json(json!({
                "message": "If an account exists for this email, a password reset email has been sent."
            }));
        }

        let new_password = generate_random_password();
        let user_num = profile.user_num;
        match services::smtp::send_updated_password(
            &profile.user_name,
            &profile.user_email,
            &new_password,
        )
        .await
        {
            Ok(_) => {
                match db
                    .user_db
                    .update_user_password_forgot(user_num, &new_password)
                    .await
                {
                    Ok(_) => {
                        ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                        HttpResponse::Ok().json(json!({
                        "message": "If an account exists for this email, a password reset email has been sent."
                        }));
                    }
                    Err(e) => {
                        error!("Failed to update password for user {}: {:?}", user_num, e);
                        HttpResponse::InternalServerError()
                            .json(json!({ "message": "Failed to update password" }));
                    }
                }
            }
            Err(e) => {
                error!("Failed to send new password for user {}: {:?}", user_num, e);
                HttpResponse::InternalServerError()
                    .json(json!({ "message": "Failed to send new password" }));
            }
        }
    } else if !profiles.is_empty() {
        log::warn!(
            "Username mismatch for email {}: request username '{}' didn't match any profiles",
            email,
            username
        );
        HttpResponse::Ok().json(json!({
            "message": "If an account exists for this email, a password reset email has been sent."
        }));
    }

    // Always return the same response regardless of outcome
    HttpResponse::Ok().json(json!({
        "message": "If an account exists for this email, a password reset email has been sent."
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-email",
    tag = "Auth",
    security(("cookie_session" = [])),
    request_body = ChangeEmailRequest,
    responses(
        (status = 200, description = "Email updated successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Failed to update email")
    )
)]
#[post("/change-email")]
pub async fn change_email(
    data: web::Json<ChangeEmailRequest>,
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    let request_data = data.into_inner();
    let user_profile: Option<ProfileData> = match session.get("user_data") {
        Ok(data) => data,
        Err(e) => {
            error!("Error retrieving session data: {}", e);
            None
        }
    };

    if let Some(profile) = user_profile {
        let user_num = profile.user_num;
        let pincode = &request_data.pincode;
        let new_email = &request_data.email;

        if pincode.is_empty() {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Pincode cannot be empty" }));
        }

        if new_email.is_empty() {
            return HttpResponse::BadRequest().json(json!({ "message": "Email cannot be empty" }));
        }

        match db
            .user_db
            .update_user_email(user_num, pincode, new_email)
            .await
        {
            Ok(()) => {
                // Update the session cache with the new email and reset verification
                let mut updated_profile = profile;
                updated_profile.user_email = new_email.clone();
                updated_profile.is_verified = false;
                if let Err(e) = session.insert("user_data", &updated_profile) {
                    error!("Failed to update session with new email: {}", e);
                }
                ran_db_core::webmanager::user::cache::invalidate_profile(user_num);

                // Send verification email to the new address (non-fatal)
                match services::verification::create_verification_token(user_num, new_email) {
                    Ok(jwt) => {
                        if let Ok((user_email, username)) =
                            db.web_db.save_verification_token(user_num, &jwt).await
                        {
                            let is_suppressed = {
                                let cache = states.suppression_cache.read().await;
                                cache.contains(&user_email.to_lowercase())
                            };
                            if is_suppressed {
                                tracing::info!("Skipping verification email to suppressed address: {}", user_email);
                                let _ = db.web_db.log_email_sent(user_num, "verification", &user_email, "suppressed", None).await;
                            } else {
                                let config = services::smtp::SmtpConfig::from_env().ok();
                                let domain = config
                                    .map(|c| c.web_domain)
                                    .unwrap_or_else(|| "https://www.ranonlinegs.com".to_string());
                                let link = format!("{}/verify-email?token={}", domain, jwt);
                                let send_result = services::smtp::send_verification_email(
                                    &username,
                                    &user_email,
                                    &link,
                                )
                                .await;
                                let (status, err_msg) = match &send_result {
                                    Ok(_) => ("sent", None),
                                    Err(e) => {
                                        error!("Failed to send verification email after email change for user {}: {:?}", user_num, e);
                                        ("failed", Some(e.to_string()))
                                    }
                                };
                                let _ = db
                                    .web_db
                                    .log_email_sent(
                                        user_num,
                                        "verification",
                                        &user_email,
                                        status,
                                        err_msg.as_deref(),
                                    )
                                    .await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create verification token after email change for user {}: {:?}", user_num, e);
                    }
                }

                HttpResponse::Ok().json(json!({ "message": "Email updated successfully" }))
            }
            Err(err) => HttpResponse::BadRequest().json(json!({
                "message": "Failed to change email",
                "error": err.to_string()
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "message": "Unauthorized" }))
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    tag = "Auth",
    security(("cookie_session" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password updated successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Failed to update password")
    )
)]
#[post("/change-password")]
pub async fn change_password(
    data: web::Json<ChangePasswordRequest>,
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let request_data = data.into_inner();
    let user_profile: Option<ProfileData> = match session.get("user_data") {
        Ok(data) => data,
        Err(e) => {
            error!("Error retrieving session data: {}", e);
            None
        }
    };

    if let Some(profile) = user_profile {
        // Sanitize before passing data
        let user_num = profile.user_num;
        let pincode = &request_data.pincode;
        let old_password = &request_data.old_password;
        let new_password = &request_data.new_password;
        // Sanitize before passing data

        if new_password.is_empty() || old_password.is_empty() || pincode.is_empty() {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Some fields cannot be empty" }));
        }

        match db
            .user_db
            .update_user_password(user_num, pincode, old_password, new_password)
            .await
        {
            Ok(()) => {
                ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                HttpResponse::Ok().json(json!({ "message": "Password updated successfully" }))
            }
            Err(err) => HttpResponse::BadRequest().json(json!({
                "message": "Failed to change password",
                "error": err.to_string()
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "message": "Unauthorized" }))
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-pincode",
    tag = "Auth",
    security(("cookie_session" = [])),
    request_body = ChangePincodeRequest,
    responses(
        (status = 200, description = "Pincode updated successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Failed to update pincode")
    )
)]
#[post("/change-pincode")]
pub async fn change_pincode(
    data: web::Json<ran_models::users::ChangePincodeRequest>,
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let user_profile: Option<ProfileData> = match session.get("user_data") {
        Ok(data) => data,
        Err(e) => {
            error!("Error retrieving session data: {}", e);
            None
        }
    };

    if let Some(profile) = user_profile {
        let user_num = profile.user_num;
        let old_pincode = &data.old_pincode;
        let new_pincode = &data.new_pincode;

        if new_pincode.is_empty() {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Pincode cannot be empty" }));
        }

        match db
            .user_db
            .update_user_pincode(user_num, old_pincode, new_pincode)
            .await
        {
            Ok(()) => {
                ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                HttpResponse::Ok().json(json!({ "message": "Pincode updated successfully" }))
            }
            Err(err) => HttpResponse::BadRequest().json(json!({
                "message": "Failed to change pincode",
                "error": err.to_string()
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "message": "Unauthorized" }))
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-pincode",
    tag = "Auth",
    security(("cookie_session" = [])),
    responses(
        (status = 200, description = "New pincode sent to email", body = serde_json::Value),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Failed to reset pincode")
    )
)]
#[post("/reset-pincode")]
pub async fn reset_pincode(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    let user_profile: Option<ProfileData> = match session.get("user_data") {
        Ok(data) => data,
        Err(e) => {
            error!("Error retrieving session data: {}", e);
            None
        }
    };

    if let Some(profile) = user_profile {
        let is_suppressed = {
            let cache = states.suppression_cache.read().await;
            cache.contains(&profile.user_email.to_lowercase())
        };
        if is_suppressed {
            tracing::info!("Skipping pincode reset email to suppressed address: {}", profile.user_email);
            return HttpResponse::Ok().json(json!({
                "message": "Your new account pincode has been sent to your registered email address."
            }));
        }

        let new_pincode = generate_random_pincode();
        let user_num = profile.user_num;

        match services::smtp::send_updated_pincode(&profile.user_name, &profile.user_email, &new_pincode)
            .await {
                Ok(_) => {
                    match db
                        .user_db
                        .reset_user_pincode(user_num, &new_pincode)
                        .await {
                            Ok(_) => {
                                ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
                                return HttpResponse::Ok().json(json!({
                                    "message": "Your new account pincode has been sent to your registered email address."
                                }));
                            }
                            Err(e) => {
                                error!("Failed to reset pincode for user {}: {:?}", user_num, e);
                                return HttpResponse::InternalServerError()
                                    .json(json!({ "message": "Failed to reset pincode" }));
                            }
                        }
                }
                Err(e) => {
                    error!("Failed to send new pincode for user {}: {:?}", user_num, e);
                    return HttpResponse::InternalServerError()
                        .json(json!({ "message": "Failed to send new pincode" }));
                }
            }
    } else {
        return HttpResponse::Unauthorized().json(json!({ "message": "Unauthorized" }));
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/validate-username",
    tag = "Auth",
    request_body = ValidateUsernameRequest,
    responses(
        (status = 200, description = "Username availability check result", body = bool),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Failed to check username")
    )
)]
#[post("/validate-username")]
pub async fn validate_username(
    data: web::Json<ValidateUsernameRequest>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    if let Err(validation_errors) = data.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    match db.web_db.check_username_exists(&data.username).await {
        Ok(exists) => HttpResponse::Ok().json(!exists),
        Err(e) => {
            error!("Failed to check username availability: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "message": "Failed to check username availability" }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/validate-email",
    tag = "Auth",
    request_body = ValidateEmailRequest,
    responses(
        (status = 200, description = "Email availability check result", body = bool),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Failed to check email")
    )
)]
#[post("/validate-email")]
pub async fn validate_email(
    data: web::Json<ValidateEmailRequest>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    if let Err(validation_errors) = data.validate() {
        let flat_errors = flatten_validation_errors(&validation_errors);
        return HttpResponse::BadRequest().json(flat_errors);
    }

    match db.web_db.check_email_exists(&data.email).await {
        Ok(exists) => HttpResponse::Ok().json(!exists),
        Err(e) => {
            error!("Failed to check email availability: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "message": "Failed to check email availability" }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/verify-email",
    tag = "Auth",
    params(
        ("token" = String, Query, description = "JWT verification token")
    ),
    responses(
        (status = 200, description = "Email verification result", body = serde_json::Value),
        (status = 400, description = "Invalid or expired token")
    )
)]
#[get("/verify-email")]
pub async fn verify_email(
    query: web::Query<VerifyEmailRequest>,
    db: web::Data<Arc<DBState>>,
) -> impl Responder {
    let claims = match services::verification::decode_verification_token(&query.token) {
        Ok(claims) => claims,
        Err(e) => {
            let msg = if e.to_string().contains("expired") {
                "Verification link has expired. Please request a new one."
            } else {
                "Invalid verification link."
            };
            return HttpResponse::BadRequest().json(json!({
                "verified": false,
                "message": msg,
            }));
        }
    };

    match db
        .web_db
        .verify_email_token(claims.sub, &query.token)
        .await
    {
        Ok((true, message)) => {
            ran_db_core::webmanager::user::cache::invalidate_profile(claims.sub);
            let _ = db
                .web_db
                .log_email_sent(claims.sub, "verification", &claims.email, "verified", None)
                .await;
            HttpResponse::Ok().json(json!({
                "verified": true,
                "message": message,
            }))
        }
        Ok((false, message)) => {
            HttpResponse::BadRequest().json(json!({
                "verified": false,
                "message": message,
            }))
        }
        Err(e) => {
            error!("Failed to verify email token: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "verified": false,
                "message": "An error occurred during verification.",
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification",
    tag = "Auth",
    security(("cookie_session" = [])),
    responses(
        (status = 200, description = "Verification email sent", body = serde_json::Value),
        (status = 400, description = "Already verified"),
        (status = 429, description = "Too many requests"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Failed to send email")
    )
)]
#[post("/resend-verification")]
pub async fn resend_verification(
    session: actix_session::Session,
    db: web::Data<Arc<DBState>>,
    states: web::Data<Arc<AppState>>,
) -> impl Responder {
    let profile = match crate::middleware::session::get_user_session(session).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    if profile.is_verified {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Email is already verified" }));
    }

    let user_num = profile.user_num;

    match db
        .web_db
        .get_recent_verification_email_count(user_num, 60)
        .await
    {
        Ok(count) if count >= 3 => {
            return HttpResponse::TooManyRequests().json(json!({
                "message": "Too many verification email requests. Please try again later."
            }));
        }
        Err(e) => {
            error!("Failed to check verification email rate limit: {:?}", e);
        }
        _ => {}
    }

    let is_suppressed = {
        let cache = states.suppression_cache.read().await;
        cache.contains(&profile.user_email.to_lowercase())
    };
    if is_suppressed {
        tracing::info!("Skipping resend verification to suppressed address: {}", profile.user_email);
        let _ = db.web_db.log_email_sent(user_num, "resend_verification", &profile.user_email, "suppressed", None).await;
        return HttpResponse::Ok().json(json!({ "message": "Verification email sent" }));
    }

    let jwt = match services::verification::create_verification_token(user_num, &profile.user_email) {
        Ok(jwt) => jwt,
        Err(e) => {
            error!("Failed to create verification token: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "message": "Failed to generate verification token" }));
        }
    };

    if let Err(e) = db.web_db.save_verification_token(user_num, &jwt).await {
        error!("Failed to save verification token: {:?}", e);
        return HttpResponse::InternalServerError()
            .json(json!({ "message": "Failed to save verification token" }));
    }

    let config = services::smtp::SmtpConfig::from_env().ok();
    let domain = config
        .map(|c| c.web_domain)
        .unwrap_or_else(|| "https://www.ranonlinegs.com".to_string());
    let link = format!("{}/verify-email?token={}", domain, jwt);

    match services::smtp::send_verification_email(
        &profile.user_name,
        &profile.user_email,
        &link,
    )
    .await
    {
        Ok(_) => {
            let _ = db
                .web_db
                .log_email_sent(
                    user_num,
                    "resend_verification",
                    &profile.user_email,
                    "sent",
                    None,
                )
                .await;
            ran_db_core::webmanager::user::cache::invalidate_profile(user_num);
            HttpResponse::Ok().json(json!({ "message": "Verification email sent" }))
        }
        Err(e) => {
            error!("Failed to send verification email: {:?}", e);
            let _ = db
                .web_db
                .log_email_sent(
                    user_num,
                    "resend_verification",
                    &profile.user_email,
                    "failed",
                    Some(&e.to_string()),
                )
                .await;
            HttpResponse::InternalServerError()
                .json(json!({ "message": "Failed to send verification email" }))
        }
    }
}

fn generate_random_password() -> String {
    let password_length = 20;
    // Allowed characters: uppercase, lowercase, digits, and some special characters.
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                              abcdefghijklmnopqrstuvwxyz\
                              0123456789"
        .chars()
        .collect();

    let mut rng = rng();
    (0..password_length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx]
        })
        .collect()
}

fn generate_random_pincode() -> String {
    let pincode_length = 6;
    // Allowed characters: uppercase, lowercase, digits, and some special characters.
    let charset: Vec<char> = "0123456789"
        .chars()
        .collect();

    let mut rng = rng();
    (0..pincode_length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx]
        })
        .collect()
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(authenticate)
        .service(logout)
        .service(validate)
        .service(forgot_password)
        .service(register)
        .service(change_email)
        .service(change_password)
        .service(change_pincode)
        .service(reset_pincode)
        .service(verify_email)
        .service(
            web::scope("")
                .wrap(crate::services::rate_limit::resend_verification_rate_limiter())
                .service(resend_verification),
        )
        .service(
            web::scope("")
                .wrap(crate::services::rate_limit::validation_rate_limiter())
                .service(validate_username)
                .service(validate_email),
        );
}
