pub mod gcash;
pub mod wise;
use base64::prelude::{Engine, BASE64_STANDARD as BASE64};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use gcash::GCashTransaction;
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;
use wise::WiseTransaction;

pub mod paypal;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WebTransaction {
    pub transaction_id: Uuid,
    pub user_num: i64,
    pub user_name: String,
    pub transaction_type: i16,
    pub transaction_time: DateTime<Local>,
    pub transaction_price: f64,
    pub transaction_status: i16,
    pub transaction_amount: f64,
    pub updated_at: Option<DateTime<Local>>, // Nullable field
    pub transaction_cost: f64,
    pub proof_image: Option<Vec<u8>>, // Assuming image stored as binary
    pub reference_code: Option<String>,
    pub admin_remarks: Option<String>,
}

impl WebTransaction {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            transaction_id: row
                .get::<Uuid, _>("TransactionID")
                .unwrap_or_else(Uuid::nil), // Default to nil UUID if conversion fails

            user_num: row.get::<i64, _>("UserNum").unwrap_or_default(),

            user_name: row
                .get::<&str, _>("UserName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            transaction_type: row.get::<i16, _>("TransactionType").unwrap(),

            transaction_time: {
                let naive = row
                    .get::<NaiveDateTime, _>("TransactionTime")
                    .unwrap_or_else(|| {
                        NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                            .expect("Failed to parse default date")
                    });
                to_local(naive)
            },

            transaction_price: row.get::<f64, _>("TransactionPrice").unwrap_or_default(),
            transaction_status: row.get::<i16, _>("TransactionStatus").unwrap_or_default(),
            transaction_amount: row.get::<f64, _>("TransactionAmount").unwrap_or_default(),
            updated_at: row
                .get::<NaiveDateTime, _>("UpdatedAt")
                .map(|naive| to_local(naive)),
            transaction_cost: row.get::<f64, _>("TransactionCost").unwrap_or_default(),

            proof_image: row
                .get::<&[u8], _>("ProofImage")
                .map(|bytes| bytes.to_vec()),

            reference_code: row.get::<&str, _>("RefferenceCode").map(|s| s.to_string()),

            admin_remarks: row.get::<&str, _>("AdminRemarks").map(|s| s.to_string()),
        }
    }
}

// Struct for API response that includes base64-encoded image
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct WebTransactionResponse {
    pub transaction_id: Uuid,
    pub user_num: i64,
    pub user_name: String,
    pub transaction_type: i16,
    pub transaction_time: DateTime<Local>,
    pub transaction_price: f64,
    pub transaction_status: i16,
    pub transaction_amount: f64,
    pub updated_at: Option<DateTime<Local>>,
    pub transaction_cost: f64,
    pub proof_image: Option<String>, // Base64-encoded image
    pub reference_code: Option<String>,
    pub admin_remarks: Option<String>,
}

impl WebTransactionResponse {
    pub fn from_transactions(tx: &WebTransaction) -> Self {
        // Convert binary proof_image to base64 string if present
        let proof_image = tx.proof_image.as_ref().map(|bytes| {
            let base64_string = BASE64.encode(bytes);
            format!("{}", base64_string)
        });

        Self {
            transaction_id: tx.transaction_id,
            user_num: tx.user_num,
            user_name: tx.user_name.clone(),
            transaction_type: tx.transaction_type,
            transaction_time: tx.transaction_time,
            transaction_price: tx.transaction_price,
            transaction_status: tx.transaction_status,
            transaction_amount: tx.transaction_amount,
            updated_at: tx.updated_at,
            transaction_cost: tx.transaction_cost,
            proof_image,
            reference_code: tx.reference_code.clone(),
            admin_remarks: tx.admin_remarks.clone(),
        }
    }
}

impl From<&WebTransaction> for WebTransactionResponse {
    fn from(transaction: &WebTransaction) -> Self {
        // Convert binary proof_image to base64 string if present
        let proof_image = transaction.proof_image.as_ref().map(|bytes| {
            let base64_string = BASE64.encode(bytes);
            format!("{}", base64_string)
        });

        Self {
            transaction_id: transaction.transaction_id,
            user_num: transaction.user_num,
            user_name: transaction.user_name.clone(),
            transaction_type: transaction.transaction_type,
            transaction_time: transaction.transaction_time,
            transaction_price: transaction.transaction_price,
            transaction_status: transaction.transaction_status,
            transaction_amount: transaction.transaction_amount,
            updated_at: transaction.updated_at,
            transaction_cost: transaction.transaction_cost,
            proof_image,
            reference_code: transaction.reference_code.clone(),
            admin_remarks: transaction.admin_remarks.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Transaction {
    pub user_num: i64,
    pub user_name: String,
    pub r#type: i16,
    pub amount: f64,
    pub points: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RechargeInfo {
    pub id: Uuid,
    pub r#type: i16,
    pub amount: f64,
    pub discount: Option<f64>,
    pub points: f64,
    pub status: i16,
}

impl RechargeInfo {
    pub fn new(
        id: Uuid,
        r#type: i16,
        amount: f64,
        discount: Option<f64>,
        points: f64,
        status: i16,
    ) -> Self {
        Self {
            id,
            r#type,
            amount,
            discount,
            points,
            status,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Copy, utoipa::ToSchema)]
pub struct AddRechargeInfo {
    pub r#type: i16,
    pub amount: f64,
    pub discount: Option<f64>,
    pub points: f64,
    pub status: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AddGatewayInfo {
    pub provider: i16,
    pub name: String,
    pub number: String,
    pub image: String,
    pub status: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GatewayInfo {
    pub id: Uuid,
    pub provider: i16,
    pub name: String,
    pub number: String,
    pub image: String,
    pub status: i16,
}

pub enum TransactionInfo {
    GCash(GCashTransaction),
    Wise(WiseTransaction),
}

impl TransactionInfo {
    pub fn reference_number(&self) -> &str {
        match self {
            TransactionInfo::GCash(g) => &g.reference_number,
            TransactionInfo::Wise(w) => &w.reference_number,
        }
    }

    pub fn proof_image(&self) -> Option<&str> {
        match self {
            TransactionInfo::GCash(gc) => gc.proof_image.as_deref(),
            TransactionInfo::Wise(_) => None,
        }
    }
}
