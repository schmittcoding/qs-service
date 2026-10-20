use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime, TimeZone};
use ran_models::{
    email::{CreateEmailTemplateRequest, EmailTemplate, UpdateEmailTemplateRequest},
    BaseDataCountResult,
};
use uuid::Uuid;

use crate::webmanager::WebDB;

fn row_to_email_template(row: &tiberius::Row) -> EmailTemplate {
    let created_at_naive: NaiveDateTime = row
        .try_get::<NaiveDateTime, _>("CreatedAt")
        .ok()
        .flatten()
        .unwrap_or_default();
    let updated_at_naive: NaiveDateTime = row
        .try_get::<NaiveDateTime, _>("UpdatedAt")
        .ok()
        .flatten()
        .unwrap_or_default();

    EmailTemplate {
        template_id: row
            .try_get::<Uuid, _>("TemplateID")
            .ok()
            .flatten()
            .unwrap_or_default(),
        template_name: row
            .try_get::<&str, _>("TemplateName")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        template_category: row
            .try_get::<&str, _>("TemplateCategory")
            .ok()
            .flatten()
            .unwrap_or("general")
            .to_string(),
        subject_line: row
            .try_get::<&str, _>("SubjectLine")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        html_body: row
            .try_get::<&str, _>("HtmlBody")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        status: row
            .try_get::<u8, _>("Status")
            .ok()
            .flatten()
            .unwrap_or(0) as i16,
        version: row
            .try_get::<i32, _>("Version")
            .ok()
            .flatten()
            .unwrap_or(1),
        created_by: row
            .try_get::<&str, _>("CreatedBy")
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        created_at: Local
            .from_local_datetime(&created_at_naive)
            .single()
            .unwrap_or_else(|| Local::now()),
        updated_at: Local
            .from_local_datetime(&updated_at_naive)
            .single()
            .unwrap_or_else(|| Local::now()),
    }
}

#[async_trait::async_trait]
pub trait WebEmailTemplateTrait {
    async fn create_email_template(
        &self,
        req: CreateEmailTemplateRequest,
        admin_username: &str,
    ) -> Result<EmailTemplate>;

    async fn update_email_template(
        &self,
        template_id: Uuid,
        req: UpdateEmailTemplateRequest,
    ) -> Result<EmailTemplate>;

    async fn get_email_templates(
        &self,
        status: Option<i16>,
        category: Option<String>,
        search: Option<String>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult<EmailTemplate>>;

    async fn get_email_template_by_id(&self, template_id: Uuid) -> Result<Option<EmailTemplate>>;

    async fn delete_email_template(&self, template_id: Uuid) -> Result<EmailTemplate>;
}

#[async_trait::async_trait]
impl WebEmailTemplateTrait for WebDB {
    async fn create_email_template(
        &self,
        req: CreateEmailTemplateRequest,
        admin_username: &str,
    ) -> Result<EmailTemplate> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let category = req.template_category.unwrap_or_else(|| "general".to_string());
        let status: u8 = req.status.unwrap_or(0) as u8;

        let sp_query = "EXEC sp_CreateEmailTemplate
            @TemplateName = @P1,
            @TemplateCategory = @P2,
            @SubjectLine = @P3,
            @HtmlBody = @P4,
            @Status = @P5,
            @CreatedBy = @P6";

        let row = client
            .query(
                sp_query,
                &[
                    &req.template_name.as_str(),
                    &category.as_str(),
                    &req.subject_line.as_str(),
                    &req.html_body.as_str(),
                    &status,
                    &admin_username,
                ],
            )
            .await
            .context("Failed to execute sp_CreateEmailTemplate")?
            .into_row()
            .await
            .context("Failed to fetch result row")?
            .context("No row returned from sp_CreateEmailTemplate")?;

        Ok(row_to_email_template(&row))
    }

    async fn update_email_template(
        &self,
        template_id: Uuid,
        req: UpdateEmailTemplateRequest,
    ) -> Result<EmailTemplate> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let status: Option<u8> = req.status.map(|s| s as u8);

        let sp_query = "EXEC sp_UpdateEmailTemplate
            @TemplateID = @P1,
            @TemplateName = @P2,
            @TemplateCategory = @P3,
            @SubjectLine = @P4,
            @HtmlBody = @P5,
            @Status = @P6";

        let row = client
            .query(
                sp_query,
                &[
                    &template_id,
                    &req.template_name,
                    &req.template_category,
                    &req.subject_line,
                    &req.html_body,
                    &status,
                ],
            )
            .await
            .context("Failed to execute sp_UpdateEmailTemplate")?
            .into_row()
            .await
            .context("Failed to fetch result row")?
            .context("No row returned from sp_UpdateEmailTemplate")?;

        if let Ok(Some(err_code)) = row.try_get::<&str, _>("ErrorCode") {
            if err_code == "NOT_FOUND" {
                anyhow::bail!("NOT_FOUND");
            }
        }

        Ok(row_to_email_template(&row))
    }

    async fn get_email_templates(
        &self,
        status: Option<i16>,
        category: Option<String>,
        search: Option<String>,
        page: i16,
        page_size: i16,
    ) -> Result<BaseDataCountResult<EmailTemplate>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let status_param: Option<u8> = status.map(|s| s as u8);

        let sp_query = "EXEC sp_GetEmailTemplates
            @Status = @P1,
            @Category = @P2,
            @Search = @P3,
            @Page = @P4,
            @PageSize = @P5";

        let results = client
            .query(
                sp_query,
                &[&status_param, &category, &search, &page, &page_size],
            )
            .await
            .context("Failed to execute sp_GetEmailTemplates")?
            .into_results()
            .await
            .context("Failed to fetch results")?;

        let mut result_sets = results.into_iter();

        let total_count = result_sets
            .next()
            .unwrap_or_default()
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let data: Vec<EmailTemplate> = result_sets
            .next()
            .unwrap_or_default()
            .iter()
            .map(|row| row_to_email_template(row))
            .collect();

        Ok(BaseDataCountResult {
            data,
            total_records: total_count,
        })
    }

    async fn get_email_template_by_id(&self, template_id: Uuid) -> Result<Option<EmailTemplate>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_GetEmailTemplateById @TemplateID = @P1";

        let row = client
            .query(sp_query, &[&template_id])
            .await
            .context("Failed to execute sp_GetEmailTemplateById")?
            .into_row()
            .await
            .context("Failed to fetch result row")?;

        Ok(row.map(|r| row_to_email_template(&r)))
    }

    async fn delete_email_template(&self, template_id: Uuid) -> Result<EmailTemplate> {
        let status_inactive: u8 = 2;
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let sp_query = "EXEC sp_UpdateEmailTemplate
            @TemplateID = @P1,
            @Status = @P2";

        let none_str: Option<&str> = None;

        let row = client
            .query(
                sp_query,
                &[
                    &template_id,
                    &none_str,
                    &none_str,
                    &none_str,
                    &none_str,
                    &Some(status_inactive),
                ],
            )
            .await
            .context("Failed to execute sp_UpdateEmailTemplate for soft delete")?
            .into_row()
            .await
            .context("Failed to fetch result row")?
            .context("Template not found")?;

        if let Ok(Some(err_code)) = row.try_get::<&str, _>("ErrorCode") {
            if err_code == "NOT_FOUND" {
                anyhow::bail!("NOT_FOUND");
            }
        }

        Ok(row_to_email_template(&row))
    }
}
