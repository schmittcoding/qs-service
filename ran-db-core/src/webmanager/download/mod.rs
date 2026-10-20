use super::WebDB;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use ran_models::{download::{DownloadItem, DownloadItemPayload}};
use tracing::info;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait WebDownloadTrait {
    async fn get_download_links(&self, status: Option<u8>, r#type: Option<i16>) -> Result<Vec<DownloadItem>>;
    async fn add_download_link(&self, data: DownloadItemPayload, admin_id: String) -> Result<(String, Uuid)>;
    async fn edit_download_link(&self, id: Uuid, data: DownloadItemPayload, admin_id: String) -> Result<String>;
}

#[async_trait::async_trait]
impl WebDownloadTrait for WebDB {
    async fn get_download_links(&self, status: Option<u8>, r#type: Option<i16>) -> Result<Vec<DownloadItem>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = r#"
            SELECT * FROM WebDownload
            WHERE 
               (@P1 IS NULL OR WebDownloadStatus = @P1) AND
               (@P2 IS NULL OR WebDownloadType = @P2)
            ORDER BY WebDownloadVersion DESC
        "#;

        let mut stream = client
            .query(query, &[
                &status,
                &r#type,
            ])
            .await
            .context("Failed to execute query")?;

        let mut download_links = Vec::new();
        while let Some(result) = stream.next().await {
            let row = result.context("Error processing row")?;
            if let Some(row_data) = row.as_row() {
                download_links.push(DownloadItem::from_row(row_data));
            }
        }

        info!("✅ Retrieved {} download link(s)", download_links.len());
        Ok(download_links)
    }

    async fn add_download_link(&self, data: DownloadItemPayload, admin_id: String) -> Result<(String, Uuid)> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_AddDownloadItem @Provider = @P1, @Link = @P2, @Version = @P3, @Status = @P4, @Type = @P5, @AdminID = @P6";

        let result = client
            .query(sp_query, &[
                &data.provider,
                &data.link,
                &data.version,
                &data.status,
                &data.r#type,
                &admin_id
            ])
            .await?
            .into_row()
            .await?;

        match result {
            Some(row) => {
                let message = row
                    .try_get::<&str, _>("Message")
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                let id = if message == "SUCCESS" {
                    row
                        .get::<Uuid, _>("WebDownloadID")
                        .unwrap_or_else(Uuid::nil)
                } else {
                    Uuid::nil() 
                };

                Ok((message.to_string(), id))
            },
            None => {
                anyhow::bail!("No response from sp_AddDownloadItem procedure");
            }
        }
    }

    async fn edit_download_link(&self, id: Uuid, data: DownloadItemPayload, admin_id: String) -> Result<String> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_EditDownloadItem @Id = @P1, @Provider = @P2, @Link = @P3, @Version = @P4, @Status = @P5, @Type = @P6, @AdminID = @P7";

        let result = client
            .query(sp_query, &[
                &id,
                &data.provider,
                &data.link,
                &data.version,
                &data.status,
                &data.r#type,
                &admin_id
            ])
            .await?
            .into_row()
            .await?;

        match result {
            Some(row) => {
                let message = row
                    .try_get::<&str, _>("Message")
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                Ok(message.to_string())
            },
            None => {
                anyhow::bail!("No response from sp_EditDownloadItem procedure");
            }
        }
    }
}