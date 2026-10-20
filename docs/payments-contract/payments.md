# Player payments

This contract documents the current Rust implementation as of **2026-10-16**. The API base for this chapter is **`{api_origin}/api/v1/payments`** (`API_VERSION=1`). The machine-readable fragment is [parts/payments.openapi.json](parts/payments.openapi.json). It defines the `UserSessionCookie` security scheme and the shared payment schemas (`RechargeInfo`, `GatewayInfo`, `WebTransactionResponse`, `TransactionPayload`, ...), which the admin and webhook fragments reuse. Verbatim Rust structs are in [structs.md](structs.md). Examples use invented UUIDs, names and references.

Implementation sources:
- [Route handlers](../../ran-access-point/src/routes/payments/mod.rs#L49)
- [PayMongo checkout client](../../ran-access-point/src/routes/payments/paymongo_integration.rs#L256)
- [Payment models](../../ran-models/src/payments/mod.rs#L13)
- [DB trait](../../ran-db-core/src/webmanager/transaction/mod.rs#L45)
- [Session helper](../../ran-access-point/src/middleware/session/mod.rs#L7)
- [Points trigger](../../db-scripts/webdb/Triggers/trg_AddWebPoints.sql)

## Endpoint inventory

Every success status is **200**; none of these routes return 201.

| Method and path | Caller / required proof | Body | Success |
| --- | --- | --- | --- |
| `GET /get-recharge-info` | Public | None | `RechargeInfo[]` with `status = 1` |
| `GET /get-recharge-info/{id}` | Public | None | `RechargeInfo` (any status) |
| `POST /transaction/{id}` | Session cookie with `is_verified = true` | `TransactionPayload` for GCash or Wise, nothing for PayPal or PayMongo | `TransactionAccepted`, `PayPalApproval` or `PayMongoCheckoutCreated` |
| `GET /get-gateway-info` | Public | None | `GatewayInfo[]` with `status = 1` |
| `GET /get-recharge-logs` | Session cookie | None; query `page`, `limit`, `status`, `category` | `TransactionPage` of the caller's own transactions |

The PayPal and PayMongo webhooks share this scope (`/webhook/paypal`, `/webhook/paymongo`). They are documented in [provider-webhooks.md](provider-webhooks.md).

## Purchase flow

```text
Browser                          gs-api                                   Provider / Admin
  GET /get-recharge-info  ───▶  RechargeInfo[] (status 1)
  GET /get-gateway-info   ───▶  GatewayInfo[] (manual methods: where to pay)
  POST /transaction/{id}  ───▶  INSERT WebTransaction (status 0, Pending)
       type 1 GCash / 3 Wise     ◀── {message, data, recharge_info}        admin PUT /admin/v1/recharge/update-transaction/{tx}?status=1|2
       type 2 PayPal             ◀── {approval_url}  ──redirect──▶ PayPal ──webhook──▶ /webhook/paypal ─▶ status 1|2
       type 4 PayMongo           ◀── {..., approval_url} ─redirect─▶ PayMongo ─webhook─▶ /webhook/paymongo ─▶ status 1|2
                                 status 0→1 ⇒ trigger trg_AddWebPoints: WebUser.WebPoints += points
  GET /get-recharge-logs  ───▶  own WebTransaction rows (30 s cache)
```

The package's `type` decides the payment method. The client never chooses it: it picks a package, and the server dispatches on that package's `RechargeInfo.type`.

| Package `type` | Body | DB insert | Response | Approval |
| --- | --- | --- | --- | --- |
| `1` GCash | Required, `{"type":"gcash",...}` | `transact_gcash`: reference plus optional proof image | `TransactionAccepted` | Administrator |
| `2` PayPal | Must be absent | `transact` | `PayPalApproval` | PayPal webhook |
| `3` Wise | Required, `{"type":"wise",...}` | `transact_v2`: reference | `TransactionAccepted` | Administrator |
| `4` PayMongo | Must be absent | `transact` | `PayMongoCheckoutCreated` | PayMongo webhook |

Each insert writes:

```text
TransactionType = package type, TransactionAmount = package points,
TransactionPrice = TransactionCost = package amount, TransactionStatus = 0 (DB default)
```

## Common field rules

- **Request bodies are lenient.** Unknown JSON fields are ignored, and there is no `deny_unknown_fields` or `validator`. Missing required fields or wrong types fail JSON extraction.
- **The transaction body is optional.** `POST /transaction/{id}` extracts its body as `Option<Json<TransactionPayload>>`. If the JSON is malformed, the content type is wrong, or the body exceeds the actix default **2 MiB** limit, the body is silently treated as absent. A GCash or Wise purchase then fails with `400 Missing transaction data`, not a parse error.
- **Path IDs are UUIDs.** A segment that is not a UUID makes the actix path extractor return a **plain-text 404**.
- **Money and points are JSON numbers** (`f64`, DB `MONEY`).
  - `amount` is the real-money price. PayPal charges it in **USD**. PayMongo charges it in **PHP**, converted as `amount × 100` and truncated to integer centavos.
  - `points` is the R-Coin credited.
  - `discount` is display-only and never applied.
- **Timestamps** are RFC 3339 strings with the API server's local offset.
- **Images are plain standard base64.** `proof_image` and `GatewayInfo.image` never carry a `data:` prefix in responses. Requests must not carry one either (see `POST /transaction/{id}`).
- **Integer codes:** `type` / `transaction_type` / `category` are 1 GCash, 2 PayPal, 3 Wise, 4 PayMongo. `transaction_status` / `status` are 0 Pending, 1 Approved, 2 Declined. Package and gateway `status` 1 means active. See [structs.md](structs.md#integer-codes).

## Session

| Item | Contract |
| --- | --- |
| Mechanism | `actix-session` `CookieSessionStore`, private (encrypted) cookie |
| Cookie name | env `COOKIE_NAME` (`YXGames` in the reviewed `.env`) |
| Attributes | `HttpOnly`, `SameSite=Lax`, `Secure` unless `WEB_DOMAIN` ends with `.local`, `Domain=WEB_DOMAIN` |
| Session payload | Key `user_data` holds `ProfileData`; the routes read `user_num`, `user_name`, `user_email` and `is_verified` |
| Issued by | The user login endpoint under `/api/v1/auth` (outside this contract) |
| Browser requirement | `fetch(..., {credentials: "include"})`; CORS allows credentials only for listed origins |
| Missing or expired | **500** `{"message":"No session found"}` (not 401; see [errors.md](errors.md#known-implementation-quirks)) |

Public endpoints run behind the same session middleware but never read the session.

### List active packages

`GET /api/v1/payments/get-recharge-info`

There is no query and no pagination, and the order is unspecified (there is no `ORDER BY`). The response is a bare array of active packages:

```json
[
  {"id": "7c9e6679-7425-40de-944b-e07fc1f90ae7", "type": 1, "amount": 500.0, "discount": 0.0, "points": 500.0, "status": 1},
  {"id": "1b4e28ba-2fa1-41d2-883f-0016d3cca427", "type": 2, "amount": 10.0, "discount": 0.0, "points": 1000.0, "status": 1}
]
```

`RechargeInfo`:

| Field | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `id` | Required, non-null UUID | `RechargeID`; use it as `{id}` in `POST /transaction/{id}` |
| `type` | Required, non-null i16 | Payment method code (1–4) |
| `amount` | Required, non-null f64 | Price in the method's currency |
| `discount` | Required, **nullable** f64 | Display-only; usually `0.0` |
| `points` | Required, non-null f64 | R-Coin credited on approval |
| `status` | Required, non-null i16 | Always `1` on this route |

Errors: **500** `{"error":"Failed to fetch recharge info"}`.

### Read one package

`GET /api/v1/payments/get-recharge-info/{id}`

Returns the `RechargeInfo`. Unlike the list, it does **not** filter by status.

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | `RechargeInfo` | Found |
| 400 | `{"error":"Invalid recharge ID format"}` | Nil UUID `00000000-0000-0000-0000-000000000000` |
| 404 | plain text | Path is not a UUID |
| 500 | `{"error":"Failed to fetch recharge info"}` | Not found (the DB error is not distinguished) or DB failure |

### Start a purchase

`POST /api/v1/payments/transaction/{id}`

```http
POST /api/v1/payments/transaction/7c9e6679-7425-40de-944b-e07fc1f90ae7
Cookie: YXGames=<session>
Content-Type: application/json
```

GCash package (type 1):

```json
{"type": "gcash", "reference_number": "1023456789012", "proof_image": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="}
```

Wise package (type 3):

```json
{"type": "wise", "reference_number": "P123456789"}
```

PayPal (type 2) and PayMongo (type 4) packages: send **no body**. Sending one routes the request to the `Invalid transaction data` 400.

`TransactionPayload` (internally tagged on `type`):

| Field | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `type` | Required string | `"gcash"` or `"wise"` (lowercase). It must match the package type: gcash↔1, wise↔3. |
| `reference_number` | Required, non-null string | Must be non-blank after trimming. At most 50 characters (`NVARCHAR(50)`); longer values fail the insert with a 500. Uniqueness is **not** checked. |
| `proof_image` | GCash only. Optional, nullable string | Plain standard base64 of the receipt image, at most 5,000,000 characters. Validation also accepts a string starting with `data:image`, but the DB layer then fails to base64-decode it and the request returns **500**. Send raw base64 only. |

Processing order:

1. Session lookup. A missing session returns 500 `No session found`.
2. `is_verified` must be true, otherwise 403.
3. Load the package by `{id}`. Not found, a nil UUID or a DB error all return 404 `Recharge ID not found`.
4. Dispatch on `(package.type, body)`:
   - `(1, gcash)` or `(3, wise)`: validate, then insert and spawn a Discord notification (env `DISCORD_SERVER_WEBHOOK`; the GCash proof image is attached).
   - `(1|3, none)`: 400 `Missing transaction data`.
   - `(2, none)`: insert, then create a PayPal order. Intent is CAPTURE, currency USD, one item named `R-Coin {points}`, SKU `RCOIN{points}-{user_num}`, `invoice_id` = the new transaction UUID. The return and cancel URLs come from env `PAYPAL_RETURN_URL` / `PAYPAL_CANCEL_URL`.
   - `(4, none)`: insert, then create a PayMongo checkout session. Payment methods are `card` and `gcash`, currency PHP. The line item is `Recharge for {points} R-Coin`. `reference_number` and `metadata.transaction_id` are both set to the new transaction UUID. Billing name and email come from the session. The success and cancel URLs come from env `PAYMONGO_SUCCESS_URL` / `PAYMONGO_CANCEL_URL`.
   - Anything else: 400 `Invalid transaction data`.

Success bodies:

```json
{"message": "Transaction processed", "data": {"reference_number": "P123456789"}, "recharge_info": {"id": "5a1d6c2e-8b3f-4e7a-9c1d-2f3e4a5b6c7d", "type": 3, "amount": 25.0, "discount": 0.0, "points": 2500.0, "status": 1}}
```

```json
{"approval_url": "https://www.sandbox.paypal.com/checkoutnow?token=5O190127TN364715T"}
```

```json
{"message": "PayMongo Checkout Session created", "checkout_session_id": "cs_example123", "status": "active", "approval_url": "https://checkout.paymongo.com/cs_example123", "recharge_info": {"id": "2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e6f", "type": 4, "amount": 499.0, "discount": 0.0, "points": 500.0, "status": 1}}
```

The created `transaction_id` is **not returned** for any method. Clients find it through `GET /get-recharge-logs`.

- **GCash / Wise:** `data` echoes the submitted payload without its `type` tag. For GCash it includes `proof_image`, which is `null` when the image was omitted.
- **PayPal and PayMongo:** redirect the browser to `approval_url`. The player lands on the configured return or success URL, but **that redirect is not proof of payment**. Only the webhook changes the status.

| HTTP | Envelope | Condition |
| --- | --- | --- |
| 200 | See above | Pending transaction created (and, for PayPal/PayMongo, provider session created) |
| 400 | `{"error": "...", "recharge_info": {...}}` | Validation failed: `Reference number cannot be empty`, `Proof image is too large`, `Invalid image format` |
| 400 | `{"message": "Missing transaction data", "recharge_info": {...}}` | GCash/Wise package without a usable body |
| 400 | `{"message": "Invalid transaction data", "recharge_info": {...}}` | Body sent to a PayPal/PayMongo package, or the tag does not match the package type |
| 403 | `{"message": "Email verification required to use this feature", "error_code": "EMAIL_NOT_VERIFIED"}` | Session user not verified |
| 404 | `{"message": "Recharge ID not found"}` | Package missing, nil UUID or DB error; a non-UUID path returns plain text |
| 422 | `{"message": "Transaction processed without data", "recharge_info": {...}}` | PayPal/PayMongo: insert failed |
| 500 | `{"message": "No session found"}` | No session |
| 500 | `{"message": "Error processing request", "recharge_info": {...}}` | GCash/Wise insert failed, including a `data:` prefixed `proof_image` and a reference longer than 50 characters |
| 500 | `{"error": "Invalid order configuration" \| "Order creation failed" \| "PayPal API compatibility issue. Please contact support.", "details": "..."}` | PayPal order build or API failure. The pending transaction row **remains**. |
| 500 | `{"error": "Approval link not found"}` | PayPal returned no `approve` link |
| 500 | `{"message": "Failed to create PayMongo Checkout Session: ...", "recharge_info": {...}}` | PayMongo API failure. The pending transaction row **remains**. |

**Idempotency:** there is none. Every successful call inserts a new Pending row and, for PayPal and PayMongo, a new provider session. Retrying after a timeout can create duplicates. Abandoned PayPal/PayMongo checkouts leave Pending rows indefinitely; nothing expires them.

### List active gateways

`GET /api/v1/payments/get-gateway-info`

This lists the account details players pay into for manual methods. It returns `GatewayInfo[]` with `status = 1`:

```json
[{"id": "0d4f7b9a-2c1e-4f6b-9a3d-5e8c7b6a1f20", "provider": 1, "name": "JUAN DELA CRUZ", "number": "9171234567", "image": "iVBORw0KGgo=", "status": 1}]
```

| Field | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `id` | Required UUID | `GatewayID` |
| `provider` | Required i16 | Opaque; there is no mapping in the code |
| `name` | Required string | Account holder |
| `number` | Required string | Account or mobile number, as stored (leading zeros may already be lost; see admin create) |
| `image` | Required string | Base64 image bytes (e.g. a QR code); rows with a NULL image are skipped |
| `status` | Required i16 | Always `1` here |

Errors: **500** `{"error":"Failed to fetch gateway info"}`.

### List own transactions

`GET /api/v1/payments/get-recharge-logs?page=1&limit=10&status=0&category=1`

| Query | Required / default | Validation and interpretation |
| --- | --- | --- |
| `page` | Optional, default `1` | 1-based. `0` underflows: it panics in debug builds and returns empty `data` in release builds. |
| `limit` | Optional, default `10` | Clamped to at most 100 |
| `status` | Optional i16 | Exact `transaction_status` match, applied in memory |
| `category` | Optional i16 | Exact `transaction_type` match, applied in memory |
| `search`, `start_date`, `end_date` | Accepted | **Ignored** |

Response `TransactionPage`:

```json
{
  "page": 1, "limit": 10, "total_pages": 1, "total_items": 1,
  "data": [{"transaction_id": "9f1c2d3e-4b5a-4c6d-8e7f-a0b1c2d3e4f5", "user_num": 10234, "user_name": "player01", "transaction_type": 1, "transaction_time": "2026-10-16T14:03:22+08:00", "transaction_price": 500.0, "transaction_status": 0, "transaction_amount": 500.0, "updated_at": null, "transaction_cost": 500.0, "proof_image": null, "reference_code": "1023456789012", "admin_remarks": null}]
}
```

- **`total_pages`** is `ceil(total_items / limit)`, or `1` when there are no items.
- **Ordering:** rows are sorted by `TransactionTime DESC`.
- **Caching:** results are cached per `(user_num, page, limit)` for 30 seconds. A new purchase can therefore take up to 30 s to appear on a page that was already viewed.

`WebTransactionResponse` field meanings are in [structs.md](structs.md#database-column-mapping). Note that `transaction_amount` is **points** and `transaction_price` / `transaction_cost` are **money**.

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | `TransactionPage` | — |
| 400 | plain text | A query value has the wrong type |
| 500 | `{"message":"No session found"}` | No session |
| 500 | `{"error":"Failed to fetch recharge logs"}` | DB failure |

## Source and verification boundaries

- **How this chapter was derived:** by reading the handlers, models, DB trait, stored procedure and trigger listed above.
- **What [build_contract.py](build_contract.py) `--check` proves:** the OpenAPI is valid; every request, response and schema example validates against its schema; and the documented `(path, method)` set equals the actix route macros in the two route files.
- **What it does not prove:** that a live server returns these bodies, or that PayPal/PayMongo sandbox flows succeed. It also does not cover the live MSSQL schema, which may differ from the scripts in `db-scripts/`.
