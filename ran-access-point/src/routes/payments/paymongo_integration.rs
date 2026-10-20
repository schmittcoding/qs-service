use std::{collections::HashMap, env, error::Error};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;
use serde_json::{Value};
use tracing;

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Debug)]
struct PaymentIntentAttributes {
    amount: u64,
    currency: String,
    payment_method_allowed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_descriptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_url: Option<String>
}

#[derive(Serialize, Debug)]
struct PaymentIntentRequest {
    data: PaymentIntentData
}

#[derive(Serialize, Debug)]
struct PaymentIntentData {
    #[serde(rename = "type")]
    object_type: String,
    attributes: PaymentIntentAttributes,
}

#[derive(Deserialize, Debug)]
pub struct PaymentIntentResponseAttributes {
    // id: String,
    pub status: String,
    pub amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<NextAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livemode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>
}

#[derive(Deserialize, Debug)]
pub struct NextAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub url: String
}

#[derive(Deserialize, Debug)]
pub struct PayMongoResponseData {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub attributes: PaymentIntentResponseAttributes
}

#[derive(Deserialize, Debug)]
pub struct PayMongoResponse {
    pub data: PayMongoResponseData
}

#[derive(Deserialize, Debug)]
struct PayMongoErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PayMongoErrorDetail {
    code: String,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<PayMongoErrorSource>,
}

#[derive(Deserialize, Debug)]
struct PayMongoErrorResponse {
    errors: Vec<PayMongoErrorDetail>,
}

#[derive(Serialize, Debug)]
struct CheckoutLineItem {
    currency: String,
    amount: u64, // In centavos
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckoutBilling {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<CheckoutAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckoutAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    line1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
}

#[derive(Serialize, Debug)]
struct CheckoutSessionAttributes {
    billing: CheckoutBilling,
    send_email_receipt: bool,
    show_description: bool,
    show_line_items: bool,
    cancel_url: String, // Where to redirect if user cancels
    success_url: String, // Where to redirect after successful payment
    line_items: Vec<CheckoutLineItem>,
    payment_method_types: Vec<String>, // e.g., ["card", "gcash"]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_descriptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_number: Option<String>, // Optional reference number
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Value>, // Crucial for linking back to your system
}

#[derive(Serialize, Debug)]
struct CheckoutSessionRequest {
    data: CheckoutSessionData,
}

#[derive(Serialize, Debug)]
struct CheckoutSessionData {
    #[serde(rename = "type")]
    object_type: String, // Must be "checkout_session"
    attributes: CheckoutSessionAttributes,
}

#[derive(Deserialize, Debug)]
pub struct CheckoutSessionResponseAttributes {
    pub checkout_url: String, // The URL to redirect the user to
    pub status: String, // e.g., "open", "paid", "expired"
    pub payment_method_types: Vec<String>,
    pub livemode: bool,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<CheckoutBilling>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement_descriptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct PayMongoCheckoutSessionResponseData {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String, // "checkout_session"
    pub attributes: CheckoutSessionResponseAttributes,
}

#[derive(Deserialize, Debug)]
pub struct PayMongoCheckoutSessionResponse {
    pub data: PayMongoCheckoutSessionResponseData,
}

#[derive(Debug, Deserialize)]
pub struct PaymentDataAttributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Value
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayMongoWebhookAttributesData {
    pub id: String,
     #[serde(rename = "type")]
    pub object_type: String, // e.g. "payment"
    pub attributes: Value
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayMongoWebhookAttributes {
    #[serde(rename = "type")]
    pub event_type: String, // e.g., "payment.paid", "payment.failed"
    pub livemode: bool,
    pub data: PayMongoWebhookAttributesData,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayMongoWebhookData {
    pub id: String,
     #[serde(rename = "type")]
    pub object_type: String, // e.g. "event"
    pub attributes: PayMongoWebhookAttributes
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayMongoWebhookPayload {
    pub data: PayMongoWebhookData
}

// Struct for Checkout Session resource attributes in a webhook
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckoutSessionWebhookAttributes {
    pub status: String, // "paid", "expired", "open" etc.
    pub payment_method_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<CheckoutBilling>, // Re-use billing from Checkout API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>, // Crucial for linking back to your system's `recharge_id`
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub payments: Option<Vec<PaymentAttributes>>, // Nested payments array for checkout.paid
    // // ... other checkout session specific fields you might need
}

pub async fn create_paymongo_checkout_session(
    transaction_id: Uuid,
    amount_in_centavos: u64,
    currency: &str,
    item_name: &str,
    customer_name: &str,
    customer_email: &str,
    payment_methods: Vec<String>,
) -> Result<PayMongoCheckoutSessionResponse, Box<dyn Error>> {
    let api_base_url = "https://api.paymongo.com/v1/checkout_sessions";
    let secret_key: String = env::var("PAYMONGO_SECRET_KEY").expect("PAYMONGO_SECRET_KEY must be set");

    let mut metadata_map = serde_json::Map::new();
    metadata_map.insert("transaction_id".to_string(), serde_json::Value::String(transaction_id.to_string()));

    let metadata_value = Some(Value::Object(metadata_map));

    let request_body = CheckoutSessionRequest {
        data: CheckoutSessionData {
            object_type: "checkout_session".to_string(),
            attributes: CheckoutSessionAttributes {
                billing: CheckoutBilling {
                    name: Some(customer_name.to_string()),
                    email: Some(customer_email.to_string()),
                    phone: None,
                    address: None,
                },
                send_email_receipt: true,
                show_description: true,
                show_line_items: true,
                cancel_url: env::var("PAYMONGO_CANCEL_URL").expect("PAYMONGO_CANCEL_URL must be set"),
                success_url: env::var("PAYMONGO_SUCCESS_URL").expect("PAYMONGO_SUCCESS_URL must be set"),
                line_items: vec![
                    CheckoutLineItem {
                        currency: currency.to_string(),
                        amount: amount_in_centavos,
                        name: item_name.to_string(),
                        quantity: Some(1),
                        description: None
                    }
                ],
                payment_method_types: payment_methods,
                description: Some(format!("Payment for {}", item_name)),
                statement_descriptor: None,
                reference_number: Some(transaction_id.clone().to_string()),
                metadata: metadata_value,
            }
        }
    };

    let auth_string = format!("{}:", secret_key);
    let auth_header_value = format!("Basic {}", base64::encode(auth_string.as_bytes()));

    let client = reqwest::Client::new();
    let response = client
        .post(api_base_url)
        .header("Authorization", auth_header_value)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();

    tracing::info!("PayMongo API Response Status (Create Checkout): {}", status);

    if status.is_success() {
        let paymongo_response: PayMongoCheckoutSessionResponse = response.json().await?;
        Ok(paymongo_response)
    } else {
        let error_response: PayMongoErrorResponse = response.json().await?;

        let mut error_details = String::new();
        for error in error_response.errors {
            error_details.push_str(&format!("Code: {}, Detail: {}. ", error.code, error.detail));
            if let Some(source) = error.source {
                if let Some(pointer) = source.pointer {
                    error_details.push_str(&format!("Pointer: {}. ", pointer));
                }
                if let Some(parameter) = source.parameter {
                    error_details.push_str(&format!("Parameter: {}. ", parameter));
                }
            }
        }
        Err(format!("PayMongo API Error (Status {} - Create Checkout): {}", status, error_details).into())
    }

}

pub async fn create_paymongo_payment_intent (
    transaction_id: Uuid,
    amount: u64,
    currency: &str,
    payment_methods: Vec<String>,
    description: Option<String>,
) -> Result<PayMongoResponse, Box<dyn Error>> {
    let api_base_url = env::var("PAYMONGO_BASE_URL").expect("PAYMONGO_BASE_URL must be set");
    let secret_key = env::var("PAYMONGO_SECRET_KEY").expect("PAYMONGO_SECRET_KEY must be set");
    let auth_header = format!("Basic {}", base64::encode(format!("{}:", secret_key)));

    let mut metadata_map = serde_json::Map::new();
    metadata_map.insert("transaction_id".to_string(), serde_json::Value::String(transaction_id.to_string()));

    let request_body = PaymentIntentRequest {
        data: PaymentIntentData {
            object_type: "payment_intent".to_string(),
            attributes: PaymentIntentAttributes {
                amount,
                currency: currency.to_string(),
                payment_method_allowed: payment_methods,
                description,
                statement_descriptor: Some("Recharge".to_string()),
                metadata: Some(serde_json::Value::Object(metadata_map)),
                return_url: Some("https://ranonlinegs.com/recharge?type=success&method=paymongo".to_string())
            }
        }
    };

    tracing::info!("Request body {}", serde_json::json!(request_body));

    let client = reqwest::Client::new();
    let response = client
        .post(api_base_url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let paymongo_response: PayMongoResponse = response.json().await?;

        tracing::info!("Success 2");
        Ok(paymongo_response)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(format!(
            "PayMongo API Error: Status - {}, Response - {}",
            status, error_text
        )
        .into())
    }
}

pub async fn verify_paymongo_signature(
    payload_body: &str,
    paymongo_signature_header: &str,
    webhook_secret_key: &str
) -> bool {
    let parts: HashMap<_, _> = paymongo_signature_header
        .split(",")
        .filter_map(|part| {
            let mut kv = part.splitn(2, "=");
            let key = kv.next()?;
            let value = kv.next()?;
            Some((key, value))
        })
        .collect();

    let timestamp_str = match parts.get("t") {
        Some(t) => t,
        None => {
            tracing::warn!("PayMongo-Signature header missing timestamp (t) part.");
            return false;
        }
    };

    let signature_from_header = match parts.get("te") {
        Some(te) => te,
        None => {
            tracing::warn!("PayMongo-Signature header missing signature (te) part.");
            return false;
        }
    };

    let signed_payload = format!("{}{}", timestamp_str, payload_body);

    let mut mac = match HmacSha256::new_from_slice(webhook_secret_key.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to create HMAC-SHA256 with secret: {}", e);
            return false
        }
    };

    mac.update(signed_payload.as_bytes());

    let result = mac.finalize();
    let expected_signature_bytes = result.into_bytes();

    let expected_signature_hex = hex::encode(expected_signature_bytes);

    let is_valid = expected_signature_hex == *signature_from_header;
    if !is_valid {
        tracing::warn!(
            "Webhook signature mismatch. Expected: {}, Received: {}",
            expected_signature_hex,
            signature_from_header
        );
    }
    is_valid
}
