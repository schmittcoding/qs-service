# Provider webhooks

This contract documents the current Rust implementation as of **2026-10-16**. The endpoints are server-to-server, under **`{api_origin}/api/v1/payments/webhook/*`**. The machine-readable fragment is [parts/webhooks.openapi.json](parts/webhooks.openapi.json); `RechargeInfo` and the status codes are shared with [parts/payments.openapi.json](parts/payments.openapi.json). The examples are synthetic.

Implementation sources:
- [PayPal handler](../../ran-access-point/src/routes/payments/mod.rs#L716)
- [PayMongo handler](../../ran-access-point/src/routes/payments/mod.rs#L875)
- [Invoice-ID extraction](../../ran-access-point/src/routes/payments/mod.rs#L1116)
- [Status mapping](../../ran-access-point/src/routes/payments/mod.rs#L1142)
- [PayMongo webhook structs](../../ran-access-point/src/routes/payments/paymongo_integration.rs#L205)
- [Status update](../../ran-db-core/src/webmanager/transaction/mod.rs#L91)
- [Points trigger](../../db-scripts/webdb/Triggers/trg_AddWebPoints.sql)

## Endpoint inventory

| Method and path | Caller | Body | Success |
| --- | --- | --- | --- |
| `POST /api/v1/payments/webhook/paypal` | PayPal | PayPal event JSON | 200 `{"message": ...}` |
| `POST /api/v1/payments/webhook/paymongo` | PayMongo | PayMongo event JSON | 200 `text/plain` `Webhook received and processed` |

## Responsibility boundary

The webhooks are the only way a PayPal (type 2) or PayMongo (type 4) transaction leaves **Pending**. Each one does exactly this:

1. Resolve the gs-api `transaction_id` that was embedded at checkout.
2. Run `UPDATE WebTransaction SET TransactionStatus = @new WHERE TransactionID = @id AND TransactionStatus = 0`.

Crediting is done by the database trigger, not by Rust. `trg_AddWebPoints` adds `TransactionAmount` (the package points) to `WebUser.WebPoints` for each row that changes from non-1 to 1. Because only Pending rows can change, a transaction is credited at most once, even if a provider delivers the same event several times.

| Correlation key | Set by `POST /transaction/{id}` as | Read back from |
| --- | --- | --- |
| PayPal | `purchase_units[0].invoice_id = transaction_id` | `resource.invoice_id`, then `resource.purchase_units[0].invoice_id`, then `resource.custom` |
| PayMongo | checkout `metadata.transaction_id = transaction_id` (and `reference_number`) | `data.attributes.data.attributes.metadata.transaction_id` |

## Authentication

**There is none.** PayPal signature verification is absent (the HMAC helper is commented out). The PayMongo `PayMongo-Signature` check exists in `paymongo_integration.rs`, but its call is commented out. Anyone who can reach these URLs and knows a Pending transaction UUID can approve that transaction. If you replicate these endpoints, add provider signature verification. This document describes the endpoints as they are.

## PayPal

`POST /api/v1/payments/webhook/paypal`, with `Content-Type: application/json`.

Only these fields are read:

```text
{ event_type, resource: { id, invoice_id?, purchase_units?: [{ invoice_id? }], custom? } }
```

```json
{
  "id": "WH-58D329510W468432D-8HN650336L201105X",
  "event_type": "PAYMENT.CAPTURE.COMPLETED",
  "resource": {"id": "2GG279541U471931P", "invoice_id": "9f1c2d3e-4b5a-4c6d-8e7f-a0b1c2d3e4f5", "status": "COMPLETED"}
}
```

The handler first validates that `resource`, `resource.id` (a string) and `event_type` are present. It then dispatches on `event_type`:

| `event_type` | Action | New status | Response |
| --- | --- | --- | --- |
| `CHECKOUT.ORDER.APPROVED` | Capture the order through the PayPal API using `resource.id`. If the invoice ID resolves, invalidate the buyer's profile cache. | Unchanged | 200 `Capture successful` / 500 `Failed to capture order` |
| `PAYMENT.CAPTURE.COMPLETED` | Update the transaction and invalidate the buyer's profile cache | `1` | 200 `Webhook processed successfully` / 500 `Failed to process transaction` |
| `CHECKOUT.CAPTURE.COMPLETED` | Same branch as above, but the status mapping yields `0` | `0` (no effect) | Same as above |
| `PAYMENT.CAPTURE.DENIED`, `PAYMENT.CAPTURE.DECLINED` | Update the transaction. A failed update is only logged. | `2` | 200 `Payment declined webhook received` |
| anything else (or a non-string `event_type`) | None | — | 200 `Webhook received, but not processed` |

The normal PayPal sequence is: buyer approves → `CHECKOUT.ORDER.APPROVED` (server captures) → `PAYMENT.CAPTURE.COMPLETED` (status 1, points credited).

| HTTP | Body | Condition |
| --- | --- | --- |
| 400 | `{"message":"Missing resources in webhook payload"}` | No `resource` |
| 400 | `{"message":"Missing order_id in webhook payload"}` | `resource.id` missing or not a string |
| 400 | `{"message":"Missing event_type in webhook payload"}` | No `event_type` |
| 400 | `{"message":"Missing custom_id in webhook payload"}` | COMPLETED event without a resolvable invoice ID |
| 400 | `{"message":"Invalid transaction UUID"}` | The invoice ID is not a UUID (COMPLETED or DENIED/DECLINED) |
| 400 / 415 | plain text | Malformed JSON / wrong content type (actix extractor) |
| 500 | `{"message":"Failed to process transaction"}` | COMPLETED update failed: unknown ID, the row is **no longer Pending** (e.g. a redelivery), or a DB error. PayPal retries non-2xx deliveries. |
| 500 | `{"message":"Failed to capture order"}` | PayPal capture API failure |

## PayMongo

`POST /api/v1/payments/webhook/paymongo`. The body is read as raw bytes, decoded as UTF-8, and parsed as `PayMongoWebhookPayload`:

```json
{
  "data": {
    "id": "evt_example123",
    "type": "event",
    "attributes": {
      "type": "payment.paid",
      "livemode": false,
      "data": {
        "id": "pay_example456",
        "type": "payment",
        "attributes": {"amount": 49900, "currency": "PHP", "status": "paid", "metadata": {"transaction_id": "9f1c2d3e-4b5a-4c6d-8e7f-a0b1c2d3e4f5"}}
      },
      "created_at": 1792130000,
      "updated_at": 1792130000
    }
  }
}
```

| Field path | Required / nullable | Interpretation |
| --- | --- | --- |
| `data.id`, `data.type` | Required strings | Event ID / `"event"`. Not used. |
| `data.attributes.type` | Required string | The event name. Only `payment.paid` and `payment.failed` are handled. |
| `data.attributes.livemode` | Required bool | Not checked: test-mode events also settle transactions |
| `data.attributes.created_at`, `updated_at` | Required u64 | Unix seconds. Not used. |
| `data.attributes.data.id`, `.type` | Required strings | Payment ID / `"payment"`. Not used. |
| `data.attributes.data.attributes` | Required object | Re-parsed only for handled events. It must contain a `metadata` key, which may be `null`. |
| `...attributes.metadata.transaction_id` | Optional UUID string | The gs-api transaction to settle |

| Event | New status |
| --- | --- |
| `payment.paid` | `1` (points credited by the trigger) |
| `payment.failed` | `2` |
| any other | ignored |

Responses:

| HTTP | Body | Condition |
| --- | --- | --- |
| 200 | text `Webhook received and processed` | **Always** once the payload parses. This includes unknown events, a missing `transaction_id`, and failed or duplicate status updates, so PayMongo never retries those. |
| 400 | text `Invalid UTF-8 payload` | Body is not UTF-8 |
| 400 | text `Invalid payload format` | Body does not match `PayMongoWebhookPayload` |
| 400 | text `Error processing payment.paid event` / `...payment.failed event` | The inner attributes have no `metadata` key |
| 400 | JSON `{"message":"Invalid transaction UUID "}` (trailing space) | `metadata.transaction_id` is not a UUID |

Unlike PayPal, this handler does **not** invalidate the buyer's profile cache. The player's displayed `web_points` can stay stale until the cache entry is refreshed or the player signs in again.

## Source and verification boundaries

- **How this was derived:** the event names, branches and messages come from reading the handlers.
- **Not tested:** no PayPal or PayMongo sandbox delivery was replayed for this document.
- **Inferred, not observed:** PayPal's real event payloads also contain many fields this API ignores. The status that PayMongo reports for a newly created checkout session (`active`) is inferred from PayMongo's API, not observed.
