# Recharge administration

This contract documents the current Rust implementation as of **2026-10-16**. API base: **`{api_origin}/admin/v1/recharge`**. Every route sits behind `JwtMiddleware`. The machine-readable fragment is [parts/recharge-admin.openapi.json](parts/recharge-admin.openapi.json). It defines the `AdminBearerJWT` security scheme and the admin request and response schemas, and reuses `RechargeInfo`, `GatewayInfo`, `WebTransactionResponse` and `TransactionPage` from the payments fragment. Verbatim Rust structs are in [structs.md](structs.md). Examples are fixtures.

Implementation sources:
- [Route handlers](../../ran-access-point/src/routes/admin/recharge/mod.rs#L21)
- [Admin scope and JWT wrap](../../ran-access-point/src/routes/admin/mod.rs#L29)
- [JWT middleware](../../ran-access-point/src/services/jwt.rs#L89)
- [DB trait](../../ran-db-core/src/webmanager/transaction/mod.rs#L45)
- [sp_GetWebTransactions](../../db-scripts/webdb/Procedures/Administration/sp_GetWebTransactions.sql)
- [Points trigger](../../db-scripts/webdb/Triggers/trg_AddWebPoints.sql)

## Endpoint inventory

Every success status is **200**, including the creates. Paths are relative to `/admin/v1/recharge`.

| Method and route | Body / query | Success body | Purpose |
| --- | --- | --- | --- |
| `GET` (scope root) | Query `TransactionFilterParams` (serde_qs) | `AdminTransactionPage` | Transaction review list with multi-value filters |
| `GET /recharge-logs` | Query `PaginationParams` | `TransactionPage` | Legacy transaction list with single-value filters |
| `PUT /update-transaction/{id}?status=N` | Query `UpdateStatusParams`; no body | `{"message":"Transaction updated successfully"}` | Approve (1) or decline (2) a Pending transaction |
| `GET /get-recharge-info` | Query `page`, `limit`, `status`, `category` | `AdminRechargeInfoPage` | List packages |
| `POST /add-recharge-info` | `AddRechargeInfo` | `{"message":"Added new recharge info successfully"}` | Create a package |
| `PUT /update-recharge-info/{id}` | `AddRechargeInfo` | `RechargeInfo` | Replace a package |
| `GET /get-gateway-info` | None | `GatewayInfo[]` (all statuses) | List gateways |
| `POST /add-gateway-info` | `AddGatewayInfo` | `{"message":"Added new gateway info successfully"}` | Create a gateway |
| `PUT /update-gateway-info/{id}` | `AddGatewayInfo` | `GatewayInfo` | Replace a gateway |

## Authority

```http
Authorization: Bearer ADMIN_JWT
Accept: application/json
```

| Item | Contract |
| --- | --- |
| Token | JWT, `HS256`, signed with env `SESSION_SECRET` |
| Claims | `{sub: string, role: string, exp: unix seconds}` (`Claims`); the token must decode with all three |
| Checks | Signature and `exp` only. **No role check**: any token the admin login issues is accepted on every route. |
| Prefix | Literal `Bearer ` (case-sensitive) |
| Issuer | `POST /admin/v1/auth/authenticate` (outside this contract) |
| Failure | **401** `text/plain` `Authorization token missing or invalid`, returned before the handler runs |

The approving administrator is not recorded anywhere (there is a TODO in the code).

## Common field rules

- **Request bodies are lenient.** Unknown fields are ignored and there are no `validator` rules, so zero or negative amounts are accepted.
- **Path `{id}` is a UUID.** Anything else gets actix's plain-text **404**.
- **Error envelope:** `{"error": "..."}`. Framework extraction errors are plain text, and the 401 above is plain text.
- **Two pagination styles exist.** `GET` (scope root) passes `page` straight to SQL, which treats it as 0-based (see the quirk below). `/recharge-logs` and `/get-recharge-info` paginate in memory and are 1-based.
- **Images** (`AddGatewayInfo.image`) are plain standard base64. Invalid base64 makes the DB layer **panic**, dropping the connection with no HTTP response.

### Transaction review list

`GET /admin/v1/recharge?page=0&limit=20&search=player&status[]=0&method[]=1&method[]=3`

| Query | Required / default | Validation and interpretation |
| --- | --- | --- |
| `page` | Optional, default `1` | Cast `usize → i16`. The SQL computes `OFFSET = page × limit`, so **`page=0` is the first page** and the default `1` skips the first `limit` rows. |
| `limit` | Optional, default `10` | Cast to `i16`, not clamped |
| `search` | Optional string | `UserName LIKE '%search%'` **or** `RefferenceCode = search` |
| `method[]` | Optional, repeatable i16 | Transaction types (1–4). An empty list means all. |
| `status[]` | Optional, repeatable i16 | Transaction statuses (0–2). An empty list means all. |

The query is parsed by `serde_qs` (depth 5, non-strict). Brackets may be written `[]`, indexed `[0]`, or percent-encoded. Repeating a key without brackets (`status=0&status=1`) is **not** supported.

Sort order is **Pending first**, then `TransactionTime DESC`.

```json
{
  "data": [{"transaction_id": "9f1c2d3e-4b5a-4c6d-8e7f-a0b1c2d3e4f5", "user_num": 10234, "user_name": "player01", "transaction_type": 1, "transaction_time": "2026-10-16T14:03:22+08:00", "transaction_price": 500.0, "transaction_status": 0, "transaction_amount": 500.0, "updated_at": null, "transaction_cost": 500.0, "proof_image": "iVBORw0KGgo=", "reference_code": "1023456789012", "admin_remarks": null}],
  "page": 0,
  "limit": 20,
  "total_items": 1
}
```

`total_items` is the filtered count from the procedure's first result set. There is **no `total_pages`**; compute it as `ceil(total_items / limit)`.

| HTTP | Body | Condition |
| --- | --- | --- |
| 400 | text `Invalid query: ...` | serde_qs parse failure |
| 401 | text | JWT |
| 500 | `{"error":"An unexpected error occurred. Please try again later."}` | DB failure |

### Legacy transaction list

`GET /admin/v1/recharge/recharge-logs?page=1&limit=10&status=0&category=1&search=player01`

| Query | Required / default | Validation and interpretation |
| --- | --- | --- |
| `page` | Optional, default `1` | 1-based. `0` underflows: it panics in debug builds and returns empty `data` in release builds. |
| `limit` | Optional, default `10` | Clamped to at most 100 |
| `status` | Optional i16 | Filtered in **SQL** |
| `category` | Optional i16 | Transaction type, filtered in memory |
| `search` | Optional string | In memory. Matches if `user_name` **contains** it (case-sensitive), **or** it equals the `transaction_id` string, **or** it equals `reference_code`. |
| `start_date`, `end_date` | Accepted | Ignored |

The endpoint loads **every** matching row (including proof images) before paginating. The response is `TransactionPage` `{page, limit, total_pages, total_items, data}`, ordered `TransactionTime DESC`. A DB failure returns **500** `{"error":"Failed to fetch recharge logs"}`.

### Approve or decline a transaction

`PUT /admin/v1/recharge/update-transaction/{transaction_id}?status=1`

The new status is a **query parameter**; any body is ignored.

| Query | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `status` | Required i16 | `1` approves, `2` declines. Any i16 is accepted, with no range check. |

Effects on success:

1. `UPDATE ... SET TransactionStatus = status WHERE TransactionID = id AND TransactionStatus = 0`.
2. If the new status is `1`, trigger `trg_AddWebPoints` credits `transaction_amount` points to the buyer.
3. The buyer's profile cache is invalidated.
4. A Discord embed is posted to env `DISCORD_TRANSACTION_WEBHOOK` (skipped if the variable is unset).

`admin_remarks` is not written.

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | `{"message":"Transaction updated successfully"}` | Pending row updated |
| 400 | text | `status` missing or not an integer |
| 404 | text | Path is not a UUID |
| 500 | `{"error":"Transaction with id <uuid> not found"}` | Unknown ID **or the row is no longer Pending**. Re-approving an approved transaction lands here, which also prevents double crediting. |

This endpoint is the settlement path for GCash (type 1) and Wise (type 3). It also works on Pending PayPal and PayMongo rows, for example after a lost webhook.

### List packages

`GET /admin/v1/recharge/get-recharge-info?page=1&limit=10&status=1&category=2`

| Query | Required / default | Validation and interpretation |
| --- | --- | --- |
| `page` | Optional, default `1` | 1-based. A page past the end is clamped to the last page. `0` underflows (panic in debug, empty in release). |
| `limit` | Optional, default `10` | Not clamped |
| `status` | Optional i16 | `RechargeStatus` filter in SQL. It is cast to `u8`, so negative values wrap. |
| `category` | Optional i16 | `RechargeType` filter in SQL |

```json
{"data": [{"id": "7c9e6679-7425-40de-944b-e07fc1f90ae7", "type": 1, "amount": 500.0, "discount": 0.0, "points": 500.0, "status": 1}], "total_items": 1}
```

The response does not echo `page` or `limit`. A DB failure returns **500** `{"error":"An unexpected error occurred. Please try again later."}`.

### Create a package

`POST /admin/v1/recharge/add-recharge-info`

```json
{"type": 1, "amount": 500.0, "discount": 10.0, "points": 550.0, "status": 1}
```

`AddRechargeInfo`:

| Field | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `type` | Required i16 | Payment method 1–4; this decides how players pay |
| `amount` | Required f64 | Price. It is charged in USD for PayPal and PHP for PayMongo. |
| `discount` | Optional, nullable f64 | Display-only. Written when present and non-null; otherwise the column default applies. |
| `points` | Required f64 | R-Coin credited on approval |
| `status` | Required i16 | `1` makes it visible to players |

The handler runs an inline `INSERT` (not a stored procedure). `RechargeID` defaults to `NEWSEQUENTIALID()`, and the new ID is **not returned**.

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | `{"message":"Added new recharge info successfully"}` | Inserted |
| 400 / 413 / 415 | text | Bad JSON, more than 2 MiB, or wrong content type |
| 404 | `{"error":"recharge info not found"}` | Zero rows affected |
| 500 | `{"error":"Database connection error"}` | Pool failure |
| 500 | `{"error":"Failed to update status"}` | INSERT failure (the message text is wrong) |

### Replace a package

`PUT /admin/v1/recharge/update-recharge-info/{id}`

The body is `AddRechargeInfo`, as above. The update sets `type`, `amount`, `points` and `status`. **`discount` is ignored on update**: the column is not in the `SET` clause, so the returned row shows the stored discount.

On success the response is the updated `RechargeInfo` (`OUTPUT inserted.*`). An unknown ID or a DB failure returns **500** `{"error":"Failed to fetch recharge info"}`.

### List gateways

`GET /admin/v1/recharge/get-gateway-info`

Returns `GatewayInfo[]` for **all** statuses, as a bare array. Rows with a NULL image are skipped. A DB failure returns **500** `{"error":"Failed to fetch gateway info"}`.

### Create a gateway

`POST /admin/v1/recharge/add-gateway-info`

```json
{"provider": 1, "name": "JUAN DELA CRUZ", "number": "09171234567", "image": "iVBORw0KGgo=", "status": 1}
```

`AddGatewayInfo`:

| Field | Required / nullable | Validation and interpretation |
| --- | --- | --- |
| `provider` | Required i16 | Opaque provider code |
| `name` | Required string | Account holder |
| `number` | Required string | Must parse as `i64`, otherwise the request **panics** (no HTTP response). The parsed integer is stored, so leading zeros are lost: `"09171234567"` becomes `"9171234567"`. |
| `image` | Required string | Plain standard base64. Invalid base64 **panics**. |
| `status` | Required i16 | `1` makes it visible to players |

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | `{"message":"Added new gateway info successfully"}` | Inserted. The ID is not returned. |
| 400 / 413 / 415 | text | Bad JSON, size or content type |
| 500 | `{"error":"Failed to add gateway info"}` | DB failure |

### Replace a gateway

`PUT /admin/v1/recharge/update-gateway-info/{id}`

The body is `AddGatewayInfo`, and every field is replaced. `number` is stored exactly as sent here (there is no integer parse on update). Invalid base64 in `image` **panics**.

On success the response is the updated `GatewayInfo`. An unknown ID or a DB failure returns **500** `{"error":"Failed to fetch recharge info"}` (the message says "recharge" even though this is the gateway route).

## Source and verification boundaries

These facts come from reading the code:

- The admin scope wiring in `main.rs` and `routes/admin/mod.rs`.
- The JWT middleware.
- The handlers, the DB trait, `sp_GetWebTransactions` and `trg_AddWebPoints`.

`build_contract.py --check` matches the nine `#[get|post|put]` macros in the recharge route file against this contract. `update_recharge_info` is registered twice in `configure()`, but that is a single route. No live admin requests were sent.
