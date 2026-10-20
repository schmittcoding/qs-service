use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub template_id: Uuid,
    pub template_name: String,
    pub template_category: String,
    pub subject_line: String,
    pub html_body: String,
    pub status: i16,
    pub version: i32,
    pub created_by: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// -- Request DTOs --

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateEmailTemplateRequest {
    #[validate(length(min = 1, max = 100, message = "Template name must be 1-100 characters"))]
    pub template_name: String,
    #[validate(length(max = 50, message = "Category must be at most 50 characters"))]
    pub template_category: Option<String>,
    #[validate(length(min = 1, max = 200, message = "Subject line must be 1-200 characters"))]
    pub subject_line: String,
    #[validate(length(min = 1, max = 102400, message = "HTML body must be 1-100KB"))]
    pub html_body: String,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateEmailTemplateRequest {
    #[validate(length(min = 1, max = 100, message = "Template name must be 1-100 characters"))]
    pub template_name: Option<String>,
    #[validate(length(max = 50, message = "Category must be at most 50 characters"))]
    pub template_category: Option<String>,
    #[validate(length(min = 1, max = 200, message = "Subject line must be 1-200 characters"))]
    pub subject_line: Option<String>,
    #[validate(length(min = 1, max = 102400, message = "HTML body must be 1-100KB"))]
    pub html_body: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailTemplateFilterParams {
    pub status: Option<i16>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub page: Option<i16>,
    pub limit: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewTemplateRequest {
    pub variables: Option<std::collections::HashMap<String, String>>,
}

// -- Campaign --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCampaign {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub template_id: Uuid,
    pub template_name: String,
    pub subject_line: String,
    pub html_body: Option<String>,
    pub segment_type: i16,
    pub segment_param: Option<i32>,
    pub status: i16,
    pub scheduled_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub total_recipients: i32,
    pub total_sent: i32,
    pub total_failed: i32,
    pub created_by: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateEmailCampaignRequest {
    #[validate(length(min = 1, max = 100, message = "Campaign name must be 1-100 characters"))]
    pub campaign_name: String,
    pub template_id: Uuid,
    pub segment_type: i16,
    pub segment_param: Option<i32>,
    pub scheduled_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignFilterParams {
    pub status: Option<i16>,
    pub page: Option<i16>,
    pub limit: Option<i16>,
}

// -- Recipient --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignRecipient {
    pub campaign_id: Uuid,
    pub user_num: i64,
    pub user_email: String,
    pub user_name: String,
    pub send_status: i16,
    pub attempt_count: i16,
    pub sent_at: Option<DateTime<Local>>,
    pub error_message: Option<String>,
}

// -- Progress --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub campaign_id: Uuid,
    pub status: i16,
    pub total_recipients: i32,
    pub total_sent: i32,
    pub total_failed: i32,
    pub total_pending: i32,
}

// -- WebSocket event broadcast --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignProgressEvent {
    pub campaign_id: Uuid,
    pub status: i16,
    pub total_recipients: i32,
    pub total_sent: i32,
    pub total_failed: i32,
    pub total_pending: i32,
}

// -- Suppression --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSuppressionEntry {
    pub suppression_id: Uuid,
    pub email: String,
    pub reason: String,
    pub diagnostic_info: Option<String>,
    pub source: Option<String>,
    pub suppressed_at: DateTime<Local>,
    pub removed_at: Option<DateTime<Local>>,
    pub removed_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuppressionListParams {
    pub page: Option<i16>,
    pub limit: Option<i16>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RemoveSuppressionRequest {
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,
}
