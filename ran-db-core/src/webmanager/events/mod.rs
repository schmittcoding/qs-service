mod deduction;
mod event_management;
mod gimmick;
mod gvg_registration;
mod leaderboard;
mod match_management;
mod match_status;
mod registration;
pub mod registration_validation;
mod snapshots;
pub mod tally;

use crate::webmanager::WebDB;
use anyhow::Result;
use ran_models::errors::DatabaseError;
use ran_models::events::event::EventDefinition;
use uuid::Uuid;

impl WebDB {
    /// Health check for event system
    pub async fn event_health_check(&self) -> Result<bool, DatabaseError> {
        let mut client = self.pool.get().await?;
        let stream = client.query("SELECT 1", &[]).await?;
        let row = stream
            .into_row()
            .await?
            .ok_or(DatabaseError::TestConnectionFailed)?;

        match row.get::<i32, _>(0) {
            Some(1) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get the current active event
    pub async fn get_active_event(&self) -> Result<Option<EventDefinition>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT TOP 1 * FROM EventDefinition WHERE IsActive = 1 ORDER BY CreatedAt DESC",
                &[],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| EventDefinition::from_row(&r)))
    }

    /// Get event by ID
    pub async fn get_event_by_id(&self, event_id: Uuid) -> Result<Option<EventDefinition>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT * FROM EventDefinition WHERE EventID = @P1",
                &[&event_id],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| EventDefinition::from_row(&r)))
    }

    /// Get event by slug
    pub async fn get_event_by_slug(&self, slug: &str) -> Result<Option<EventDefinition>> {
        let mut client = self.pool.get().await?;
        let row = client
            .query(
                "SELECT * FROM EventDefinition WHERE EventSlug = @P1 AND IsActive = 1",
                &[&slug],
            )
            .await?
            .into_row()
            .await?;

        Ok(row.map(|r| EventDefinition::from_row(&r)))
    }

    /// List all events for public display, sorted active/ongoing first then by latest start date
    pub async fn list_events_public(&self) -> Result<Vec<EventDefinition>> {
        let mut client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM EventDefinition WHERE IsActive = 1
                 ORDER BY IsActive DESC, EventStart DESC",
                &[],
            )
            .await?
            .into_first_result()
            .await?;

        Ok(rows.iter().map(|r| EventDefinition::from_row(r)).collect())
    }
}
