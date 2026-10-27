pub mod cache;

use super::WebDB;
use crate::metrics;
use anyhow::{Context, Result};
use chrono::Utc;
use ran_models::users::{ProfileData, RegistrationRequest};
use secrecy::{ExposeSecret, SecretString};
use tiberius::numeric::Numeric;
use tracing::{debug, error, instrument};

#[async_trait::async_trait]
pub trait WebUserTrait {
    async fn get_profile_from_db(&self, user_num: i64) -> Result<Option<ProfileData>>;
    async fn register_user(&self, register_data: RegistrationRequest) -> Result<i64>;
    async fn check_username_exists(&self, username: &str) -> Result<bool>;
    async fn check_email_exists(&self, email: &str) -> Result<bool>;
    async fn save_verification_token(&self, user_num: i64, token: &str) -> Result<(String, String)>;
    async fn verify_email_token(&self, user_num: i64, token: &str) -> Result<(bool, String)>;
}

#[async_trait::async_trait]
impl WebUserTrait for WebDB {
    #[instrument(skip(self), level = "debug", err)]
    async fn get_profile_from_db(&self, user_num: i64) -> Result<Option<ProfileData>> {
        if user_num <= 0 {
            anyhow::bail!("Invalid user_num: must be positive");
        }

        if let Some(cached) = cache::get_profile(user_num) {
            metrics::record_query_success("get_profile_cache_hit");
            return Ok(Some(cached));
        }

        metrics::record_query_success("get_profile_cache_miss");
        let mut timer = metrics::QueryTimer::new("get_profile");

        let mut client = self
            .pool
            .get()
            .await
            .inspect_err(|_| timer.finish_failure())
            .context("Failed to get DB connection")?;

        let row = client
            .query("EXEC GetProfileData @UserNum = @P1", &[&user_num])
            .await
            .inspect_err(|_| timer.finish_failure())
            .context("Failed to execute GetProfileData")?
            .into_row()
            .await
            .inspect_err(|_| timer.finish_failure())
            .context("Failed to fetch row from stream")?;

        let Some(row) = row else {
            timer.finish_success();
            return Ok(None);
        };

        let profile = ProfileData {
            user_name: row.try_get::<&str, _>("UserName").ok().flatten().unwrap_or_default().to_string(),
            user_num,
            user_email: row.try_get::<&str, _>("UserEmail").ok().flatten().unwrap_or_default().to_string(),
            user_available: row
                .try_get::<bool, _>("UserAvailable")
                .ok()
                .flatten()
                .unwrap_or(false) as u8,
            user_type: row.try_get("UserType").ok().flatten().unwrap_or(1),
            web_points: row.try_get("WebPoints").ok().flatten().unwrap_or_default(),
            mileage_points: row
                .try_get::<Numeric, _>("MileagePoints")
                .ok()
                .flatten()
                .map(f64::from)
                .unwrap_or_default(),
            created_at: row.try_get::<chrono::NaiveDateTime, _>("CreatedAt").ok().flatten()
                .map(|dt| dt.and_local_timezone(chrono::Local).unwrap())
                .unwrap_or_else(|| Utc::now().with_timezone(&chrono::Local)),
            last_updated: row.try_get::<chrono::NaiveDateTime, _>("UpdatedAt").ok().flatten()
                .map(|dt| dt.and_local_timezone(chrono::Local).unwrap())
                .unwrap_or_else(|| Utc::now().with_timezone(&chrono::Local)),
            is_verified: row
                .try_get::<bool, _>("UserVerified")
                .ok()
                .flatten()
                .unwrap_or(false),
        };

        cache::store_profile(profile.clone());
        timer.finish_success();
        Ok(Some(profile))
    }

    async fn register_user(&self, register_data: RegistrationRequest) -> Result<i64> {
        // Perform additional validation if needed
        if register_data.username.is_empty()
            || register_data.password.is_empty()
            || register_data.email.is_empty()
        {
            return Err(anyhow::anyhow!("Missing required fields for registration"));
        }

        debug!("Registering new user: {}", register_data.username);

        // Wrap sensitive data in Secret - clone them first to avoid borrowing issues
        let password_str = register_data.password.as_str();
        let pincode_str = register_data.pincode.as_str();

        let password = SecretString::new(password_str.into());
        let pincode = SecretString::new(pincode_str.into());

        // Get database connection from pool
        let mut client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get DB connection for registration: {}", e);
                return Err(anyhow::anyhow!(
                    "Failed to get DB connection for registration: {}",
                    e
                ));
            }
        };

        // Get database name from environment with more graceful error handling
        let user_db = match std::env::var("USER_DB_NAME") {
            Ok(db_name) => db_name,
            Err(_) => {
                error!("USER_DB_NAME environment variable not set");
                return Err(anyhow::anyhow!("USER_DB_NAME environment variable not set"));
            }
        };

        // Expose sensitive data only for the duration of the query
        let password_exposed = password.expose_secret();
        let pincode_exposed = pincode.expose_secret();

        // GameRegister manages its own transaction; wrapping it here would leave a
        // mismatched transaction count when the procedure rolls back on a failed check.
        let sql = "EXEC GameRegister @UserFullName=@P1, @UserID=@P2, @UserPass=@P3, @UserPin=@P4, @UserEmail=@P5, @RanDatabaseName=@P6";

        let stream = match client
            .query(
                sql,
                &[
                    &register_data.full_name,
                    &register_data.username,
                    &password_exposed,
                    &pincode_exposed,
                    &register_data.email,
                    &user_db,
                ],
            )
            .await
        {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to execute GameRegister procedure: {}", e);
                return Err(anyhow::anyhow!(
                    "Failed to execute GameRegister procedure: {}",
                    e
                ));
            }
        };

        let row = match stream.into_row().await {
            Ok(row) => row,
            Err(e) => {
                error!("Failed to fetch row from GameRegister procedure: {}", e);
                return Err(anyhow::anyhow!(
                    "Failed to fetch row from GameRegister procedure: {}",
                    e
                ));
            }
        };

        if let Some(row) = row {
            // Extract the "Result" column from the returned row.
            let result_code: Option<i32> = row.try_get("Result").ok().flatten();

            match result_code {
                Some(1) => {
                    let user_num: i64 = row.try_get::<i32, _>("UserNum").ok().flatten()
                        .context("GameRegister did not return UserNum")? as i64;
                    debug!("User registration successful, user_num: {}", user_num);
                    Ok(user_num)
                }
                Some(0) => {
                    // Duplicate key detected.
                    error!("User registration failed: Duplicate user detected");
                    Err(anyhow::anyhow!("Duplicate user detected"))
                }
                None => {
                    error!("User registration failed: GameRegister returned NULL result");
                    Err(anyhow::anyhow!("GameRegister returned NULL result"))
                }
                Some(code) => {
                    // Unexpected result code.
                    error!(
                        "User registration failed: Unexpected result code: {:#?}",
                        code
                    );
                    Err(anyhow::anyhow!("Unexpected result code: {:#?}", code))
                }
            }
        } else {
            error!("GameRegister stored procedure returned no data");
            Err(anyhow::anyhow!(
                "GameRegister stored procedure returned no data"
            ))
        }
    }

    async fn check_username_exists(&self, username: &str) -> Result<bool> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT COUNT(1) AS cnt FROM RanWeb.dbo.WebUser WHERE UserName = @P1";
        let row = client
            .query(query, &[&username])
            .await
            .context("Failed to execute username check query")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(row) => {
                let count: Option<i32> = row.try_get("cnt").ok().flatten();
                Ok(count.unwrap_or(0) > 0)
            }
            None => Ok(false),
        }
    }

    async fn check_email_exists(&self, email: &str) -> Result<bool> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT COUNT(1) AS cnt FROM RanWeb.dbo.WebUser WHERE UserEmail = @P1";
        let row = client
            .query(query, &[&email])
            .await
            .context("Failed to execute email check query")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(row) => {
                let count: Option<i32> = row.try_get("cnt").ok().flatten();
                Ok(count.unwrap_or(0) > 0)
            }
            None => Ok(false),
        }
    }

    #[instrument(skip(self), level = "debug", err)]
    async fn save_verification_token(&self, user_num: i64, token: &str) -> Result<(String, String)> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let row = client
            .query(
                "EXEC SaveVerificationToken @UserNum = @P1, @Token = @P2",
                &[&user_num, &token],
            )
            .await
            .context("Failed to execute SaveVerificationToken")?
            .into_row()
            .await
            .context("Failed to fetch row")?
            .context("SaveVerificationToken returned no data")?;

        let email = row
            .try_get::<&str, _>("UserEmail")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string();
        let username = row
            .try_get::<&str, _>("UserName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string();

        Ok((email, username))
    }

    #[instrument(skip(self), level = "debug", err)]
    async fn verify_email_token(&self, user_num: i64, token: &str) -> Result<(bool, String)> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let row = client
            .query(
                "EXEC VerifyEmailToken @UserNum = @P1, @Token = @P2",
                &[&user_num, &token],
            )
            .await
            .context("Failed to execute VerifyEmailToken")?
            .into_row()
            .await
            .context("Failed to fetch row")?
            .context("VerifyEmailToken returned no data")?;

        let is_verified: bool = row
            .try_get::<i32, _>("IsVerified")
            .ok()
            .flatten()
            .unwrap_or(0)
            == 1;
        let message = row
            .try_get::<&str, _>("Message")
            .ok()
            .flatten()
            .unwrap_or("Unknown error")
            .to_string();

        Ok((is_verified, message))
    }
}
