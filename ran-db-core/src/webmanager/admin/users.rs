use anyhow::Result;
use ran_models::admin::users::AdminUserProfile;

use crate::webmanager::WebDB;

#[async_trait::async_trait]
pub trait WebAdminUserTrait {
    async fn get_admin_by_id(
        &self,
        admin_id: String,
    ) -> Result<AdminUserProfile>;
}

#[async_trait::async_trait]
impl WebAdminUserTrait for WebDB {
    async fn get_admin_by_id(
        &self,
        admin_id: String,
    ) -> Result<AdminUserProfile> {
        let mut client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                log::error!("Failed to get DB connection: {}", e);
                return Err(anyhow::anyhow!("Failed to get DB connection: {}", e));
            }
        };

        let sp_query = "EXEC sp_GetAdminById @AdminId = @P1";

        let results = client
            .query(sp_query, &[&admin_id])
            .await?
            .into_row()
            .await?;

        match results {
            Some(row) => {
                Ok(AdminUserProfile::from_row(&row))
            }
            None => Err(anyhow::anyhow!("User {} not found", admin_id))
        }
    }
} 