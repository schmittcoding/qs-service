use super::WebDB;
use anyhow::{Context, Result};
use ran_models::users::AuthResponse;
use tiberius::Row;

#[async_trait::async_trait]
pub trait WebAuthTrait {
    async fn authenticate_user(&self, username: &str, password: &str) -> Result<AuthResponse>;
}

#[async_trait::async_trait]
impl WebAuthTrait for WebDB {
    async fn authenticate_user(&self, username: &str, password: &str) -> Result<AuthResponse> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "EXEC AuthenticateUser @Username = @P1, @Password = @P2";
        let stream = client
            .query(query, &[&username, &password])
            .await
            .context("Query execution failed")?;

        let row: Option<Row> = stream.into_row().await.context("Failed to fetch row")?;

        let (is_authenticated, user_num, message) = match row {
            Some(data) => {
                let is_authenticated: Option<i32> = data.try_get(0).ok().flatten();
                let user_num: Option<i64> = data.try_get(1).ok().flatten();
                let message: Option<String> = data
                    .try_get::<&str, _>(2)
                    .ok()
                    .flatten()
                    .map(|s| s.to_owned());
                (is_authenticated, user_num, message)
            }
            None => (Some(0), None, Some("No data returned".to_string())),
        };

        Ok(AuthResponse {
            is_authenticated,
            user_num,
            message,
        })
    }
}
