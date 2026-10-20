# Workflow: Admin Items Module

## Feature Summary

Build an admin-only read API over the `RanGameS1.dbo.Item` table in **GameDB**. The module exposes three JWT-protected endpoints: a paginated list with optional filters (`cha_num`, `item_main`/ItemMID, `item_sub`/ItemSID, `inven_type`/InvenType), a lookup by `ItemUUID` (MSSQL UNIQUEIDENTIFIER), and a paginated list scoped to a specific character number. This is a read-only admin tool for inspecting in-game item inventory data; there is no frontend consumer, so no TypeScript contract is produced.

## Agents Involved

1. **Architect** — Map existing GameDB admin patterns (characters module is the closest peer), lock down the Item row model, pagination envelope, UUID handling via Tiberius, filter semantics, and exact stored procedure signatures.
2. **CodeGen** — Implement the model file in `ran-models/src/admin/items/`, the GameDB trait + impl in `ran-db-core/src/gamemanager/admin/items/`, the route module in `ran-access-point/src/routes/admin/items/`, and wire everything into the existing `mod.rs` files.
3. **Debugger** *(conditional)* — Only invoked if `cargo check` fails, especially around Tiberius `Uuid` typing, `SmallDateTime` mapping, or TINYINT (`u8`) vs SMALLINT (`i16`) mismatches.
4. **Reviewer** — Verify JWT guard placement, pagination math, SQL-injection safety (all params bound), error handling via `anyhow::Context`, and pattern consistency with the existing admin characters module.

*ContractGen is intentionally omitted — admin-only, no frontend consumer.*

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

> You are the Architect for the gs-api Rust workspace (3 crates: `ran-access-point`, `ran-models`, `ran-db-core`; 5 MSSQL databases: WebDB, GameDB, ShopDB, UserDB, LogDB). Your job is to produce the implementation blueprint for a new **admin-only Items module** backed by `GameDB` (`RanGameS1.dbo.Item`). Do not write code — return a plan.
>
> **Feature scope (JWT-protected, registered under `admin_route()`):**
>
> - `GET /admin/items` — paginated list. Query params: `page` (u32, default 1), `page_size` (u32, default 20, clamp to a sane max such as 100), optional filters `cha_num` (i32), `item_main` (i16 → `ItemMID`), `item_sub` (i16 → `ItemSID`), `inven_type` (u8 → `InvenType`). Sort by `ItemCreateDate DESC`.
> - `GET /admin/items/{uuid}` — single item by `ItemUUID` (MSSQL `UNIQUEIDENTIFIER`).
> - `GET /admin/items/cha_num/{cha_num}` — paginated items filtered by `ChaNum` (same pagination params as the list endpoint).
>
> **Exploration tasks (read these files and summarize the concrete patterns you find — do not paraphrase, extract the exact shapes):**
>
> 1. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/character.rs` — existing item-adjacent DTO; see what naming/serde conventions it uses.
> 2. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/mod.rs` — the current module tree for admin items.
> 3. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/mod.rs` — `GameDB` struct, pool accessor, `DatabaseTrait` impl.
> 4. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/characters/` — this is the closest peer. Pull out: the trait name, async signature style, how `EXEC ProcName @P1 = @P1` is built, how rows are mapped (`try_get::<Type, _>("ColumnName")`), how pagination (offset/limit + total count) is returned, and how UUID or similar special columns are handled if any.
> 5. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/characters/` — the route module for characters. Extract: the `configure(cfg: &mut web::ServiceConfig)` shape, how `web::Query` is used for filters, how path params are parsed, how validation errors are returned, how the paginated response JSON is shaped.
> 6. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/items/mod.rs` and `dispatch.rs` — the existing (likely placeholder) admin items route module. Determine what is already wired so CodeGen does not duplicate scaffolding.
> 7. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/mod.rs` — confirm how `admin_route()` mounts `admin::items` and whether `JwtMiddleware` is applied at the scope or per-service.
> 8. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/mod.rs` — see how sub-modules (`characters`, and the new `items`) are re-exported and how the `GameDB` impl blocks are split.
> 9. Grep the workspace for an existing read of a `UNIQUEIDENTIFIER` column via Tiberius to confirm the exact Rust type used (`uuid::Uuid`, `tiberius::Uuid`, or a string representation) and the `try_get` turbofish that works. If none exists, note that this is a new pattern that CodeGen must introduce carefully; recommend `uuid::Uuid` with the `tiberius` feature flag if the `Cargo.toml` already enables it, otherwise accept the UUID as a string at the HTTP boundary and let the stored procedure `CAST` it.
>
> **Deliverables (return as a single structured response to the orchestrator — do NOT write a report file):**
>
> - **File plan** — exact absolute paths to be created or modified, grouped by crate.
> - **`Item` row DTO** — the Rust struct mirroring the selected columns of `RanGameS1.dbo.Item`. **Exclude these columns entirely — do not map them:** `BasicAttackDamage`, `BasicDefence`, `LockBox`, `ItemMainColor`, `ItemSubColor`, `TempGrindValue`, `TempGrindFireDate`, `CostumeStatType1`–`CostumeStatType7`, `CostumeInvestPoint1`–`CostumeInvestPoint7`. Specify exact Rust types per remaining column (note: TINYINT → `u8`, SMALLINT → `i16`, INT → `i32`, FLOAT(53) → `f64`, SMALLDATETIME → `chrono::NaiveDateTime`, UNIQUEIDENTIFIER → see exploration step 9). Call out every `serde` rename attribute needed to convert `PascalCase` SQL column names to `snake_case` JSON (e.g., `#[serde(rename = "item_uuid")]` on a field named `item_uuid` mapped from `ItemUUID`). Decide whether the entire struct uses `#[serde(rename_all = "snake_case")]` at the struct level — this is the cleaner approach.
> - **Pagination envelope** — the exact response shape (match the characters module; likely `{ "items": [...], "total": N, "page": N, "page_size": N }` or similar).
> - **Query DTO** — the `ItemListQuery` struct with `#[derive(Deserialize)]` and optional filter fields; document defaults and clamp rules for `page_size`.
> - **Trait design** — proposed trait name (e.g., `GameAdminItemsTrait`), three async method signatures, return types, and how filters are threaded to the stored procedure (all-optional parameters with `NULL` sentinels, or separate procs — pick one and justify based on the characters peer).
> - **Stored procedure contracts** — names and parameter lists for the three procs (see "SQL Stored Procedures Needed" below); specify input parameter names, types, and the expected result set shape (include total-count column strategy — e.g., a second result set, a window function `COUNT(*) OVER()`, or a separate count query).
> - **Edge cases** — invalid UUID format, UUID not found (404 vs empty response), `cha_num` with zero results, `page` out of range, `page_size = 0`, extremely large `page_size`, filter combinations that return empty sets, behavior when all filters are `None` (should return the full paginated list).
> - **Validation rules** — `page >= 1`, `1 <= page_size <= 100`, filter values bounded to the SQL column domains (reject `item_main`/`item_sub` values outside `i16` range — Rust type already enforces this).
> - **Risk flags** — read-only so low write-risk, but flag: (a) result payload size (80+ columns × page_size rows is large; confirm page_size cap), (b) UUID round-tripping, (c) SMALLDATETIME serialization to JSON (ISO 8601 via `chrono` serde feature).

### Phase 2 — CodeGen

**Depends on:** Architect output (file plan, struct shape, trait signatures, pagination envelope, stored procedure signatures).

**Prompt:**

> You are CodeGen for the gs-api Rust workspace. Implement the admin-only Items module exactly as specified by the Architect's plan. Before writing a single line, read the Architect's output end-to-end and then read the peer implementation at `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/characters/` and `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/characters/` — your output must mirror those patterns in style, error handling, naming, and response shape.
>
> **Files to create or modify:**
>
> 1. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/item.rs` — **create**. Define the `Item` struct using the exact field list from the Architect (which excludes `BasicAttackDamage`, `BasicDefence`, `LockBox`, `ItemMainColor`, `ItemSubColor`, `TempGrindValue`, `TempGrindFireDate`, `CostumeStatType1`–`CostumeStatType7`, `CostumeInvestPoint1`–`CostumeInvestPoint7`). Derive `Debug`, `Clone`, `Serialize`, `Deserialize`. Apply `#[serde(rename_all = "snake_case")]` at the struct level. Also define `ItemListQuery` (query DTO with optional `page`, `page_size`, `cha_num`, `item_main`, `item_sub`, `inven_type`) and `PaginatedItems` (response envelope) here, or in a sibling file if the Architect specifies.
> 2. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/mod.rs` — **modify**. Re-export the new types alongside the existing `character` re-exports.
> 3. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/items/mod.rs` — **create**. Define `GameAdminItemsTrait` with three async methods: `list_items(&self, query: &ItemListQuery) -> Result<PaginatedItems>`, `get_item_by_uuid(&self, uuid: <UuidType>) -> Result<Option<Item>>`, `list_items_by_character(&self, cha_num: i32, page: u32, page_size: u32) -> Result<PaginatedItems>`. Implement the trait for `GameDB` in the same file. Use `self.pool.get().await.context(...)?`, `EXEC AdminGetItems @P1 = @P1, ...` pattern, and manual `try_get::<Type, _>("ColumnName")` row mapping for every column. Use `anyhow::Context` on every `.await`.
> 4. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/mod.rs` — **modify**. Add `pub mod items;`.
> 5. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/items/dispatch.rs` — **modify or create**. Implement the three handlers. Each handler signature: `async fn name(_claims: JwtClaims, db: web::Data<Arc<DBState>>, ...) -> impl Responder`. Success responses return `HttpResponse::Ok().json(payload)`. Validation failures return `HttpResponse::BadRequest().json(flatten_validation_errors(&errors))`. Not-found on UUID returns `HttpResponse::NotFound().json(json!({"message": "Item not found"}))`. DB errors: log with `tracing::error!` and return `HttpResponse::InternalServerError().json(json!({"status":"error","message":"Internal server error"}))`.
> 6. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/items/mod.rs` — **modify**. Export `pub fn configure(cfg: &mut web::ServiceConfig)` that registers all three handlers (e.g., `cfg.service(list_items).service(get_item_by_uuid).service(list_items_by_character)`).
> 7. `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/mod.rs` — **verify and modify if needed**. Confirm `admin_route()` already has `web::scope("/items").configure(admin::items::configure)` (or equivalent) under the JWT-guarded admin scope. If not present, add it.
> 8. `db-scripts/gamedb/Procedures/AdminGetItems.sql` — **create**. Paginated list proc with optional filters (`@ChaNum INT = NULL`, `@ItemMID SMALLINT = NULL`, `@ItemSID SMALLINT = NULL`, `@InvenType TINYINT = NULL`, `@Page INT`, `@PageSize INT`). Use `ISNULL(@ChaNum, ChaNum)` pattern for optional filters. Return rows ordered by `ItemCreateDate DESC` plus a total count column via `COUNT(*) OVER()` window function. Select only the columns mapped in the `Item` struct (exclude the 21 omitted columns listed above).
> 9. `db-scripts/gamedb/Procedures/AdminGetItemByUUID.sql` — **create**. Single-row fetch by `@ItemUUID UNIQUEIDENTIFIER`. Select only the mapped columns.
> 10. `db-scripts/gamedb/Procedures/AdminGetItemsByCharacter.sql` — **create**. Paginated list filtered by `@ChaNum INT`, ordered by `ItemCreateDate DESC`, returning rows and total count via `COUNT(*) OVER()`. Select only the mapped columns.
>
> **Rules (non-negotiable — from `CLAUDE.md`):**
>
> - All DB calls go through stored procedures. Never inline SQL beyond `EXEC ProcName @P1 = @P1, ...`.
> - Pass parameters via Tiberius bind (`&[&param1, &param2]`) — never interpolate strings.
> - Use `anyhow::Result<T>` + `.context(...)` in the DB layer. Use pattern-matched `Result` handling in handlers.
> - Use `tracing::error!` / `tracing::info!` — never `println!` or `eprintln!`.
> - Never use `unwrap()` in handler or DB code.
> - No new error enums. Use `anyhow` and the existing `DatabaseError` (in `ran-models/src/errors/`).
> - Match the characters module's style exactly — derives, attribute order, doc comments (minimal), module layout.
> - Clamp `page_size` to `[1, 100]` and default `page` to 1 inside the handler (or via `serde(default)` on the DTO — match what the characters module does).
> - For UUID path parsing, use the type the Architect selected in exploration step 9. If that is `uuid::Uuid`, use `web::Path<uuid::Uuid>` and actix-web will reject malformed UUIDs with 400 automatically.
>
> After writing all files, run `cargo check -p ran-models`, then `cargo check -p ran-db-core`, then `cargo check -p ran-access-point` from the workspace root at `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api`. If any of these fail, stop and report the first error verbatim to the orchestrator so the Debugger phase can engage.

### Phase 3 — Debugger *(conditional: only if CodeGen's `cargo check` failed)*

**Depends on:** CodeGen output and the first build error.

**Prompt:**

> You are the Debugger for the gs-api Rust workspace. CodeGen just implemented the admin Items module and `cargo check` failed. Your task is to diagnose and fix the error without introducing new patterns.
>
> **High-probability failure modes to check first:**
>
> 1. **Tiberius `try_get` type mismatches** — TINYINT must be `u8`, SMALLINT `i16`, INT `i32`, BIGINT `i64`, FLOAT(53) `f64`, SMALLDATETIME `chrono::NaiveDateTime` (requires the `chrono` feature on Tiberius — check the root `Cargo.toml`). A mismatched turbofish (e.g., `try_get::<i32, _>("ItemPosX")` when the column is TINYINT) is the most common failure on this feature.
> 2. **UUID handling** — if `uuid::Uuid` was used, Tiberius needs the `uuid` feature flag. Verify at `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/Cargo.toml`. If the feature is missing, fall back to reading the UUID as a `String` and parsing it in the DTO layer, OR add the feature flag if that matches how other Tiberius features are already enabled in the workspace.
> 3. **`SMALLDATETIME` serialization** — if JSON serialization of `NaiveDateTime` fails, ensure `chrono` is pulled in with the `serde` feature in the relevant `Cargo.toml`. Do not swap to a string type without checking how the characters module handles datetime columns.
> 4. **Missing `pub mod` declarations** — the new `items` module must be declared in `ran-db-core/src/gamemanager/admin/mod.rs` and in `ran-models/src/admin/items/mod.rs`.
> 5. **Trait not in scope** — the handler must `use ran_db_core::gamemanager::admin::items::GameAdminItemsTrait;` (or whatever the re-export path is) to call the trait methods on `GameDB`.
> 6. **Actix-web extractor ordering** — `JwtClaims` must come before `web::Data` / `web::Path` / `web::Query` in handler argument lists if the characters module does it that way; follow the peer exactly.
>
> **Workflow:**
>
> 1. Run `cargo check -p ran-models`, `cargo check -p ran-db-core`, `cargo check -p ran-access-point` from `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api` to reproduce the error.
> 2. Read only the files named in the error output.
> 3. Apply the minimum change to make it compile. Do not refactor adjacent code.
> 4. Re-run `cargo check` on all three crates. Report the final clean build to the orchestrator.
>
> If `cargo clippy --workspace --all-targets -- -D warnings` surfaces additional issues, fix only the ones in the files CodeGen produced — do not touch pre-existing warnings in other crates.

### Phase 4 — Reviewer

**Depends on:** Clean `cargo check` across all three crates.

**Prompt:**

> You are the Reviewer for the gs-api Rust workspace. Review the admin Items module for correctness, security, and pattern consistency with the existing admin characters module. This is an admin-only read API over GameDB — the risk profile is: no data mutation, but the payload is large (80+ columns × page_size rows) and the JWT guard is the only gate on game-internal inventory data.
>
> **Files to review (read each one in full):**
>
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/item.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-models/src/admin/items/mod.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/items/mod.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/mod.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/items/mod.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/items/dispatch.rs`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/mod.rs` (diff only)
>
> **Peer references to compare against:**
>
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-db-core/src/gamemanager/admin/characters/`
> - `/Users/cedricestrada/Desktop/development/workspace-schmitt/prod/gs-api/ran-access-point/src/routes/admin/characters/`
>
> **Checklist:**
>
> 1. **JWT enforcement** — every one of the three handlers receives the JWT claims extractor (or is registered under a scope that applies `JwtMiddleware`). No handler can be invoked without a valid admin token.
> 2. **SQL safety** — every parameter to `EXEC` is bound via Tiberius (`&[&p1, &p2]`), never interpolated. All three procedures are called; no raw `SELECT` strings.
> 3. **Pagination correctness** — `page`/`page_size` default and clamp logic is present in the handler (not only at the SQL layer); offset math is correct (`(page - 1) * page_size`); total count is returned; `page = 0` or `page_size = 0` cannot produce a negative offset or divide-by-zero.
> 4. **Row mapping** — every column in the `Item` struct is extracted with the correct Tiberius type (TINYINT→`u8`, SMALLINT→`i16`, INT→`i32`, FLOAT(53)→`f64`, SMALLDATETIME→`NaiveDateTime`, UNIQUEIDENTIFIER→the chosen UUID type). No `unwrap()` on `try_get`.
> 5. **Error handling** — DB layer uses `.context(...)` on every `.await`. Handler layer pattern-matches `Result`, logs via `tracing::error!`, and returns the correct HTTP status. 404 on missing UUID, 400 on malformed UUID or bad query params, 500 on DB errors, 200 otherwise.
> 6. **Filter semantics** — optional filters are truly optional at the SQL layer (NULL sentinels in the stored procedure, or equivalent) so that omitting `cha_num` does not accidentally filter to `ChaNum = 0`.
> 7. **Serde attributes** — field renames map `PascalCase` SQL columns to `snake_case` JSON consistently. No leaked `ItemUUID`-style keys in the response.
> 8. **Response payload size** — `page_size` is capped (recommended: 100). Unbounded `page_size` on an 80+ column row is a DoS vector.
> 9. **Pattern consistency** — module layout, derive order, import order, handler argument order all match the admin characters module.
> 10. **CLAUDE.md compliance** — no `println!`/`eprintln!`, no `unwrap()`, no inline SQL, no new error enums, no speculative helpers, no ORM-style abstractions.
>
> **Run before signing off:**
>
> ```
> cargo check --workspace
> cargo clippy -p ran-models -p ran-db-core -p ran-access-point -- -D warnings
> ```
>
> Both must be clean. If anything surfaces, loop back to Debugger or CodeGen with a specific file-level change list. Return a final sign-off summary to the orchestrator listing each checklist item as pass/fail plus any residual risks.

## Files Expected to Change

**`ran-models` crate**

- `ran-models/src/admin/items/item.rs` *(new)* — `Item`, `ItemListQuery`, `PaginatedItems` DTOs
- `ran-models/src/admin/items/mod.rs` *(modify)* — re-export new module

**`ran-db-core` crate**

- `ran-db-core/src/gamemanager/admin/items/mod.rs` *(new)* — `GameAdminItemsTrait` + `impl ... for GameDB`
- `ran-db-core/src/gamemanager/admin/mod.rs` *(modify)* — add `pub mod items;`

**`ran-access-point` crate**

- `ran-access-point/src/routes/admin/items/dispatch.rs` *(modify)* — three handler functions
- `ran-access-point/src/routes/admin/items/mod.rs` *(modify)* — `configure()` registration
- `ran-access-point/src/routes/mod.rs` *(verify/modify)* — ensure the `/items` scope is mounted under `admin_route()`

**`db-scripts/` (written by CodeGen during development)**

- `db-scripts/gamedb/Procedures/AdminGetItems.sql`
- `db-scripts/gamedb/Procedures/AdminGetItemByUUID.sql`
- `db-scripts/gamedb/Procedures/AdminGetItemsByCharacter.sql`

## Risk Areas

- **Payload size** — `RanGameS1.dbo.Item` has 80+ columns. At `page_size = 100` that is 8,000+ field values per response. Hard-cap `page_size` at 100 (or lower) in the handler. Without this cap, a single admin call could stall the pool and bloat logs.
- **UUID round-tripping** — MSSQL `UNIQUEIDENTIFIER` encoding in Tiberius is version-sensitive. Confirm the exact feature flag (`uuid` or `tiberius-uuid`) against the workspace `Cargo.toml` during Architect exploration. If the project does not already read UUID columns anywhere, accept UUIDs as strings at the HTTP boundary and let the stored procedure `CAST(@P1 AS UNIQUEIDENTIFIER)`.
- **SMALLDATETIME precision** — `SMALLDATETIME` has minute-level precision only. When serialized to JSON via `chrono::NaiveDateTime`, trailing seconds may read as `:00`. This is expected; do not attempt millisecond fidelity.
- **Optional filter semantics** — if the stored procedure uses `WHERE ChaNum = ISNULL(@ChaNum, ChaNum)`, NULL filters behave as "any". Verify this pattern is used consistently for all four optional filters so that empty-query calls return the full paginated list.
- **JWT guard placement** — confirm `JwtMiddleware` wraps the `/admin` scope (not just some handlers). A single handler registered outside the guarded scope would leak inventory data.
- **Index coverage** — the three procs filter by `ChaNum`, `ItemUUID`, `ItemMID`, `ItemSID`, `InvenType` and order by `ItemCreateDate DESC`. Confirm with the DBA that supporting indexes exist before heavy use, otherwise admin calls could hot-spot GameDB.

## SQL Stored Procedures Needed

All in **GameDB** (`RanGameS1`), to be written by the developer under `db-scripts/gamedb/Procedures/`:

1. `AdminGetItems(@Page INT, @PageSize INT, @ChaNum INT = NULL, @ItemMID SMALLINT = NULL, @ItemSID SMALLINT = NULL, @InvenType TINYINT = NULL)` — paginated list with optional filters, ordered by `ItemCreateDate DESC`, returning both the rows and a total count.
2. `AdminGetItemByUUID(@ItemUUID UNIQUEIDENTIFIER)` — single-row fetch.
3. `AdminGetItemsByCharacter(@ChaNum INT, @Page INT, @PageSize INT)` — paginated list scoped to a character, ordered by `ItemCreateDate DESC`, returning rows and total count.
