# Errors, limits, and known quirks

## Error envelopes

These endpoints have **no single error envelope**. Four shapes occur, and which one you get depends on the handler, not on the HTTP status:

| Shape | Where | Example |
|---|---|---|
| `{"message": string, "recharge_info"?: RechargeInfo, "error_code"?: string}` (`PaymentsMessageError`) | Player `POST /transaction/{id}`, the no-session case, PayPal webhook | `{"message":"Recharge ID not found"}` |
| `{"error": string, "details"?: string, "recharge_info"?: RechargeInfo}` (`PaymentsErrorEnvelope`) | Player list/read failures, transaction validation, PayPal order failures | `{"error":"Reference number cannot be empty","recharge_info":{...}}` |
| `{"error": string}` (`RechargeAdminError`) | Every admin recharge handler | `{"error":"Failed to fetch gateway info"}` |
| `text/plain` | Admin JWT 401, actix extractor failures (path/query/JSON), PayMongo webhook, serde_qs | `Authorization token missing or invalid` |

Branch on the HTTP status first. Then read the body as text and try to parse it as JSON. A client should accept either `message` or `error` as the human-readable text. `message` and `error` are not stable codes. The only machine code is `error_code: "EMAIL_NOT_VERIFIED"`.

## Status table

| HTTP | Condition | Client action |
|---|---|---|
| 400 | Validation or missing data on `POST /transaction/{id}`; nil UUID; malformed query or JSON (plain text); webhook payload problems | Fix the input. Do not retry unchanged. |
| 401 | Admin JWT missing, invalid or expired (plain text). **Never returned for a missing player session.** | Admin re-login |
| 403 | `EMAIL_NOT_VERIFIED` on `POST /transaction/{id}` | Send the player to email verification |
| 404 | Recharge package not found on `POST /transaction/{id}`; any path segment that is not a UUID (plain text); admin create with zero rows affected | Reload the package list |
| 413 | JSON body over 2 MiB (actix default `JsonConfig`) on endpoints that extract `Json<T>` | Shrink the payload. On `POST /transaction/{id}` an oversized body is instead treated as absent, which gives a 400. |
| 415 | Wrong `Content-Type` on `Json<T>` endpoints | Send `application/json` |
| 422 | PayPal/PayMongo purchase: inserting the pending transaction failed | Retry later |
| 500 | Missing player session (`No session found`); DB failures; "not found" on several admin routes; a non-Pending transaction on status updates; provider API failures | See the quirks below. For a player, a 500 `No session found` means sign in again. |
| no response | Panics: invalid base64 or non-integer `number` on admin gateway routes; `page=0` in debug builds | Validate the input client-side before sending |

## Limits

| Item | Value |
|---|---|
| JSON body | 2 MiB (actix-web 4 default; no `JsonConfig` override) |
| `client_request_timeout` | 10 s (time to receive request headers) |
| Keep-alive | 15 s |
| Rate limiting | None on these scopes |
| Player list page size | `limit` clamped to at most 100 on `/get-recharge-logs` and admin `/recharge-logs`; not clamped elsewhere |
| `reference_number` | At most 50 chars (DB `NVARCHAR(50)`) |
| `proof_image` | At most 5,000,000 base64 chars by validation; the 2 MiB body limit applies first |
| CORS | Credentialed. Release origins: `https://ranonlinegs.com`, `https://api.paypal.com`, `http://tauri.localhost`, `tauri://localhost`. Debug builds also allow local and dev origins. |

## Retries and idempotency

There is no `Idempotency-Key` header, and no request carries a client-supplied operation ID.

| Operation | Retry guidance |
|---|---|
| GET lists | Safe to retry with backoff. Player transaction logs are cached for 30 s. |
| `POST /transaction/{id}` | **Not idempotent.** Each call inserts a new Pending row, plus a new PayPal order or PayMongo session where applicable. After a timeout, check `/get-recharge-logs` before retrying. |
| Admin `PUT /update-transaction/{id}` | Safe: only Pending rows change. A second approval returns 500 and does not credit twice. |
| Admin creates | Not idempotent: duplicates are possible. Created IDs are not returned, so reload the list. |
| Admin replaces | Idempotent: full replacement. |
| Webhooks | Status changes are guarded by `TransactionStatus = 0`, so redelivery cannot double-credit. A PayPal redelivery after settlement returns 500. PayMongo always returns 200. |

## Known implementation quirks

The contract documents these **as they behave today**. When replicating, decide on purpose whether to copy each one or fix it.

| # | Area | Behavior | Source |
|---|---|---|---|
| Q1 | Player session | A missing or expired session returns **500** `{"message":"No session found"}`, not 401. `get_user_session` returns an `Err(HttpResponse)` that the handlers discard. | `routes/payments/mod.rs#L659`, `#L1264` |
| Q2 | Admin `GET /recharge` pagination | `sp_GetWebTransactions` computes `@Offset = @Page * @PageSize`, but the handler defaults `page` to 1. Page 1 therefore skips the first `limit` rows; `page=0` is the real first page. | `sp_GetWebTransactions.sql` L21 |
| Q3 | Webhook authentication | Neither webhook verifies signatures. PayMongo's verifier exists but is commented out. | `routes/payments/mod.rs#L887` |
| Q4 | PayMongo webhook | Always returns 200 text once it parses, including on DB failure or a missing `transaction_id`. The inner `HttpResponse` values are discarded with `;`. | `routes/payments/mod.rs#L962` |
| Q5 | PayMongo webhook | Does not invalidate the buyer's profile cache (PayPal does). | `routes/payments/mod.rs#L1184` |
| Q6 | PayPal `CHECKOUT.CAPTURE.COMPLETED` | Shares the COMPLETED branch, but its status mapping is 0, so it is a no-op. | `routes/payments/mod.rs#L1151` |
| Q7 | `POST /transaction/{id}` body | A `proof_image` starting with `data:image` passes validation, then fails the base64 decode in the DB layer and returns 500. A malformed or oversized body is silently treated as "no body". | `routes/payments/mod.rs#L255`, `transaction/mod.rs#L657` |
| Q8 | `POST /transaction/{id}` errors | Any DB error while loading the package becomes 404 `Recharge ID not found`. A failed PayPal/PayMongo call leaves an orphan Pending row. | `routes/payments/mod.rs#L120` |
| Q9 | `GET /get-recharge-info/{id}` | A missing row returns 500 (not 404), and the internal error text says "Gateway". | `transaction/mod.rs#L532` |
| Q10 | Admin `update-recharge-info` | `discount` is silently ignored because it is not in the `UPDATE ... SET`. The handler is registered twice. | `transaction/mod.rs#L417`, `recharge/mod.rs#L26,L30` |
| Q11 | Admin gateway create/update | Panics (`unwrap`) on invalid base64 `image`, and on create, a non-integer `number`. Create also drops leading zeros from `number`. | `transaction/mod.rs#L279`, `#L475` |
| Q12 | `page=0` on in-memory paginators | `(page - 1)` underflows `usize`. Debug builds (`overflow-checks = true`) panic; release builds wrap and return empty `data`. | `recharge/mod.rs#L128`, `#L367`; `payments/mod.rs#L1320` |
| Q13 | Admin `add-recharge-info` | Uses inline SQL in the handler instead of a stored procedure. The INSERT-failure message says "Failed to update status". | `recharge/mod.rs#L381` |
| Q14 | Admin status update | Any i16 is accepted as `status`. A non-Pending row returns 500 with the internal anyhow message. | `recharge/mod.rs#L202` |
| Q15 | Admin JWT | Only signature and expiry are checked, with no role claim enforcement. | `services/jwt.rs#L89` |
| Q16 | Naming | `transaction_amount` is points, `transaction_price`/`transaction_cost` are money, the DB column is `RefferenceCode` (sic), and the gateway update's error says "recharge info". | `payments/mod.rs` (models) |
| Q17 | User transaction cache | The cache is keyed by `(user_num, page, limit)` with a 30 s TTL and is not invalidated on purchase. | `routes/payments/mod.rs#L1287` |
