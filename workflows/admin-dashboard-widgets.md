# Workflow: Admin Dashboard Widgets

## Feature Summary

Build four admin-only GET endpoints under `/admin/v1/dashboard/` that return raw integer metrics for the admin frontend dashboard. The endpoints cover: currently online players (GameDB), total registered accounts (WebDB), transaction count by status (WebDB), and total recharge amount by time period (WebDB). All routes are protected by the existing `JwtMiddleware`. Responses are plain JSON integers — no wrapper object — making the contract trivial but requiring careful handler serialization to avoid wrapping in a struct.

## Agents Involved

1. **Architect** — Explores the existing `metrics.rs` route/trait files, the `WebTransaction` and `GSUserInfo`/`ChaInfo` tables, the existing admin route registration in `mod.rs`, and produces the definitive data-flow and procedure signatures before any code is touched.
2. **SQLGen** — Writes three stored procedures: one in GameDB (online player count via `ChaOnline`), one in WebDB (registered user count), one combined WebDB procedure for transaction count by status, and one combined WebDB procedure for total recharge amount by time period.
3. **CodeGen** — Adds trait methods to the existing `gamemanager/admin/metrics.rs` and creates a new `webmanager/admin/dashboard.rs` trait file; creates the new `ran-access-point/src/routes/admin/dashboard/mod.rs` handler file; registers the new scope in `admin/mod.rs`.
4. **OpenAPI Validator** — Verifies all four handler functions carry `#[utoipa::path(...)]` annotations and are registered in `ran-access-point/src/openapi.rs`.
5. **Reviewer** — Reviews all new and modified files for pattern correctness, security, and consistency with the rest of the admin layer.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are the Architect agent for the gs-api Rust workspace. The workspace has three crates:
- `ran-access-point` — actix-web 4 HTTP server (binary)
- `ran-models` — shared data models and DTOs
- `ran-db-core` — async database traits backed by Tiberius + deadpool against five MSSQL databases: WebDB (`RanWeb`), GameDB (`RanGameS1`), ShopDB, UserDB (`RanUser`), LogDB

All DB access uses stored procedures. Row mapping is manual via `.try_get()`. Auth for admin routes uses `JwtMiddleware` (JWT Bearer). The existing admin route tree is registered in `ran-access-point/src/routes/admin/mod.rs`.

**Feature:** Four new admin-only GET endpoints returning a single integer each:

| Endpoint | Description | Database |
|---|---|---|
| `GET /admin/v1/dashboard/online` | Count of players currently online | GameDB |
| `GET /admin/v1/dashboard/registered` | Count of total registered web accounts | WebDB |
| `GET /admin/v1/dashboard/transactions?type={pending\|approved\|declined}` | Count of WebTransaction rows by status | WebDB |
| `GET /admin/v1/dashboard/transactions/total?type={today\|month\|year}` | Sum of `TransactionAmount` from approved WebTransaction rows, filtered by time period | WebDB |

**Your tasks:**

1. Read the following files in full before writing anything:
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/mod.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/metrics.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/gamemanager/admin/metrics.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/gamemanager/admin/mod.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/mod.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/players/mod.rs`
   - `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Tables/CreateTransaction.sql`
   - `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetWebTransactions.sql`
   - `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql`
   - `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/openapi.rs`

2. Determine the exact MSSQL table and column to use for each metric:
   - Online players: `RanGameS1.dbo.ChaInfo.ChaOnline` (a flag column already used in `sp_GetCharacters`). Confirm the column type and what value represents "online".
   - Registered accounts: `RanWeb.dbo.WebUser` table — `COUNT(*)` of all rows (or only `UserAvailable = 1` if that is the correct business definition; read the table DDL to decide).
   - Transaction count by status: `RanWeb.dbo.WebTransaction.TransactionStatus` (0 = pending, 1 = approved, 2 = declined — verify from existing code).
   - Recharge total by period: `SUM(TransactionAmount)` from `RanWeb.dbo.WebTransaction` where `TransactionStatus = 1` (approved), filtered by `TransactionTime` for today / current month / current year.

3. Determine the correct Rust trait placement:
   - Online count: should be a new method on `GameMetricsTrait` in `ran-db-core/src/gamemanager/admin/metrics.rs`. That file already exists and already has one trait method.
   - WebDB dashboard counts: should go in a new file `ran-db-core/src/webmanager/admin/dashboard.rs` as a new trait `WebDashboardTrait`. Explain why a new file is correct vs. adding to `players/mod.rs`.

4. Determine the correct route module placement:
   - The four endpoints should live in a new file `ran-access-point/src/routes/admin/dashboard/mod.rs`.
   - A new scope `/dashboard` should be added inside the JWT-guarded block in `admin/mod.rs`.

5. Identify every file that must be created or modified, with the change type (create / modify).

6. Describe the complete request flow for each endpoint (HTTP → middleware → handler → DB trait → stored procedure → MSSQL → response).

7. List every stored procedure that SQLGen must write, with:
   - Procedure name
   - Target database
   - Input parameters (name, T-SQL type, nullable)
   - Expected output columns (name, T-SQL type)
   - The business logic in plain English

8. Flag any edge cases:
   - What to return when there are zero rows matching (0 is a valid integer response, not a 404).
   - How `TransactionAmount` is typed in T-SQL (`MONEY`) and how that maps to Rust (`f64` via Tiberius `Numeric` or explicit cast to `BIGINT`/`INT`). Recommend the safest approach.
   - Whether the `transactions/total` endpoint should return a float or integer, and why.

Produce your findings as structured text. Do not write any code yet.

---

### Phase 2 — SQLGen

**Depends on:** Architect output (procedure names, target databases, parameter signatures, output columns)

**Prompt:**

You are the SQLGen agent for the gs-api project. You write T-SQL stored procedures saved to `db-scripts/` in the repository root at `/Users/schmitt/development/ran-online/gs/gs-api/`.

The folder structure is:
- `db-scripts/webdb/Procedures/Administration/` — admin procedures for WebDB (`RanWeb`)
- `db-scripts/gamedb/StoredProcedure/` — procedures for GameDB (`RanGameS1`)

**Before writing any procedure**, read the following existing scripts to match their style exactly (USE statement, GO separators, SET ANSI_NULLS ON, SET QUOTED_IDENTIFIER ON, CREATE OR ALTER PROCEDURE):
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetWebTransactions.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql`

Also read the table DDL to know exact column names and types:
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Tables/CreateTransaction.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Tables/CreateWebUser.sql`

**Write the following four stored procedures:**

**Procedure 1: `sp_GetOnlinePlayerCount`**
- Target database: GameDB (`RanGameS1`)
- Save to: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/StoredProcedure/sp_GetOnlinePlayerCount.sql`
- Input parameters: none
- Output: single row, single column `OnlineCount INT`
- Logic: `SELECT COUNT(*) AS OnlineCount FROM RanGameS1.dbo.ChaInfo WITH (NOLOCK) WHERE ChaOnline = 1`
- Note: Use `SET NOCOUNT ON`. Use `WITH (NOLOCK)` for performance.

**Procedure 2: `sp_GetRegisteredUserCount`**
- Target database: WebDB (`RanWeb`)
- Save to: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetRegisteredUserCount.sql`
- Input parameters: none
- Output: single row, single column `RegisteredCount INT`
- Logic: `SELECT COUNT(*) AS RegisteredCount FROM RanWeb.dbo.WebUser WITH (NOLOCK)` — count all rows, do not filter by `UserAvailable`. This is the total accounts-ever-created metric for the dashboard.
- Note: `SET NOCOUNT ON`.

**Procedure 3: `sp_GetTransactionCountByStatus`**
- Target database: WebDB (`RanWeb`)
- Save to: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetTransactionCountByStatus.sql`
- Input parameters:
  - `@Status SMALLINT NOT NULL` — maps from the `type` query param: `pending` → 0, `approved` → 1, `declined` → 2
- Output: single row, single column `TransactionCount INT`
- Logic: `SELECT COUNT(*) AS TransactionCount FROM RanWeb.dbo.WebTransaction WITH (NOLOCK) WHERE TransactionStatus = @Status`
- Note: `SET NOCOUNT ON`.

**Procedure 4: `sp_GetTotalRechargeAmount`**
- Target database: WebDB (`RanWeb`)
- Save to: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetTotalRechargeAmount.sql`
- Input parameters:
  - `@Period NVARCHAR(10) NOT NULL` — accepts `'today'`, `'month'`, or `'year'`
- Output: single row, single column `TotalAmount DECIMAL(18,2)` — cast from MONEY with explicit `CAST(SUM(TransactionAmount) AS DECIMAL(18,2))`. If no rows match, return `0.00`.
- Logic:
  - Filter `RanWeb.dbo.WebTransaction` where `TransactionStatus = 1` (approved only)
  - `'today'`: `CAST(TransactionTime AS DATE) = CAST(GETDATE() AS DATE)`
  - `'month'`: `YEAR(TransactionTime) = YEAR(GETDATE()) AND MONTH(TransactionTime) = MONTH(GETDATE())`
  - `'year'`: `YEAR(TransactionTime) = YEAR(GETDATE())`
  - Use `ISNULL(SUM(...), 0)` to guarantee a non-null return even when there are no matching rows.
- Note: `SET NOCOUNT ON`. Use `WITH (NOLOCK)`.

**After writing all four files**, output a summary table with:
- Exact file path written
- Procedure name
- Parameter names and T-SQL types
- Return column name and T-SQL type

SQLGen must output this summary table so CodeGen can use the exact names without guessing.

---

### Phase 3 — CodeGen

**Depends on:** Architect output (file placement decisions), SQLGen summary table (procedure names and exact parameter/return column names)

**Prompt:**

You are the CodeGen agent for the gs-api Rust workspace located at `/Users/schmitt/development/ran-online/gs/gs-api/`. The workspace has three crates: `ran-access-point`, `ran-models`, `ran-db-core`. All DB access is via async traits backed by Tiberius stored procedure calls. Row mapping is manual via `.try_get()`. Admin routes use `JwtMiddleware` (JWT Bearer).

**Before writing any code**, read all of the following files:
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/gamemanager/admin/metrics.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/gamemanager/admin/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/players/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/players/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/openapi.rs`

**Task 1 — Extend `GameMetricsTrait` in `ran-db-core/src/gamemanager/admin/metrics.rs`**

Add one new async trait method and its `impl` block for `GameDB`:

```
async fn get_online_player_count(&self) -> Result<i32>
```

Implementation:
- Get a connection from `self.pool`.
- Execute `"EXEC sp_GetOnlinePlayerCount"` with no parameters.
- Call `.into_row()` on the result stream.
- Extract the `OnlineCount` column as `i32` using `.try_get::<i32, _>("OnlineCount")`.
- Use `.context("Failed to get online player count")` on each fallible call.
- Return `Ok(count)`. If the row is `None`, return `Ok(0)`.

Do not remove or change the existing `get_character_metrics` method.

**Task 2 — Create `ran-db-core/src/webmanager/admin/dashboard.rs`**

This is a new file. Model it after the structure of `ran-db-core/src/webmanager/admin/players/mod.rs`.

Define a new trait `WebDashboardTrait` on `WebDB` with three methods:

```
async fn get_registered_user_count(&self) -> Result<i32>
async fn get_transaction_count_by_status(&self, status: i16) -> Result<i32>
async fn get_total_recharge_amount(&self, period: &str) -> Result<f64>
```

Implementations:
- `get_registered_user_count`: EXEC `sp_GetRegisteredUserCount`, no params, extract `RegisteredCount` as `i32`, return `Ok(0)` if row is `None`.
- `get_transaction_count_by_status`: EXEC `sp_GetTransactionCountByStatus @Status = @P1`, param `&status`, extract `TransactionCount` as `i32`, return `Ok(0)` if `None`.
- `get_total_recharge_amount`: EXEC `sp_GetTotalRechargeAmount @Period = @P1`, param `&period.to_string()` (pass as `String` so Tiberius treats it as `NVARCHAR`), extract `TotalAmount` as `f64`. Tiberius maps T-SQL `DECIMAL` to `f64` via `Numeric` — use `.try_get::<f64, _>("TotalAmount")`. Return `Ok(0.0)` if `None`.

**Task 3 — Register `dashboard` module in `ran-db-core/src/webmanager/admin/mod.rs`**

Add `pub mod dashboard;` to the existing file. Do not remove existing lines.

**Task 4 — Create `ran-access-point/src/routes/admin/dashboard/mod.rs`**

Model handlers after `ran-access-point/src/routes/admin/players/mod.rs` (pattern: no session, uses `web::Data<Arc<DBState>>`, returns `HttpResponse`, uses `tracing::error!`, includes `#[utoipa::path]` annotation, early-returns on DB error).

Create the file at `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/dashboard/mod.rs`.

Define a `configure` function that registers all four handlers.

**Handler 1: `get_online_count`**
- Route: `#[get("/online")]`
- No query params.
- Calls `db.game_db.get_online_player_count().await`.
- On success: `HttpResponse::Ok().json(count)` where `count: i32`. This serializes as a bare JSON integer.
- On error: `tracing::error!` + `HttpResponse::InternalServerError().json(json!({"error": "Failed to fetch online player count"}))`.
- `#[utoipa::path]`: tag `"Admin - Dashboard"`, path `/admin/v1/dashboard/online`, `operation_id = "admin_get_online_count"`, security `bearer_jwt`, response `(status = 200, description = "Online player count", body = i32)`.

**Handler 2: `get_registered_count`**
- Route: `#[get("/registered")]`
- No query params.
- Calls `db.web_db.get_registered_user_count().await`.
- On success: `HttpResponse::Ok().json(count)` where `count: i32`.
- On error: same pattern as handler 1.
- `#[utoipa::path]`: tag `"Admin - Dashboard"`, path `/admin/v1/dashboard/registered`, `operation_id = "admin_get_registered_count"`.

**Handler 3: `get_transaction_count`**
- Route: `#[get("/transactions")]`
- Query param struct (define inline in the file):
  ```rust
  #[derive(Deserialize)]
  struct TransactionCountQuery {
      r#type: String,  // "pending" | "approved" | "declined"
  }
  ```
- Maps `type` string to `i16`: `"pending"` → `0`, `"approved"` → `1`, `"declined"` → `2`. Any other value returns `HttpResponse::BadRequest().json(json!({"error": "Invalid type. Expected: pending, approved, declined"}))`.
- Calls `db.web_db.get_transaction_count_by_status(status).await`.
- On success: `HttpResponse::Ok().json(count)` where `count: i32`.
- `#[utoipa::path]`: tag `"Admin - Dashboard"`, path `/admin/v1/dashboard/transactions`, `operation_id = "admin_get_transaction_count"`, params `("type" = String, Query, description = "Transaction status filter: pending, approved, or declined")`.

**Handler 4: `get_recharge_total`**
- Route: `#[get("/transactions/total")]`
- Query param struct (define inline):
  ```rust
  #[derive(Deserialize)]
  struct RechargeTotalQuery {
      r#type: String,  // "today" | "month" | "year"
  }
  ```
- Validates that `type` is one of `"today"`, `"month"`, `"year"`. Any other value returns `400 BadRequest`.
- Calls `db.web_db.get_total_recharge_amount(&query.r#type).await`.
- On success: `HttpResponse::Ok().json(total)` where `total: f64`.
- `#[utoipa::path]`: tag `"Admin - Dashboard"`, path `/admin/v1/dashboard/transactions/total`, `operation_id = "admin_get_recharge_total"`, params `("type" = String, Query, description = "Time period: today, month, or year")`.

**Important route ordering:** In the `configure` function, register `get_recharge_total` before `get_transaction_count` if actix-web path matching could conflict, OR use separate scopes. Since `/transactions` and `/transactions/total` are distinct literal paths registered via `#[get]` macros, actix-web resolves them correctly — register both in the same `configure` without nesting.

**Task 5 — Register the dashboard module in `ran-access-point/src/routes/admin/mod.rs`**

Add `mod dashboard;` to the module declarations and add this line inside the JWT-guarded service scope (matching the indentation of the existing `.service(...)` lines):
```rust
.service(actix_web::web::scope("/dashboard").configure(dashboard::configure))
```

Do not remove or reorder any existing lines.

**Task 6 — Register the four handlers in `ran-access-point/src/openapi.rs`**

In the `paths(...)` list inside `#[openapi(...)]`, add these four paths under an `// Admin - Dashboard` comment:
```
crate::routes::admin::dashboard::get_online_count,
crate::routes::admin::dashboard::get_registered_count,
crate::routes::admin::dashboard::get_transaction_count,
crate::routes::admin::dashboard::get_recharge_total,
```

Add the import in the `#[allow(unused_imports)] use crate::routes::{...}` block at the top of `openapi.rs`:
```rust
admin::dashboard::{get_online_count, get_registered_count, get_transaction_count, get_recharge_total},
```

After all edits, run:
```
cargo check -p ran-db-core
cargo check -p ran-access-point
```
and report any errors.

---

### Phase 4 — Debugger *(if needed)*

**Depends on:** CodeGen output, build errors from `cargo check`

**Prompt:**

You are the Debugger agent for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api/`. The following files were recently created or modified as part of the admin dashboard widget feature:

- `ran-db-core/src/gamemanager/admin/metrics.rs` — extended with `get_online_player_count`
- `ran-db-core/src/webmanager/admin/dashboard.rs` — new file, `WebDashboardTrait`
- `ran-db-core/src/webmanager/admin/mod.rs` — added `pub mod dashboard;`
- `ran-access-point/src/routes/admin/dashboard/mod.rs` — new file, four handlers
- `ran-access-point/src/routes/admin/mod.rs` — registered `/dashboard` scope
- `ran-access-point/src/openapi.rs` — registered four paths

Run the following commands and fix every error reported:
```
cargo check -p ran-db-core 2>&1
cargo check -p ran-access-point 2>&1
```

Common issues to check for:
1. `WebDashboardTrait` not imported in the handler file — add `use ran_db_core::webmanager::admin::dashboard::WebDashboardTrait;` to `dashboard/mod.rs`.
2. `GameMetricsTrait` import in `dashboard/mod.rs` — add `use ran_db_core::gamemanager::admin::metrics::GameMetricsTrait;`.
3. Tiberius `f64` extraction for `DECIMAL` columns — if `try_get::<f64, _>` fails at runtime, try extracting as `tiberius::numeric::Numeric` and converting. If the column was cast to `DECIMAL(18,2)` in T-SQL, Tiberius should handle it as `f64` directly.
4. Missing `Deserialize` import for inline query param structs.
5. The `r#type` field in query structs — Tiberius does not see this but serde's `Deserialize` should rename it correctly from the `type` query param key. If the field name `r#type` does not deserialize from the URL key `type`, add `#[serde(rename = "type")]` on the field.
6. If `get_recharge_total` route and `get_transaction_count` route conflict in actix-web, split them into sub-scopes or reorder registration so the more specific `/transactions/total` is registered first.

Do not refactor code outside the files listed above. Trace each error to its root cause and apply the minimal fix.

---

### Phase 5 — OpenAPI Validator

**Depends on:** CodeGen output (compiling successfully)

**Prompt:**

You are the OpenAPI Validator agent for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api/`. A new set of four admin dashboard handlers were just added as part of the admin dashboard widget feature. Your job is to verify that every handler is properly documented with utoipa annotations and registered in the central OpenAPI spec.

**Read the following files before validating:**
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/dashboard/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/openapi.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/players/mod.rs` (reference for correct annotation style)

**Checklist — verify each item for all four handlers (`get_online_count`, `get_registered_count`, `get_transaction_count`, `get_recharge_total`):**

1. Each handler function has a `#[utoipa::path(...)]` attribute above its `#[get(...)]` attribute.
2. Each `utoipa::path` includes:
   - `get` as the HTTP method
   - `path` matching the full route as it appears in the admin scope: `/admin/v1/dashboard/online`, `/admin/v1/dashboard/registered`, `/admin/v1/dashboard/transactions`, `/admin/v1/dashboard/transactions/total`
   - `operation_id` that is unique across the entire OpenAPI spec (check `openapi.rs` for conflicts)
   - `tag = "Admin - Dashboard"`
   - `security(("bearer_jwt" = []))` — all admin routes require this
   - `responses(...)` block with at least a `200` response
   - `params(...)` block for `get_transaction_count` (the `type` query param) and `get_recharge_total` (the `type` query param)
3. All four handler paths are listed in the `paths(...)` macro inside `#[openapi(...)]` in `ran-access-point/src/openapi.rs`.
4. No handler imports are missing from the top-level `use crate::routes::...` block in `openapi.rs`.
5. Run `cargo check -p ran-access-point` to confirm the OpenAPI macro compiles without errors.

For any item that fails the checklist, apply the fix directly to the file. Report a final summary of what was missing and what was fixed.

---

### Phase 6 — Reviewer

**Depends on:** All prior phases complete (code compiles, OpenAPI validated)

**Prompt:**

You are the Reviewer agent for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api/`. A new admin dashboard widget feature was just implemented. Review all new and modified files for correctness, security, and consistency with the rest of the codebase.

**Read every file listed below before writing your review:**
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/gamemanager/admin/metrics.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/dashboard.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/dashboard/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/openapi.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/StoredProcedure/sp_GetOnlinePlayerCount.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetRegisteredUserCount.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetTransactionCountByStatus.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Procedures/Administration/sp_GetTotalRechargeAmount.sql`

Also read these reference files to assess pattern conformance:
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/players/mod.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/webmanager/admin/players/mod.rs`

**Review checklist:**

1. **Auth**: None of the four dashboard handlers perform session-based auth. They all rely exclusively on `JwtMiddleware` (which runs as a scope-level middleware in `admin/mod.rs`). Confirm that no handler contains a redundant `req.extensions().get::<Claims>()` check. The existing `get_character_metrics` handler does a redundant check — the new handlers should NOT repeat that pattern. If any redundant JWT check is present, flag it as a bug (it returns `401` when the middleware already guarantees the request has a valid JWT).

2. **Input validation**: Confirm that `get_transaction_count` and `get_recharge_total` return `400 BadRequest` for unrecognized `type` values, not `500`. Verify the status mapping (`pending` → 0, `approved` → 1, `declined` → 2) matches the `TransactionStatus` column values used elsewhere in the codebase (check `ran-access-point/src/routes/admin/players/mod.rs` lines that use status 0/1/2).

3. **Response shape**: Confirm all four handlers return `HttpResponse::Ok().json(integer_or_float)` with no wrapper object. A bare `i32` or `f64` serializes as a JSON number, which is the required contract.

4. **Error handling**: All DB errors must use `tracing::error!`, not `log::error!`. The codebase standard is `tracing`. If `log::error!` is used in the new files, flag it for correction.

5. **DB trait pattern**:
   - All `impl` blocks use `anyhow::Context` on every `await?` call.
   - No `unwrap()` in DB trait impls.
   - The `None` row case returns `Ok(0)` / `Ok(0.0)`, not an `Err`.

6. **SQL scripts**: Verify all four SQL files follow the existing pattern (`USE [db]; GO; SET ANSI_NULLS ON; GO; SET QUOTED_IDENTIFIER ON; GO; CREATE OR ALTER PROCEDURE ...`). Check that `sp_GetTotalRechargeAmount` uses `ISNULL(SUM(...), 0)` so it never returns NULL, which would panic the Tiberius `f64` extraction if not handled.

7. **Module registration**: Confirm `pub mod dashboard;` was added to `ran-db-core/src/webmanager/admin/mod.rs` and `mod dashboard;` was added to `ran-access-point/src/routes/admin/mod.rs`.

8. Run:
   ```
   cargo check -p ran-db-core
   cargo check -p ran-access-point
   cargo clippy -p ran-access-point -- -D warnings
   ```
   Fix any warnings or errors before sign-off.

Produce a written review with pass/fail for each checklist item, and apply any fixes inline.

## Files Expected to Change

### `ran-db-core`
| File | Change |
|---|---|
| `src/gamemanager/admin/metrics.rs` | Modified — add `get_online_player_count` method to existing trait and impl |
| `src/webmanager/admin/dashboard.rs` | Created — new `WebDashboardTrait` with three methods |
| `src/webmanager/admin/mod.rs` | Modified — add `pub mod dashboard;` |

### `ran-access-point`
| File | Change |
|---|---|
| `src/routes/admin/dashboard/mod.rs` | Created — four handler functions and `configure` fn |
| `src/routes/admin/mod.rs` | Modified — add `mod dashboard;` and register `/dashboard` scope |
| `src/openapi.rs` | Modified — import and register four new paths |

### `db-scripts`
| File | Change |
|---|---|
| `db-scripts/gamedb/StoredProcedure/sp_GetOnlinePlayerCount.sql` | Created |
| `db-scripts/webdb/Procedures/Administration/sp_GetRegisteredUserCount.sql` | Created |
| `db-scripts/webdb/Procedures/Administration/sp_GetTransactionCountByStatus.sql` | Created |
| `db-scripts/webdb/Procedures/Administration/sp_GetTotalRechargeAmount.sql` | Created |

### `ran-models`
No changes required. The response is a raw `i32` or `f64` — no new struct needed.

## Risk Areas

- **`MONEY` → `f64` type mapping (medium risk):** Tiberius maps T-SQL `MONEY` to its `Numeric` type, not directly to `f64`. The SQL scripts must `CAST` the sum to `DECIMAL(18,2)` before returning so Tiberius can extract it as `f64` reliably. If the column comes back as `MONEY` and `.try_get::<f64, _>()` fails, the Debugger agent must switch to `try_get::<rust_decimal::Decimal, _>()` or add an explicit `AS FLOAT` cast in the procedure. This is the most likely runtime failure point.
- **`r#type` serde deserialization (low risk):** Actix-web uses serde to deserialize query strings. The raw identifier `r#type` should deserialize from the URL key `type` automatically, but if it does not, `#[serde(rename = "type")]` must be added. Debugger agent should watch for a 400 with "missing field" when `type` is provided.
- **Route ordering for `/transactions` vs `/transactions/total` (low risk):** Actix-web 4 resolves literal routes before parameterized ones, but since both are literal paths under the same scope, registration order matters if one is a prefix of the other. Because both are registered with `#[get]` macros and then added to `configure()` via `.service()`, actix-web evaluates them by specificity — `/transactions/total` is more specific and should match correctly. If not, registering them as separate scopes (`/transactions` scope with a `/total` sub-route) resolves the ambiguity.
- **No models needed (no risk):** The response is a bare scalar, so `ran-models` is untouched. There is no serialization complexity.
- **Auth bypass (no risk if patterns followed correctly):** The `JwtMiddleware` wraps the entire admin scope. Dashboard endpoints must not add a redundant `Claims` check — doing so creates a false negative (the `else` branch returns 401 even though the middleware already validated the token). The Reviewer agent is explicitly instructed to flag this.

## SQL Stored Procedures Needed

| Procedure | Database | Input Parameters | Output Columns |
|---|---|---|---|
| `sp_GetOnlinePlayerCount` | GameDB (`RanGameS1`) | none | `OnlineCount INT` |
| `sp_GetRegisteredUserCount` | WebDB (`RanWeb`) | none | `RegisteredCount INT` |
| `sp_GetTransactionCountByStatus` | WebDB (`RanWeb`) | `@Status SMALLINT NOT NULL` | `TransactionCount INT` |
| `sp_GetTotalRechargeAmount` | WebDB (`RanWeb`) | `@Period NVARCHAR(10) NOT NULL` | `TotalAmount DECIMAL(18,2)` |

**Status value mapping for `sp_GetTransactionCountByStatus`:**

| `type` query param | `@Status` value |
|---|---|
| `pending` | `0` |
| `approved` | `1` |
| `declined` | `2` |

**Period value mapping for `sp_GetTotalRechargeAmount`:**

| `type` query param | SQL filter |
|---|---|
| `today` | `CAST(TransactionTime AS DATE) = CAST(GETDATE() AS DATE)` |
| `month` | `YEAR(TransactionTime) = YEAR(GETDATE()) AND MONTH(TransactionTime) = MONTH(GETDATE())` |
| `year` | `YEAR(TransactionTime) = YEAR(GETDATE())` |
