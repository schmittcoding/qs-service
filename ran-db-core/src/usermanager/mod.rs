
use crate::{DBManager, DatabaseTrait};
use anyhow::{Context, Result};
use chrono::Utc;
use deadpool_tiberius::Pool;
use ran_models::{admin::players::{BanAccountHistory, UserLoginHistoryRecord}, errors::DatabaseError, users::ProfileData, BaseDataCountResult};
use tiberius::Row;
use tracing::{error, info};

#[derive(Clone)]
pub struct UserDB {
    pub pool: Pool,
}

#[async_trait::async_trait]
impl DatabaseTrait for UserDB {
    async fn new(db_manager: &DBManager) -> Result<Self, DatabaseError> {
        let pool = db_manager
            .get_database("user")
            .ok_or(DatabaseError::UserNotFound)?;
        Ok(Self { pool: pool.clone() })
    }

    async fn test_db_connection(&self) -> Result<(), DatabaseError> {
        let mut client = self.pool.get().await?;
        let stream = client.query("SELECT 1", &[]).await?;
        let row = stream
            .into_row()
            .await?
            .ok_or(DatabaseError::TestConnectionFailed)?;

        match row.get::<i32, _>(0) {
            Some(_) => {
                info!("✅ User DB connection verified");
                Ok(())
            }
            _ => {
                error!("Unexpected test value");
                Err(DatabaseError::TestConnectionFailed)
            }
        }
    }
}

impl UserDB {
    pub async fn reset_user_pincode(
        &self,
        user_num: i64,
        new_pincode: &str
    ) -> Result<()> {
        // Get a connection from the pool
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Define the SQL command to call the stored procedure
        let sql = "EXEC ResetUserPincode @UserNum = @P1, @NewPinCode = @P2";

        // Execute the stored procedure
        let stream = client
            .query(sql, &[&user_num, &new_pincode])
            .await
            .context("Failed to execute ResetUserPincode procedure")?;

        let result = stream
            .into_first_result()
            .await
            .context("Failed to fetch rows from stream")?;

        if let Some(row) = result.get(0) {
            // Assume IsChanged is returned as an integer field.
            let is_changed: Option<i32> = row
                .try_get("IsChanged")
                .context("Failed to read 'IsChanged' from row")?;
            let message = row
                .try_get::<&str, _>("Message")
                .context("Failed to read 'Message' from row")?
                .map(|s| s.to_owned());
            if is_changed.unwrap_or(0) == 0 {
                let error_message = message.unwrap_or("Update failed".to_owned());
                anyhow::bail!(error_message);
            }
        } else {
            anyhow::bail!("No response from ResetUserPincode procedure");
        }

        Ok(())
    }

    pub async fn update_user_pincode(
        &self,
        user_num: i64,
        old_pincode: &str,
        new_pincode: &str,
    ) -> Result<()> {
        // Get a connection from the pool
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Define the SQL command to call the stored procedure
        let sql = "EXEC UpdateUserPincode @UserNum = @P1, @OldPinCode = @P2, @NewPinCode = @P3";

        // Execute the stored procedure
        let stream = client
            .query(sql, &[&user_num, &old_pincode, &new_pincode])
            .await
            .context("Failed to execute UpdateUserPincode procedure")?;

        let result = stream
            .into_first_result()
            .await
            .context("Failed to fetch rows from stream")?;

        if let Some(row) = result.get(0) {
            // Assume IsChanged is returned as an integer field.
            let is_changed: Option<i32> = row
                .try_get("IsChanged")
                .context("Failed to read 'IsChanged' from row")?;
            let message = row
                .try_get::<&str, _>("Message")
                .context("Failed to read 'Message' from row")?
                .map(|s| s.to_owned());
            if is_changed.unwrap_or(0) == 0 {
                let error_message = message.unwrap_or("Update failed".to_owned());
                anyhow::bail!(error_message);
            }
        } else {
            anyhow::bail!("No response from UpdateUserPincode procedure");
        }

        Ok(())
    }

    pub async fn update_user_email(
        &self,
        user_num: i64,
        pincode: &str,
        new_email: &str,
    ) -> Result<()> {
        // Get a connection from the pool
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Define the SQL command to call the stored procedure
        let sql = "EXEC UpdateUserEmail @UserNum = @P1, @Pincode = @P2, @NewEmail = @P3";

        let stream = client
            .query(sql, &[&user_num, &pincode, &new_email])
            .await
            .context("Failed to execute UpdateUserEmail procedure")?;

        let result = stream
            .into_first_result()
            .await
            .context("Failed to fetch rows from stream")?;

        if let Some(row) = result.get(0) {
            // Assume IsChanged is returned as an integer field.
            let is_changed: Option<i32> = row
                .try_get("IsChanged")
                .context("Failed to read 'IsChanged' from row")?;
            let message = row
                .try_get::<&str, _>("Message")
                .context("Failed to read 'Message' from row")?
                .map(|s| s.to_owned());
            if is_changed.unwrap_or(0) == 0 {
                let error_message = message.unwrap_or("Update failed".to_owned());
                anyhow::bail!(error_message);
            }
        } else {
            anyhow::bail!("No response from UpdateUserEmail procedure");
        }

        Ok(())
    }

    pub async fn update_user_password_forgot(
        &self,
        user_num: i64,
        new_password: &str,
    ) -> Result<bool> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sql = "EXEC UpdateUserPasswordForgot @UserNum = @P1, @NewPassword = @P2";

        client
            .query(sql, &[&user_num, &new_password])
            .await
            .context("Failed to execute UpdateUserPasswordForgot procedure")?;

        Ok(true)
    }

    pub async fn update_user_password(
        &self,
        user_num: i64,
        pincode: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Get a connection from the pool
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Define the SQL command to call the stored procedure
        let sql = "EXEC UpdateUserPassword @UserNum = @P1,@Pincode = @P2, @OldPassword = @P3, @NewPassword = @P4";

        // Execute the stored procedure
        let stream = client
            .query(sql, &[&user_num, &pincode, &old_password, &new_password])
            .await
            .context("Failed to execute UpdateUserPassword procedure")?;

        let result = stream
            .into_first_result()
            .await
            .context("Failed to fetch rows from stream")?;

        if let Some(row) = result.get(0) {
            // Assume IsChanged is returned as an integer field.
            let is_changed: Option<i32> = row
                .try_get("IsChanged")
                .context("Failed to read 'IsChanged' from row")?;
            let message = row
                .try_get::<&str, _>("Message")
                .context("Failed to read 'Message' from row")?
                .map(|s| s.to_owned());
            if is_changed.unwrap_or(0) == 0 {
                let error_message = message.unwrap_or("Update failed".to_owned());
                anyhow::bail!(error_message);
            }
        } else {
            anyhow::bail!("No response from UpdateUserPassword procedure");
        }
        Ok(())
    }

    pub async fn get_user_using_email(&self, email: &str) -> Result<Vec<ProfileData>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC GetProfileWithEmail @UserEmail = @P1";
        let stream = client
            .query(query, &[&(email)])
            .await
            .context("Failed to execute query")?;

        let results = stream
            .into_first_result()
            .await
            .context("Failed to fetch rows from stream")?;
        let mut profiles = Vec::new();
        for row in results {
            // Extract values, allowing None for each field
            let user_num: Option<i32> = row.try_get("UserNum").ok().flatten();
            let user_name: Option<&str> = row.try_get("UserName").ok().flatten();
            let user_email: Option<&str> = row.try_get("UserEmail").ok().flatten();
            let user_available = row
                .try_get::<bool, _>("UserAvailable")
                .ok()
                .flatten()
                .unwrap_or(false) as u8;
            let user_type: Option<i32> = row.try_get("UserType").ok().flatten();

            // Skip rows where all fields are None
            if user_num.is_none()
                && user_name.is_none()
                && user_email.is_none()
            {
                continue;
            }

            profiles.push(ProfileData {
                user_name: user_name.unwrap_or_default().to_string(),
                user_num: user_num.unwrap_or_default() as i64,
                user_email: user_email.unwrap_or_default().to_string(),
                user_available,
                created_at: Utc::now().with_timezone(&chrono::Local),
                last_updated: Utc::now().with_timezone(&chrono::Local),
                user_type: user_type.unwrap_or(1),
                web_points: 0.0,
                mileage_points: 0.0,
                is_verified: false,
            });
        }

        Ok(profiles)
    }

    pub async fn ban_unban_account(
        &self,
        user_num: i64,
        admin_id: String,
        status: i16,
        block_date: Option<String>,
        notes: Option<String>
    ) -> Result<Row> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = match status {
            0 | 1 => "EXEC sp_BanUserAccount @UserNum = @P1, @Status = @P2, @AdminID = @P3, @Notes = @P4, @BlockDate = @P5".to_string(),
            _ => {
                return Err(anyhow::anyhow!("Invalid status: {}", status))
            }
        };

        let results = client
            .query(sp_query, &[
                &user_num,
                &status,
                &admin_id,
                &notes,
                &block_date
            ])
            .await?
            .into_row()
            .await?;

        match results {
            Some(row) => Ok(row),
            None => Err(anyhow::anyhow!("Failed to fetch row"))
        }
    }

    pub async fn ban_account_history(
        &self,
        user_num: i64,
        page: i16,
        page_size: i16
    ) -> Result<BaseDataCountResult<BanAccountHistory>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_BanAccountHistory @UserNum = @P1, @Page = @P2, @PageSize = @P3";

        let results = client
            .query(sp_query, &[&user_num, &page, &page_size])
            .await?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();

        let total_count_rows = result_sets.next().unwrap_or_default();
        let total_records = total_count_rows
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);


        let history = result_sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| BanAccountHistory::from_row(&row))
            .collect();

        Ok(
            BaseDataCountResult {
                data: history,
                total_records: total_records
            }
        )

        // let mut history_vec = Vec::new();

        // while let Some(row_result) = stream.next().await {
        //     let row = row_result.context("Error processing row")?;
        //     if let Some(row_data) = row.into_row() {
        //         history_vec.push(BanAccountHistory::from_row(&row_data))
        //     }
        // }

        // Ok(history_vec)
    }

    pub async fn get_login_history(
        &self,
        user_num: i64,
        page: i16,
        page_size: i16
    ) -> Result<BaseDataCountResult<UserLoginHistoryRecord>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetUserLoginHistory @UserNum = @P1, @Page = @P2, @PageSize = @P3";

        let results = client
            .query(sp_query, &[&user_num, &page, &page_size])
            .await
            .context("Query execution failed")?
            .into_results()
            .await
            .context("Failed to fetch user login history")?;

        let mut result_sets = results.into_iter();

        let total_count_rows = result_sets.next().unwrap_or_default();
        let total_records = total_count_rows
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let history = result_sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| UserLoginHistoryRecord::from_row(&row))
            .collect();

        Ok(
            BaseDataCountResult {
                data: history,
                total_records: total_records
            }
        )
    }

    pub async fn get_online_account_analytics(&self) -> Result<Vec<String>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT UserID FROM [dbo].[GSUserInfo] WHERE UserLoginState=1";

        let stream = client
            .query(query, &[])
            .await?
            .into_first_result()
            .await?;

        let user_ids: Vec<String> = stream
            .iter()
            .map(|row| row.get::<&str, _>("UserID").unwrap_or_default().to_string())
            .collect();

        Ok(user_ids)
    }
}

#[async_trait::async_trait]
pub trait UserAdminMetricsTrait {
    async fn get_online_player_count(&self) -> Result<i32>;
}

#[async_trait::async_trait]
impl UserAdminMetricsTrait for UserDB {
    async fn get_online_player_count(&self) -> Result<i32> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC sp_GetOnlinePlayerCount";
        let row = client
            .query(query, &[])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;

        match row {
            Some(r) => {
                let count = r
                    .try_get::<i32, _>("OnlineCount")
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                Ok(count)
            }
            None => Ok(0),
        }
    }
}
