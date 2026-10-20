use anyhow::Result;
use ran_models::{admin::auth::{AuthAdminProfileData}};
use tiberius::Row;
use tracing::{debug, error};
use uuid::Uuid;

use crate::webmanager::WebDB;


#[async_trait::async_trait]
pub trait WebAdminTrait {
    async fn authenticate_admin(&self, username: &str, password: &str) -> Result<Option<AuthAdminProfileData>>;
    async fn get_admin_profile(&self, user_id: Uuid) -> Result<Option<AuthAdminProfileData>>;
}

/// Mock implementation for testing
#[cfg(test)]
pub struct MockWebDB;

#[cfg(test)]
#[async_trait::async_trait]
impl WebAdminTrait for MockWebDB {
    async fn authenticate_admin(&self, username: &str, password: &str) -> Result<Option<AuthAdminProfileData>> {
        if username == "admin" && password == "password123" {
            Ok(Some(AuthAdminProfileData {
                user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                user_name: "admin".to_string(),
                user_type: 1,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_admin_profile(&self, user_id: Uuid) -> Result<Option<AuthAdminProfileData>> {
        let valid_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        if user_id == valid_id {
            Ok(Some(AuthAdminProfileData {
                user_id: user_id.to_string(),
                user_name: "admin".to_string(),
                user_type: 1,
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl WebAdminTrait for WebDB {
    async fn authenticate_admin(&self, username: &str, password: &str) -> Result<Option<AuthAdminProfileData>> {
        let mut client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get DB connection: {}", e);
                return Err(anyhow::anyhow!("Failed to get DB connection: {}", e));
            }
        };

        let query = "EXEC AuthenticateAdmin @UserName = @P1, @UserPass = @P2";

        let stream = match client.query(query, &[&username.to_string(), &password.to_string()]).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to execute AuthenticateAdmin query: {}", e);
                return Err(anyhow::anyhow!("Failed to execute AuthenticateAdmin query: {}", e));
            }
        };

        
        let row: Option<Row> = match stream.into_row().await {
            Ok(row) => row,
            Err(e) => {
                error!("Failed to fetch row from stream: {}", e);
                return Err(anyhow::anyhow!("Failed to fetch row from stream: {}", e));
            }
        };

        if let Some(row) = row {
            debug!("{:?}", row);

            let user_id: Option<Uuid> = row.try_get(0).ok().flatten();
            let user_type: Option<i32> = row.try_get(1).ok().flatten();

            if user_id.is_none() && user_type.is_none() {
                return Ok(None);
            }

            Ok(Some(AuthAdminProfileData {
                user_id: user_id.unwrap_or_default().to_string(),
                user_name: username.to_string(),
                user_type: user_type.unwrap_or_default()
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_admin_profile(&self, user_id: Uuid) -> Result<Option<AuthAdminProfileData>> {
        let mut client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get DB connection: {}", e);
                return Err(anyhow::anyhow!("Failed to get DB connection: {}", e));
            }
        };

        let query = "EXEC sp_GetAdminProfile @UserId = @P1";

        let stream = match client.query(query, &[&user_id]).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to execute sp_GetAdminProfile query: {}", e);
                return Err(anyhow::anyhow!("Failed to execute sp_GetAdminProfile query: {}", e));
            }
        };

        let row: Option<Row> = match stream.into_row().await {
            Ok(row) => row,
            Err(e) => {
                error!("Failed to fetch row from stream: {}", e);
                return Err(anyhow::anyhow!("Failed to fetch row from stream: {}", e));
            }
        };

        if let Some(row) = row {
            debug!("{:?}", row);

            let user_id: Option<Uuid> = row.try_get("UserID").ok().flatten();
            let user_name: Option<&str> = row.try_get("UserName").ok().flatten();
            let user_type: Option<i16> = row.try_get("UserType").ok().flatten();

            if user_id.is_none() && user_name.is_none() && user_type.is_none() {
                return Ok(None);
            }

            Ok(Some(AuthAdminProfileData {
                user_id: user_id.unwrap_or_default().to_string(),
                user_name: user_name.unwrap_or_default().to_string(),
                user_type: user_type.unwrap_or_default() as i32
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Mock Database Trait Tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_authenticate_admin_success() {
        let mock_db = MockWebDB;
        let result = mock_db.authenticate_admin("admin", "password123").await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.user_name, "admin");
        assert_eq!(profile.user_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(profile.user_type, 1);
    }

    #[tokio::test]
    async fn test_mock_authenticate_admin_wrong_password() {
        let mock_db = MockWebDB;
        let result = mock_db.authenticate_admin("admin", "wrongpassword").await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_mock_authenticate_admin_wrong_username() {
        let mock_db = MockWebDB;
        let result = mock_db.authenticate_admin("wronguser", "password123").await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_mock_authenticate_admin_both_wrong() {
        let mock_db = MockWebDB;
        let result = mock_db.authenticate_admin("wronguser", "wrongpassword").await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_mock_get_admin_profile_success() {
        let mock_db = MockWebDB;
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = mock_db.get_admin_profile(user_id).await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.user_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(profile.user_name, "admin");
        assert_eq!(profile.user_type, 1);
    }

    #[tokio::test]
    async fn test_mock_get_admin_profile_not_found() {
        let mock_db = MockWebDB;
        let user_id = Uuid::new_v4(); // random UUID, not in mock data
        let result = mock_db.get_admin_profile(user_id).await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_mock_get_admin_profile_nil_uuid() {
        let mock_db = MockWebDB;
        let user_id = Uuid::nil();
        let result = mock_db.get_admin_profile(user_id).await;

        assert!(result.is_ok());
        let profile = result.unwrap();
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_mock_get_admin_profile_returns_correct_user_id() {
        let mock_db = MockWebDB;
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = mock_db.get_admin_profile(user_id).await;

        let profile = result.unwrap().unwrap();
        // Verify that the returned user_id matches the requested one
        assert_eq!(profile.user_id, user_id.to_string());
    }
}