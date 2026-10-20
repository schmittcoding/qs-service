# Structs to replicate

Contract date: **2026-10-16**. This chapter is the Rust side of the contract. Each definition is copied verbatim from gs-api, including derives and serde attributes. The JSON Schema equivalents in [openapi.json](openapi.json) carry the same names, plus `x-rust-type` and `x-source` extensions on every schema and numeric field.

Serialization rules shared by every struct below:

- **No `rename_all`.** Field names serialize exactly as written. `r#type` serializes as the JSON key `"type"`.
- **`Option<T>` on response structs** serializes as `null`. The key is never omitted, because no response struct here uses `skip_serializing_if`.
- **`Option<T>` on request structs** accepts the key missing or `null`.
- **Unknown fields are ignored.** No struct uses `deny_unknown_fields`.
- **Validation.** No struct uses `validator`. The only checks are the ones in the handlers, listed per endpoint.
- **Types:**
  - `Uuid` is a lowercase hyphenated string.
  - `DateTime<Local>` is RFC 3339 with the API server's local offset, e.g. `2026-10-16T14:03:22+08:00`.
  - `f64` is a JSON number. The DB column is `MONEY`, so do not assume an integer.

## Endpoint → struct map

| Endpoint | Request | Success response |
|---|---|---|
| `GET /api/v1/payments/get-recharge-info` | — | `Vec<RechargeInfo>` |
| `GET /api/v1/payments/get-recharge-info/{id}` | path `Uuid` | `RechargeInfo` |
| `POST /api/v1/payments/transaction/{id}` | path `Uuid` + `Option<Json<TransactionPayload>>` | `TransactionAccepted` \| `PayPalApproval` \| `PayMongoCheckoutCreated` (`json!`) |
| `GET /api/v1/payments/get-gateway-info` | — | `Vec<GatewayInfo>` |
| `GET /api/v1/payments/get-recharge-logs` | `Query<PaginationParams>` | `TransactionPage` (`json!`, `data: Vec<WebTransactionResponse>`) |
| `POST /api/v1/payments/webhook/paypal` | `Json<serde_json::Value>` (see `PayPalWebhookEvent` schema) | `{message}` |
| `POST /api/v1/payments/webhook/paymongo` | raw bytes → `PayMongoWebhookPayload` | text |
| `GET /admin/v1/recharge` | `TransactionFilterParams` via `serde_qs` | `AdminTransactionPage` (`json!`) |
| `GET /admin/v1/recharge/recharge-logs` | `Query<PaginationParams>` | `TransactionPage` (`json!`) |
| `PUT /admin/v1/recharge/update-transaction/{id}` | path `Uuid` + `Query<UpdateStatusParams>` | `{message}` |
| `PUT /admin/v1/recharge/update-recharge-info/{id}` | path `Uuid` + `Json<AddRechargeInfo>` | `RechargeInfo` |
| `PUT /admin/v1/recharge/update-gateway-info/{id}` | path `Uuid` + `Json<AddGatewayInfo>` | `GatewayInfo` |
| `GET /admin/v1/recharge/get-gateway-info` | — | `Vec<GatewayInfo>` |
| `POST /admin/v1/recharge/add-gateway-info` | `Json<AddGatewayInfo>` | `{message}` |
| `GET /admin/v1/recharge/get-recharge-info` | `Query<PaginationParams>` | `AdminRechargeInfoPage` (`json!`, `data: Vec<RechargeInfo>`) |
| `POST /admin/v1/recharge/add-recharge-info` | `Json<AddRechargeInfo>` | `{message}` |

Envelopes marked `json!` have no Rust struct; the handler builds them inline. Their exact shapes are under [Response envelopes](#response-envelopes-json-macros) below.

## Integer codes

There is no Rust enum for any of these. They are raw `i16` (`SMALLINT`) literals.

| `TransactionType` (`RechargeInfo.type`, `WebTransactionResponse.transaction_type`, query `category` / `method[]`) | Meaning | Body on `POST /transaction/{id}` | DB insert | Settled by |
|---|---|---|---|---|
| `1` | GCash | `{"type":"gcash", reference_number, proof_image?}` | `transact_gcash` | Administrator `PUT /update-transaction/{id}?status=1\|2` |
| `2` | PayPal | none | `transact` | PayPal webhook `PAYMENT.CAPTURE.COMPLETED` / `DENIED` / `DECLINED` |
| `3` | Wise | `{"type":"wise", reference_number}` | `transact_v2` | Administrator |
| `4` | PayMongo | none | `transact` | PayMongo webhook `payment.paid` / `payment.failed` |

| `TransactionStatus` (`transaction_status`, query `status` / `status[]`) | Meaning |
|---|---|
| `0` | Pending. This is the DB default on insert, and it is the only state that can change. |
| `1` | Approved. Trigger `trg_AddWebPoints` adds `TransactionAmount` (the points) to `WebUser.WebPoints` on the 0→1 change. |
| `2` | Declined or cancelled. |

| `RecordStatus` (`RechargeInfo.status`, `GatewayInfo.status`) | Meaning |
|---|---|
| `1` | Active, shown to players. |
| anything else | Hidden from players. The code gives no other value a meaning. |

`GatewayInfo.provider` has no mapping anywhere in the code or SQL. Treat it as an opaque `i16`.

## Payment models: `ran-models/src/payments/mod.rs`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RechargeInfo {
    pub id: Uuid,
    pub r#type: i16,
    pub amount: f64,
    pub discount: Option<f64>,
    pub points: f64,
    pub status: i16,
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

// Internal row model. Never serialized to clients directly (proof_image is raw bytes here).
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

// What clients receive: same fields, proof_image base64-encoded.
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

// Internal insert model built from the session + RechargeInfo. Not part of any request/response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Transaction {
    pub user_num: i64,
    pub user_name: String,
    pub r#type: i16,
    pub amount: f64,
    pub points: f64,
}

// Internal; no serde derives.
pub enum TransactionInfo {
    GCash(GCashTransaction),
    Wise(WiseTransaction),
}
```

`WebTransactionResponse::from(&WebTransaction)` and `WebTransactionResponse::from_transactions(&WebTransaction)` are identical. Both copy every field and set `proof_image = BASE64_STANDARD.encode(bytes)`, with no `data:` prefix.

## Manual-payment payloads: `ran-models/src/payments/{gcash,wise}/mod.rs`

```rust
#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct GCashTransaction {
    pub reference_number: String,
    pub proof_image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct WiseTransaction {
    pub reference_number: String,
}
```

## Request body union: `ran-access-point/src/routes/payments/mod.rs#L38`

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")] // allows tagged deserialization
enum TransactionPayload {
    GCash(GCashTransaction),
    Wise(WiseTransaction)
}
```

Wire form is internally tagged, so the variant fields sit beside `type`:

```json
{"type": "gcash", "reference_number": "1023456789012", "proof_image": "<plain base64 or null>"}
{"type": "wise", "reference_number": "P123456789"}
```

## Query structs

```rust
// ran-models/src/lib.rs#L23 — actix web::Query (serde_urlencoded: single values)
#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<i16>,
    pub category: Option<i16>,
    pub search: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ran-models/src/admin/mod.rs#L127 — serde_qs::Config::new(5, false) (bracket arrays: method[]=1&method[]=2)
#[derive(Deserialize)]
pub struct TransactionFilterParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub search: Option<String>,
    #[serde(default)]
    pub method: Option<Vec<i16>>,
    #[serde(default)]
    pub status: Option<Vec<i16>>,
}

// ran-access-point/src/routes/admin/recharge/mod.rs#L34 — PUT /update-transaction/{id}?status=N
#[derive(Deserialize)]
struct UpdateStatusParams {
    status: i16,
}
```

## Session and admin identity

```rust
// ran-models/src/users/mod.rs#L63 — stored in the session under key "user_data".
// The payment routes use only user_num, user_name, user_email (PayMongo billing) and is_verified.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, utoipa::ToSchema)]
pub struct ProfileData {
    pub user_name: String,
    pub user_num: i64,
    pub user_email: String,
    pub user_available: u8,
    pub user_type: i32,
    pub web_points: f64,
    pub mileage_points: f64,
    pub created_at: DateTime<Local>,
    pub last_updated: DateTime<Local>,
    #[serde(default)]
    pub is_verified: bool,
}

// ran-access-point/src/services/jwt.rs#L12 — admin bearer JWT claims (HS256, secret = SESSION_SECRET).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}
```

## PayMongo webhook: `ran-access-point/src/routes/payments/paymongo_integration.rs#L205`

```rust
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
```

`PayMongoWebhookAttributesData.attributes` stays a `Value` until the handler re-parses it into `PaymentDataAttributes`, and it does that only for `payment.paid` and `payment.failed`. `metadata` is a plain `Value`, not an `Option`, so the key has to be present; the `skip_serializing_if` attribute has no effect. The PayPal webhook has no struct: it reads `event_type`, `resource.id`, `resource.invoice_id`, `resource.purchase_units[0].invoice_id` and `resource.custom` from a `serde_json::Value`.

## Response envelopes (`json!` macros)

These have no Rust type. Recreate them as structs on your side if needed. They are named here as they are in `openapi.json`.

```text
TransactionAccepted      {message:"Transaction processed", data:GCashTransaction|WiseTransaction, recharge_info:RechargeInfo}
PayPalApproval           {approval_url:String}
PayMongoCheckoutCreated  {message:"PayMongo Checkout Session created", checkout_session_id:String, status:String,
                          approval_url:String, recharge_info:RechargeInfo}
TransactionPage          {page:usize, limit:usize, total_pages:usize, total_items:usize, data:Vec<WebTransactionResponse>}
AdminTransactionPage     {data:Vec<WebTransactionResponse>, page:i16, limit:i16, total_items:i32}
AdminRechargeInfoPage    {data:Vec<RechargeInfo>, total_items:usize}
AdminMessage / WebhookMessage   {message:String}
PaymentsMessageError     {message:String, error_code?:"EMAIL_NOT_VERIFIED", recharge_info?:RechargeInfo}
PaymentsErrorEnvelope    {error:String, details?:String, recharge_info?:RechargeInfo}
RechargeAdminError       {error:String}
```

`?` means the key appears only on some responses; it is never `null`.

## Database column mapping

`WebTransaction::from_row` (`ran-models/src/payments/mod.rs#L30`) maps these columns of table `RanWeb.dbo.WebTransaction`:

| Field | Column (SQL type) | Notes |
|---|---|---|
| `transaction_id` | `TransactionID` (UNIQUEIDENTIFIER) | Generated in Rust with `Uuid::new_v4()` on insert |
| `user_num` | `UserNum` (BIGINT) | |
| `user_name` | `UserName` (NVARCHAR(20)) | |
| `transaction_type` | `TransactionType` (SMALLINT) | `.unwrap()`, so a NULL panics |
| `transaction_time` | `TransactionTime` (DATETIME, default `GETDATE()`) | Read as naive, then interpreted in the server's local zone |
| `transaction_price` | `TransactionPrice` (MONEY) | = `RechargeInfo.amount` |
| `transaction_status` | `TransactionStatus` (SMALLINT, default 0) | |
| `transaction_amount` | `TransactionAmount` (MONEY) | = `RechargeInfo.points`, **not money** |
| `updated_at` | `UpdatedAt` (DATETIME) | |
| `transaction_cost` | `TransactionCost` (MONEY) | = `RechargeInfo.amount` |
| `proof_image` | `ProofImage` (VARBINARY(MAX)) | Decoded from the request's base64 on insert |
| `reference_code` | `RefferenceCode` (NVARCHAR(50)) | Column name is misspelled; keep it if you mirror the table |
| `admin_remarks` | `AdminRemarks` | Never written by these endpoints |

`parse_recharge_info` (`ran-db-core/src/webmanager/transaction/mod.rs#L12`) maps `RechargeID`, `RechargeType`, `RechargeAmount`, `RechargeDiscount`, `RechargePoints` and `RechargeStatus`.

`parse_gateway_info` (`#L23`) maps `GatewayID`, `GatewayProvider`, `GatewayName`, `GatewayNumber` (NVARCHAR) and `GatewayImage` (VARBINARY, returned as base64), plus `GatewayStatus`. A row with a NULL image is skipped.

The table script `db-scripts/webdb/Tables/CreateTransaction.sql` is older than the live schema: it lacks `TransactionPrice` and has a `WebPoints` column. The column list above is the one the code reads and writes.
