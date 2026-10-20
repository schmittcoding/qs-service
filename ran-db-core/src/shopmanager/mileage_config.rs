use anyhow::{Context, Result};
use ran_models::shop::mileage::MileageConfig;

use crate::shopmanager::ShopDB;

#[async_trait::async_trait]
pub trait MileageConfigTrait {
    async fn get_mileage_config(&self) -> Result<Option<MileageConfig>>;
    async fn update_mileage_config(
        &self,
        rate: f64,
        effective_from: Option<&str>,
        effective_to: Option<&str>,
        description: Option<&str>,
        updated_by: &str,
    ) -> Result<MileageConfig>;
}

#[async_trait::async_trait]
impl MileageConfigTrait for ShopDB {
    async fn get_mileage_config(&self) -> Result<Option<MileageConfig>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let result = client
            .query("EXEC sp_GetMileageConfig", &[])
            .await
            .context("Failed to execute sp_GetMileageConfig")?
            .into_row()
            .await?;

        Ok(result.map(|row| MileageConfig::from_row(&row)))
    }

    async fn update_mileage_config(
        &self,
        rate: f64,
        effective_from: Option<&str>,
        effective_to: Option<&str>,
        description: Option<&str>,
        updated_by: &str,
    ) -> Result<MileageConfig> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_UpdateMileageConfig
            @MileageRate = @P1,
            @EffectiveFrom = @P2,
            @EffectiveTo = @P3,
            @Description = @P4,
            @UpdatedBy = @P5";

        let result = client
            .query(sp_query, &[
                &rate,
                &effective_from,
                &effective_to,
                &description,
                &updated_by,
            ])
            .await
            .context("Failed to execute sp_UpdateMileageConfig")?
            .into_row()
            .await?;

        match result {
            Some(row) => Ok(MileageConfig::from_row(&row)),
            None => Err(anyhow::anyhow!("Failed to read updated mileage config"))
        }
    }
}
