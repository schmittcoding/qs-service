# RAN Online GS Payments & Recharge API contract

Contract date: **2026-10-16**. Audience: the game-source developer and their agents replicating gs-api's player payment, provider webhook and administrator recharge endpoints. This package follows the layout of the `project-strife-api` contract: per-domain Markdown chapters plus hand-written OpenAPI 3.1.1 fragments, assembled and validated by `build_contract.py`.

**API origin:** deployment-specific. Local development binds `APP_HOST:APP_PORT` (`http://127.0.0.1:5010` in the reviewed `.env`).
**Player scope:** `/api/v1/payments` (`API_VERSION=1`), cookie session.
**Admin scope:** `/admin/v1/recharge`, bearer JWT.
**Change status:** documentation only. No code, database or provider configuration was changed. Behavior is documented **as implemented today**, including defects, which are listed in [errors.md](errors.md#known-implementation-quirks).

## Start here

1. Read **[structs.md](structs.md)** first. It has the verbatim Rust/Serde structs, the endpoint → struct map, the integer code tables and the DB column mapping. Replicate from here.
2. Import **[openapi.json](openapi.json)** into OpenAPI 3.1 / JSON Schema 2020-12 tooling. Every schema and numeric field carries `x-rust-type`, and schemas carry `x-source` (file#line in gs-api).
3. Import **[postman.collection.json](postman.collection.json)** and **[postman.environment.json](postman.environment.json)**. The credentials are blank: fill `session_cookie` and `admin_jwt` yourself. Mutating requests change real data if sent to a live server.
4. Read the domain chapters before you implement handlers:
   - **[payments.md](payments.md)**: player endpoints and the purchase flow.
   - **[provider-webhooks.md](provider-webhooks.md)**: PayPal and PayMongo settlement.
   - **[recharge-admin.md](recharge-admin.md)**: the admin review, package and gateway endpoints.
   - **[errors.md](errors.md)**: error envelopes, limits, retries and the quirks table.

All examples use invented UUIDs, names, references and amounts. No secrets, tokens, cookies or database credentials are included.

## Architecture and ownership

```text
Player browser (cookie session)
  -> gs-api /api/v1/payments  (actix-web, ran-access-point)
       GET packages / gateways, POST /transaction/{id}, GET own logs
  -> RanWeb MSSQL: RechargeInfo, GatewayInfo, WebTransaction
       trigger trg_AddWebPoints: on TransactionStatus 0→1, WebUser.WebPoints += TransactionAmount

PayPal / PayMongo
  <- order / checkout session created by POST /transaction/{id} (transaction UUID embedded)
  -> /api/v1/payments/webhook/{paypal|paymongo}  (unauthenticated) -> status 1 or 2

Administrator (bearer JWT from /admin/v1/auth/authenticate)
  -> gs-api /admin/v1/recharge: review/approve GCash & Wise, manage packages and gateways
```

The server owns the price, points and payment method of each package; the client only picks a package ID. Points are credited **only** by the 0→1 status transition, through the trigger. Neither a browser redirect back from PayPal or PayMongo nor a `200` from `POST /transaction/{id}` means the player has been credited.

## Availability matrix

| Surface | Status in source | Caller |
|---|---|---|
| Package and gateway lists | Public, no auth | Player browser |
| Purchase via GCash (1) / Wise (3) | Session + verified email; manual admin approval | Player browser, then admin |
| Purchase via PayPal (2) / PayMongo (4) | Session + verified email; requires PayPal / PayMongo env credentials and URLs | Player browser, then provider |
| PayPal / PayMongo webhooks | Mounted, **unsigned** | Provider servers |
| Admin recharge routes | JWT (any admin token; no role check) | Administrator UI |

## Core HTTP contract

| Item | Contract |
|---|---|
| Versioning | `/api/v{API_VERSION}` and `/admin/v{API_VERSION}`; `API_VERSION=1` |
| JSON requests | `Content-Type: application/json`. Bodies are **not strict**: unknown fields are ignored. |
| Player auth | actix-session private cookie (name from `COOKIE_NAME`, `HttpOnly`, `SameSite=Lax`); send credentials |
| Admin auth | `Authorization: Bearer <HS256 JWT>`, claims `{sub, role, exp}` |
| Success status | Always 200 (there is no 201 or 204) |
| IDs | UUID strings (lowercase, hyphenated) |
| Money / points | JSON numbers (`f64`, DB `MONEY`). PayPal charges USD; PayMongo charges PHP centavos = `amount × 100`. |
| Dates | RFC 3339 with the API server's local offset |
| Images | Plain standard base64, no `data:` prefix |
| Integer codes | Type 1 GCash / 2 PayPal / 3 Wise / 4 PayMongo; status 0 Pending / 1 Approved / 2 Declined; record status 1 = active |
| Body limit | 2 MiB |
| Errors | Mixed `{message}` / `{error}` JSON and plain text. See [errors.md](errors.md). |
| Cache headers | None set by the application |

**How JSON Schema notation maps to Rust.** OpenAPI 3.0's `nullable: true` is never used.

| Rust | JSON Schema |
|---|---|
| `Option<T>` on a response | Listed in `required`, with type `[T, "null"]` (always present, may be null) |
| `Option<T>` on a request | Omitted from `required`, with type `[T, "null"]` |
| Request schemas | `additionalProperties: true` |
| Response schemas | `additionalProperties: false` |

## Endpoint map

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/api/v1/payments/get-recharge-info` | — | Active recharge packages |
| GET | `/api/v1/payments/get-recharge-info/{id}` | — | One package |
| POST | `/api/v1/payments/transaction/{id}` | Session | Start a purchase |
| GET | `/api/v1/payments/get-gateway-info` | — | Active manual-payment gateways |
| GET | `/api/v1/payments/get-recharge-logs` | Session | Own transactions |
| POST | `/api/v1/payments/webhook/paypal` | none (unsigned) | PayPal events |
| POST | `/api/v1/payments/webhook/paymongo` | none (unsigned) | PayMongo events |
| GET | `/admin/v1/recharge` | JWT | Transaction review list (multi-filter) |
| GET | `/admin/v1/recharge/recharge-logs` | JWT | Legacy transaction list |
| PUT | `/admin/v1/recharge/update-transaction/{id}?status=N` | JWT | Approve / decline a Pending transaction |
| GET | `/admin/v1/recharge/get-recharge-info` | JWT | Packages (filter, page) |
| POST | `/admin/v1/recharge/add-recharge-info` | JWT | Create a package |
| PUT | `/admin/v1/recharge/update-recharge-info/{id}` | JWT | Replace a package |
| GET | `/admin/v1/recharge/get-gateway-info` | JWT | All gateways |
| POST | `/admin/v1/recharge/add-gateway-info` | JWT | Create a gateway |
| PUT | `/admin/v1/recharge/update-gateway-info/{id}` | JWT | Replace a gateway |

## Product rules to preserve

- **A purchase creates a Pending transaction.** GCash and Wise wait for an administrator. PayPal and PayMongo wait for their webhook.
- **Only Pending transactions can change status.** This guard is what makes approval and webhook redelivery safe from double-crediting.
- **Points are credited by the database trigger** on 0→1. Declining (2) credits nothing, and there is no reversal or refund endpoint.
- **Purchasing requires a verified email** (`ProfileData.is_verified`).
- **The package decides the payment method and price.** Clients cannot pass an amount.
- **Packages and gateways with status ≠ 1 are hidden from players.** A package with any status can still be bought by ID through `POST /transaction/{id}`, because it does not check `status`.

## Explicitly absent features

This contract has none of the following:
- A refund or reversal API.
- Transaction cancellation by the player.
- Expiry of abandoned Pending transactions.
- Admin remarks write, or delete endpoints for packages or gateways.
- Webhook signature verification.
- Idempotency keys.
- A role-based admin permission check.

Do not infer endpoints for them.

## Contract maintenance and validation

The source fragments are in `parts/`. Reassemble and validate with Python 3.11+ and the pinned tools:

```sh
python -m pip install -r requirements-docs.txt
python build_contract.py --check
```

The build writes:
- `openapi.json`
- `postman.collection.json` and `postman.environment.json`
- `request-examples.json`
- `source-map.json` (SHA-256 of the gs-api source files the contract was derived from)
- `contract-checks.json`

`--check` validates the OpenAPI document and every schema, request and response example against JSON Schema 2020-12. It also asserts that the documented `(path, method)` set **exactly equals** the actix route macros in `ran-access-point/src/routes/payments/mod.rs` and `ran-access-point/src/routes/admin/recharge/mod.rs`.

Update the fragments, chapters and `structs.md` whenever any of these change:
- the handlers or route mounting
- the `ran-models` payment structs
- the `transaction` DB trait
- `sp_GetWebTransactions` or `trg_AddWebPoints`

If a hash in `source-map.json` changes, the contract may be stale.

Standards: [OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html), [Postman collection 2.1](https://schema.postman.com/json/collection/v2.1.0/docs/index.html). The Rust implementation, not this document, defines runtime behavior.
