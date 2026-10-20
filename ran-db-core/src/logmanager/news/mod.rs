use super::LogDB;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait NewLogTrait {
    async fn log_news_action(
        &self,
        news_id: Uuid,
        action: &str,
        action_by: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
impl NewLogTrait for LogDB {
    async fn log_news_action(
        &self,
        news_id: Uuid,
        action: &str,
        action_by: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log_id = Uuid::new_v4();

        let query = "INSERT INTO [RanWeb].[dbo].[WebNewsLog] 
            (LogID, NewsID, ActionType, ChangedBy) 
            VALUES ($1, $2, $3, $4)";

        let mut client = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("❌ Failed to get database connection: {:?}", e);
                return Err(Box::new(e));
            }
        };

        match client
            .execute(query, &[&log_id, &news_id, &action, &action_by])
            .await
        {
            Ok(_) => {
                tracing::info!("✅ News action logged successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Failed to add web download data: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}
