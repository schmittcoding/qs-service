use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebReportTransaction {
    pub transaction_id: Uuid,
    pub user_num: i64,
    pub user_name: String,
    pub transaction_type: i16,
    pub transaction_time: DateTime<Local>,
    pub transaction_price: f64,
    pub transaction_status: i16,
    pub transaction_amount: f64,
    pub updated_at: Option<DateTime<Local>>, // Nullable field
    pub reference_code: Option<String>,
    pub admin_remarks: Option<String>,
}

impl WebReportTransaction {
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


            reference_code: row.get::<&str, _>("RefferenceCode").map(|s| s.to_string()),

            admin_remarks: row.get::<&str, _>("AdminRemarks").map(|s| s.to_string()),
        }
    }
}