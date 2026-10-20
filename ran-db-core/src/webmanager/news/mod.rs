use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime, TimeZone};
use futures_util::StreamExt;
use ran_models::news::{CreateNewsRequest, NewsData};
use regex::Regex;
use tracing::info;

use super::WebDB;

#[async_trait::async_trait]
pub trait WebNewsTrait {
    async fn get_news_data(&self) -> Result<Vec<NewsData>>;
    async fn create_news_data(&self, news_data: CreateNewsRequest) -> Result<()>;
}

#[async_trait::async_trait]
impl WebNewsTrait for WebDB {
    async fn get_news_data(&self) -> Result<Vec<NewsData>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Call the updated stored procedure
        let mut stream = client
            .query("EXEC GetAllNews", &[])
            .await
            .context("Failed to execute query")?;

        let mut news_vec = Vec::new();

        while let Some(result) = stream.next().await {
            let row = result.context("Error processing row")?;
            if let Some(row_data) = row.as_row() {
                // Use the correct column names returned by GetAllNews
                if let Some(id) = row_data.try_get::<i32, _>("NewsId").ok().flatten() {
                    let category = row_data
                        .try_get::<i16, _>("NewsCategory")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let title = row_data
                        .try_get::<&str, _>("NewsTitle")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    // Get the slug or generate one if it's missing
                    let slug: String = row_data
                        .try_get::<&str, _>("NewsSlug")
                        .ok()
                        .flatten()
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| generate_slug(&title));

                    let content = row_data
                        .try_get::<&str, _>("NewsContent")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let author = row_data
                        .try_get::<&str, _>("NewsAuthor")
                        .ok()
                        .flatten()
                        .unwrap_or("Administrator");

                    let status = row_data
                        .try_get::<i16, _>("NewsStatus")
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    let created_at = row_data
                        .try_get::<NaiveDateTime, _>("NewsCreationDate")
                        .ok()
                        .flatten();

                    if let Some(created_at) = created_at {
                        let created_at = Local
                            .from_local_datetime(&created_at)
                            .single()
                            .expect("Ambiguous or invalid local datetime");
                        let news_data = NewsData {
                            news_id: id,
                            news_category: category, // new field
                            news_title: title.to_string(),
                            news_slug: slug,
                            news_content: content.to_string(),
                            news_author: author.to_string(),
                            news_status: status, // new field
                            news_creation_date: created_at,
                        };

                        news_vec.push(news_data);
                    }
                }
            }
        }

        info!("✅ Retrieved {} news articles", news_vec.len());
        Ok(news_vec)
    }

    async fn create_news_data(&self, news_data: CreateNewsRequest) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "INSERT INTO [dbo].[WebNews]
                ([NewsCategory]
                ,[NewsTitle]
                ,[NewsSlug]
                ,[NewsContent]
                ,[NewsAuthor]
                ,[NewsCreationDate]
                ,[NewsStatus]) 
            VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7)";

        let slug = generate_slug(&news_data.title); // Generate a unique slug

        tracing::debug!(
            "Inserting news data: category={}, title={}, slug={}, content={}, author={}, status={}, creation_date={}",
            news_data.category,
            news_data.title,
            slug,
            news_data.content,
            news_data.author.as_deref().unwrap_or("Administrator"),
            news_data.status,
            chrono::Utc::now().naive_utc()
        );

        let result = client
            .execute(
                query,
                &[
                    &news_data.category,
                    &news_data.title,
                    &slug,
                    &news_data.content,
                    &news_data.author.as_deref().unwrap_or("Administrator"),
                    &chrono::Utc::now().naive_utc(), // Set current date and time
                    &news_data.status,
                ],
            )
            .await;

        match result {
            Ok(_) => {
                info!("✅ Created news article: {}", news_data.title);
                Ok(())
            }
            Err(e) => {
                tracing::error!("❌ Failed to execute query: {:?}", e);
                Err(e.into())
            }
        }
    }
}

fn generate_slug(title: &str) -> String {
    let re = Regex::new(r"[^\w\s-]").unwrap();
    let slug = title.to_lowercase().replace(' ', "-").replace("--", "-");
    let slug = re.replace_all(&slug, "").to_string();
    slug
}
