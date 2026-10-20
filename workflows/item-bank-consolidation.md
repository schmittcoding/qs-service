# Workflow: Item Bank Module Consolidation

## Feature Summary

Consolidate three scattered admin endpoint groups — purchases (item bank entries), item catalog/options, and item dispatch — into a single cohesive `/item-bank` admin module. This is a refactoring task: no new SQL procedures or tables are needed. Existing stored procedures are reused, and old routes are preserved via backward-compatible re-exports until the frontend migrates.

## Agents Involved

1. **Architect** — Verify the plan, trace data flows, confirm file paths and edge cases
2. **CodeGen** — Generate all new Rust files (models, DB traits, route handlers) and modify existing files
3. **Debugger** — Fix compile errors from the cross-crate refactoring
4. **OpenAPI Validator** — Ensure new route handlers have proper utoipa annotations
5. **Reviewer** — Review for correctness, security, pattern consistency, backward compatibility
6. **ContractGen** — Generate TypeScript types and API contracts for frontend handoff

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are the Architect for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`. The workspace has 3 crates: `ran-access-point` (HTTP server, actix-web 4), `ran-models` (shared DTOs), `ran-db-core` (DB layer, trait-based access to MSSQL via stored procedures). Admin auth uses JWT Bearer tokens via `JwtMiddleware`.

We are consolidating three scattered admin endpoint groups into a single `/item-bank` admin module. Read the approved plan at `/Users/schmitt/.claude/plans/i-need-your-help-tidy-forest.md` for full context.

Your task:

1. **Verify the plan** by reading the current source files:
   - `ran-access-point/src/routes/admin/purchases.rs` — current entry handlers
   - `ran-access-point/src/routes/admin/items/mod.rs` — current item options handler + game item handlers
   - `ran-access-point/src/routes/admin/items/dispatch.rs` — current dispatch handlers
   - `ran-db-core/src/shopmanager/purchases.rs` — PurchasesTrait
   - `ran-db-core/src/shopmanager/items/mod.rs` — ItemTrait
   - `ran-db-core/src/shopmanager/items/dispatch.rs` — ItemDispatchTrait
   - `ran-models/src/admin/purchases.rs` — ShopPurchaseBase
   - `ran-models/src/admin/items/mod.rs` — ItemMapOptions, ItemListQuery, Item
   - `ran-models/src/admin/items/dispatch.rs` — all dispatch DTOs
   - `ran-access-point/src/routes/admin/mod.rs` — admin route registration
   - `ran-db-core/src/shopmanager/mod.rs` — ShopDB struct and module declarations
   - `ran-models/src/admin/mod.rs` — admin model module declarations

2. **Confirm the duplicate**: Verify that `ItemDispatchTrait::get_dispatch_item_options()` and `ItemTrait::get_item_map_options()` execute the same SQL query.

3. **Trace data flows** for each endpoint being consolidated:
   - `GET /purchases` → handler → `PurchasesTrait::get_shop_purchases` → `sp_GetShopPurchases`
   - `GET /purchases/{id}` → handler → `PurchasesTrait::get_shop_purchase_by_id` → `sp_GetShopPurchaseById`
   - `GET /purchases/{user_num}/user` → handler → `PurchasesTrait::get_user_shop_purchases` → `sp_GetUserShopPurchases`
   - `GET /items/item-options` → handler → `ItemTrait::get_item_map_options` → raw SQL on ShopItemMap
   - `GET /items/dispatch` → handler → `ItemDispatchTrait::get_dispatch_list` → `sp_GetItemDispatchList`
   - `GET /items/dispatch/{id}` → handler → `ItemDispatchTrait::get_dispatch_item_by_id` → `sp_GetDispatchItemById`
   - `POST /items/dispatch` → handler → `ItemDispatchTrait::dispatch_item` → `sp_DispatchItem`

4. **Identify edge cases**:
   - Which imports/use statements reference `ShopPurchaseBase`, `ItemMapOptions`, `PurchasesTrait`, `ItemTrait`, `ItemDispatchTrait` from their current locations? These need backward-compatible re-exports.
   - Does anything outside the admin routes use these types (e.g., user-facing routes)?
   - Are there any `cfg` feature flags or conditional compilation that affects these modules?

5. **Output**:
   - Confirmed file list to create and modify
   - Any risks or edge cases not covered in the plan
   - The exact import paths that need backward-compatible re-exports
   - Confirmation that game item endpoints in `/items` are independent and won't break

---

### Phase 2 — CodeGen

**Depends on:** Architect output

**Prompt:**

You are the CodeGen agent for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`. The workspace has 3 crates: `ran-access-point` (actix-web 4 HTTP server), `ran-models` (shared DTOs), `ran-db-core` (DB layer with trait-based MSSQL access via stored procedures on `ShopDB`). Admin routes use JWT auth via `JwtMiddleware`.

Read the approved plan at `/Users/schmitt/.claude/plans/i-need-your-help-tidy-forest.md` for full context.

You are implementing Phase 1 of the Item Bank Module Consolidation — creating the new module and backward-compatible re-exports. No new SQL is needed.

**Before writing any code**, read these existing files to understand the exact patterns:
- `ran-db-core/src/shopmanager/purchases.rs` — PurchasesTrait pattern
- `ran-db-core/src/shopmanager/items/mod.rs` — ItemTrait pattern
- `ran-db-core/src/shopmanager/items/dispatch.rs` — ItemDispatchTrait pattern
- `ran-models/src/admin/purchases.rs` — ShopPurchaseBase model pattern
- `ran-models/src/admin/items/mod.rs` — ItemMapOptions model
- `ran-models/src/admin/items/dispatch.rs` — all dispatch models
- `ran-access-point/src/routes/admin/purchases.rs` — handler pattern for entries
- `ran-access-point/src/routes/admin/items/mod.rs` — handler pattern for catalog
- `ran-access-point/src/routes/admin/items/dispatch.rs` — handler pattern for dispatch
- `ran-access-point/src/routes/admin/mod.rs` — route registration pattern
- `ran-db-core/src/shopmanager/mod.rs` — module declaration pattern
- `ran-models/src/admin/mod.rs` — module declaration pattern
- `ran-models/src/lib.rs` — for `PaginationParams` and `BaseDataCountResult`

**Files to create:**

1. **`ran-models/src/admin/item_bank/mod.rs`**
   - Re-export everything from `entry.rs`, `catalog.rs`, `dispatch.rs`

2. **`ran-models/src/admin/item_bank/entry.rs`**
   - Define `ItemBankEntry` struct (same fields as `ShopPurchaseBase`: purchase_id, user_num, user_name, product_num, purchase_date, last_change_date, purchase_status, item_name, item_main, item_sub)
   - Same `from_row` implementation mapping the same SQL columns
   - Derive `Serialize`

3. **`ran-models/src/admin/item_bank/catalog.rs`**
   - Define `ItemCatalogOption` struct (same fields as `ItemMapOptions`: product_num, name, item_main, item_sub)
   - Same `from_row` implementation
   - Derive `Debug, Serialize, Deserialize, Clone`

4. **`ran-models/src/admin/item_bank/dispatch.rs`**
   - Move ALL dispatch models here: `ItemDispatch`, `ItemDispatchList`, `ItemDispatchItem`, `ItemDispatchCharacter`, `ItemDispatchDetailRequestPayload`, `ItemDispatchDetailItem`, `ItemDispatchDetailCharacter`
   - Keep exact same field names, derives, and `from_row` implementations

5. **`ran-db-core/src/shopmanager/item_bank/mod.rs`**
   - Re-export all sub-traits from `entries.rs`, `catalog.rs`, `dispatch.rs`

6. **`ran-db-core/src/shopmanager/item_bank/entries.rs`**
   - Define `ItemBankEntriesTrait` with same methods as `PurchasesTrait`:
     - `get_entries(page, limit, search, status)` → `Result<Vec<ItemBankEntry>>`
     - `get_entry_by_id(id)` → `Result<Row>`
     - `get_user_entries(user_num, page, limit, status)` → `Result<BaseDataCountResult>`
   - Implement on `ShopDB` with same stored procedure calls

7. **`ran-db-core/src/shopmanager/item_bank/catalog.rs`**
   - Define `ItemBankCatalogTrait` with single method:
     - `get_catalog_items()` → `Result<Vec<Row>>`
   - Implement on `ShopDB` with: `SELECT ProductNum, ItemMain, ItemSub, ItemName FROM dbo.ShopItemMap ORDER BY ItemName ASC`
   - This consolidates the duplicate from `ItemTrait` and `ItemDispatchTrait`

8. **`ran-db-core/src/shopmanager/item_bank/dispatch.rs`**
   - Define `ItemBankDispatchTrait` with methods (NO `get_dispatch_item_options` — that's now in catalog):
     - `get_dispatch_list(page, page_size, search)` → `Result<BaseDataCountResult>`
     - `get_dispatch_by_id(id: Uuid)` → `Result<ItemDispatch>`
     - `dispatch_item(items_json, characters_json, notes, admin_id)` → `Result<(String, Uuid)>`
   - Implement on `ShopDB` with same stored procedure calls as current `ItemDispatchTrait`

9. **`ran-access-point/src/routes/admin/item_bank/mod.rs`**
   - `pub fn configure(cfg: &mut actix_web::web::ServiceConfig)` that registers:
     - `web::scope("/entries").configure(entries::configure)`
     - `web::scope("/dispatches").configure(dispatch::configure)`
     - catalog handler directly (flat, not scoped)

10. **`ran-access-point/src/routes/admin/item_bank/entries.rs`**
    - `GET ""` → `get_entries` (list all entries, paginated)
    - `GET "/{id}"` → `get_entry_by_id` (single entry)
    - `GET "/user/{user_num}"` → `get_user_entries` (entries for a user)
    - Use `ItemBankEntriesTrait` from the new DB module
    - Follow same handler patterns as current `purchases.rs`

11. **`ran-access-point/src/routes/admin/item_bank/catalog.rs`**
    - `GET ""` → `get_catalog_items` (list dispatchable items)
    - Use `ItemBankCatalogTrait`
    - Return `Vec<ItemCatalogOption>`

12. **`ran-access-point/src/routes/admin/item_bank/dispatch.rs`**
    - `GET ""` → `get_dispatch_list` (list dispatches, paginated)
    - `GET "/{id}"` → `get_dispatch_by_id` (dispatch detail)
    - `POST ""` → `create_dispatch` (send items to players)
    - Use `ItemBankDispatchTrait` for dispatch ops and `ItemBankCatalogTrait` is NOT used here (catalog is separate)
    - Follow same handler patterns as current `items/dispatch.rs`

**Files to modify:**

13. **`ran-models/src/admin/mod.rs`** — Add `pub mod item_bank;`

14. **`ran-db-core/src/shopmanager/mod.rs`** — Add `pub mod item_bank;`

15. **`ran-access-point/src/routes/admin/mod.rs`** — Add:
    - `mod item_bank;`
    - `.service(actix_web::web::scope("/item-bank").configure(item_bank::configure))` inside the JWT-protected scope

**Files to update for backward compatibility:**

16. **`ran-models/src/admin/purchases.rs`** — Replace struct definition with re-export:
    - `pub use super::item_bank::entry::ItemBankEntry as ShopPurchaseBase;`
    - Keep the old import paths working

17. **`ran-models/src/admin/items/dispatch.rs`** — Replace struct definitions with re-exports from `admin::item_bank::dispatch::*`

18. **`ran-db-core/src/shopmanager/purchases.rs`** — Keep `PurchasesTrait` but have it delegate to or re-export from `item_bank::entries`

19. **`ran-db-core/src/shopmanager/items/dispatch.rs`** — Keep `ItemDispatchTrait` but have it delegate to or re-export from `item_bank::dispatch`

**Important patterns to follow:**
- Trait defined and implemented in the same file
- `impl Trait for ShopDB`
- `let mut client = self.pool.get().await.context("...")?;`
- Manual row mapping via `.try_get::<Type, _>("ColumnName")`
- Handlers use `web::Data<Arc<crate::DBState>>` (or `Arc<crate::database::DBState>`)
- Error responses: `HttpResponse::InternalServerError().json(json!({"error": "message"}))`
- Logging: `log::error!("...")` or `tracing::error!("...")`
- Pagination defaults: page=1, limit=10 (entries), page=0, limit=10 (dispatches)

After creating all files, run `cargo check --workspace` to verify compilation.

---

### Phase 3 — Debugger *(if needed)*

**Depends on:** CodeGen output, build errors

**Prompt:**

You are the Debugger for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`. The workspace has 3 crates: `ran-access-point`, `ran-models`, `ran-db-core`.

CodeGen just created a new `item_bank` module across all 3 crates as part of an endpoint consolidation refactoring. The new module lives at:
- `ran-models/src/admin/item_bank/` (entry.rs, catalog.rs, dispatch.rs, mod.rs)
- `ran-db-core/src/shopmanager/item_bank/` (entries.rs, catalog.rs, dispatch.rs, mod.rs)
- `ran-access-point/src/routes/admin/item_bank/` (entries.rs, catalog.rs, dispatch.rs, mod.rs)

Old modules were updated with backward-compatible re-exports:
- `ran-models/src/admin/purchases.rs` re-exports from `item_bank::entry`
- `ran-models/src/admin/items/dispatch.rs` re-exports from `item_bank::dispatch`
- `ran-db-core/src/shopmanager/purchases.rs` re-exports from `item_bank::entries`
- `ran-db-core/src/shopmanager/items/dispatch.rs` re-exports from `item_bank::dispatch`

Run `cargo check --workspace` and fix any errors. Common issues to watch for:
- Import path mismatches (old code referencing moved types)
- Missing `use` statements for `anyhow::Context`, `async_trait`, `tiberius::Row`, `uuid::Uuid`
- Trait method signature mismatches between definition and implementation
- Missing `from_row` implementations or wrong column name strings
- Re-export syntax errors (need `pub use` not just `use`)
- `actix_web` handler macro issues (wrong path patterns, missing extractors)

Run `cargo check -p ran-models` first, then `cargo check -p ran-db-core`, then `cargo check -p ran-access-point` to isolate errors by crate.

---

### Phase 4 — OpenAPI Validator

**Depends on:** Successful compilation

**Prompt:**

You are the OpenAPI Validator for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`.

New route handlers were created in `ran-access-point/src/routes/admin/item_bank/` as part of the Item Bank consolidation:
- `entries.rs` — 3 handlers: `get_entries`, `get_entry_by_id`, `get_user_entries`
- `catalog.rs` — 1 handler: `get_catalog_items`
- `dispatch.rs` — 3 handlers: `get_dispatch_list`, `get_dispatch_by_id`, `create_dispatch`

New models were created in `ran-models/src/admin/item_bank/`:
- `entry.rs` — `ItemBankEntry`
- `catalog.rs` — `ItemCatalogOption`
- `dispatch.rs` — `ItemDispatch`, `ItemDispatchList`, `ItemDispatchItem`, `ItemDispatchCharacter`, `ItemDispatchDetailRequestPayload`, `ItemDispatchDetailItem`, `ItemDispatchDetailCharacter`

Check the existing route handlers in `ran-access-point/src/routes/admin/purchases.rs` and `ran-access-point/src/routes/admin/items/dispatch.rs` for existing utoipa annotation patterns. Then validate that the new handlers and models have equivalent annotations.

Check for:
1. `#[utoipa::path(...)]` annotations on each handler
2. `#[derive(utoipa::ToSchema)]` on response/request models
3. `#[derive(utoipa::IntoParams)]` on query parameter structs
4. Registration in the OpenAPI doc builder (if one exists — check `ran-access-point/src/main.rs` or similar)

If the existing handlers DON'T have utoipa annotations, note that and skip — match the existing pattern.

---

### Phase 5 — Reviewer

**Depends on:** All prior phases complete

**Prompt:**

You are the Reviewer for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`. The workspace has 3 crates: `ran-access-point` (actix-web 4), `ran-models` (DTOs), `ran-db-core` (DB traits on ShopDB).

A refactoring was just completed to consolidate scattered Item Bank admin endpoints into a unified `/item-bank` module. Review ALL files that were created or modified:

**New files created:**
- `ran-models/src/admin/item_bank/mod.rs`
- `ran-models/src/admin/item_bank/entry.rs`
- `ran-models/src/admin/item_bank/catalog.rs`
- `ran-models/src/admin/item_bank/dispatch.rs`
- `ran-db-core/src/shopmanager/item_bank/mod.rs`
- `ran-db-core/src/shopmanager/item_bank/entries.rs`
- `ran-db-core/src/shopmanager/item_bank/catalog.rs`
- `ran-db-core/src/shopmanager/item_bank/dispatch.rs`
- `ran-access-point/src/routes/admin/item_bank/mod.rs`
- `ran-access-point/src/routes/admin/item_bank/entries.rs`
- `ran-access-point/src/routes/admin/item_bank/catalog.rs`
- `ran-access-point/src/routes/admin/item_bank/dispatch.rs`

**Modified files:**
- `ran-models/src/admin/mod.rs`
- `ran-models/src/admin/purchases.rs` (backward compat re-exports)
- `ran-models/src/admin/items/dispatch.rs` (backward compat re-exports)
- `ran-db-core/src/shopmanager/mod.rs`
- `ran-db-core/src/shopmanager/purchases.rs` (backward compat re-exports)
- `ran-db-core/src/shopmanager/items/dispatch.rs` (backward compat re-exports)
- `ran-access-point/src/routes/admin/mod.rs`

**Review focus areas:**

1. **Correctness**: Do the new trait methods call the exact same stored procedures with the same parameters? Are `from_row` mappings identical?
2. **Backward compatibility**: Do old import paths still resolve? Will existing code that uses `ShopPurchaseBase`, `PurchasesTrait`, `ItemDispatchTrait`, `ItemMapOptions` still compile?
3. **Security**: Are all new handlers inside the JWT-protected scope? Is the JWT Claims extraction correct in the dispatch POST handler?
4. **Pattern consistency**: Do new files follow the exact same patterns as existing codebase (trait+impl in same file, handler signatures, error responses, logging)?
5. **API contract**: Do new endpoints return the same JSON shapes as old ones? Are pagination defaults correct?
6. **Edge cases**: Route ordering in actix-web (e.g., `/entries/user/{user_num}` must be registered before `/entries/{id}` to avoid conflicts)

Run `cargo check --workspace` and `cargo clippy --workspace` to verify.

---

### Phase 6 — ContractGen

**Depends on:** Reviewer sign-off

**Prompt:**

You are the ContractGen agent for the gs-api Rust workspace at `/Users/schmitt/development/ran-online/gs/gs-api`.

Generate TypeScript types and API contracts for the new **Item Bank** admin module. This module consolidates three previously scattered endpoint groups into `/item-bank`.

**Source files to read:**

Models (glob `ran-models/src/admin/item_bank/`):
- `entry.rs` — `ItemBankEntry` struct
- `catalog.rs` — `ItemCatalogOption` struct
- `dispatch.rs` — `ItemDispatch`, `ItemDispatchList`, `ItemDispatchItem`, `ItemDispatchCharacter`, `ItemDispatchDetailRequestPayload`, `ItemDispatchDetailItem`, `ItemDispatchDetailCharacter`

Route handlers (glob `ran-access-point/src/routes/admin/item_bank/`):
- `entries.rs` — 3 GET endpoints
- `catalog.rs` — 1 GET endpoint
- `dispatch.rs` — 2 GET + 1 POST endpoint

There is NO user-facing route directory — this is admin-only.

**Notes for the agent:**
- `DateTime<Local>` fields serialize as ISO 8601 strings (e.g., `"2026-05-29T10:00:00+08:00"`)
- `Uuid` fields serialize as UUID strings
- Check for any `#[serde(rename = "...")]` or `#[serde(rename_all = "...")]` attributes
- The `ItemBankEntry` uses `purchase_id`, `user_num`, etc. as field names (no serde renames on the original `ShopPurchaseBase`)
- Pagination response shapes vary: entries list returns `{ data, page, limit }` or `{ data, total_items, page, limit }`; dispatches return `{ data, page, limit, total_items }`

Save the output to `workflows/item-bank-consolidation-contract.md`.

## Files Expected to Change

### New files (created)
- `ran-models/src/admin/item_bank/mod.rs`
- `ran-models/src/admin/item_bank/entry.rs`
- `ran-models/src/admin/item_bank/catalog.rs`
- `ran-models/src/admin/item_bank/dispatch.rs`
- `ran-db-core/src/shopmanager/item_bank/mod.rs`
- `ran-db-core/src/shopmanager/item_bank/entries.rs`
- `ran-db-core/src/shopmanager/item_bank/catalog.rs`
- `ran-db-core/src/shopmanager/item_bank/dispatch.rs`
- `ran-access-point/src/routes/admin/item_bank/mod.rs`
- `ran-access-point/src/routes/admin/item_bank/entries.rs`
- `ran-access-point/src/routes/admin/item_bank/catalog.rs`
- `ran-access-point/src/routes/admin/item_bank/dispatch.rs`

### Modified files
- `ran-models/src/admin/mod.rs`
- `ran-models/src/admin/purchases.rs`
- `ran-models/src/admin/items/dispatch.rs`
- `ran-db-core/src/shopmanager/mod.rs`
- `ran-db-core/src/shopmanager/purchases.rs`
- `ran-db-core/src/shopmanager/items/dispatch.rs`
- `ran-access-point/src/routes/admin/mod.rs`

## Risk Areas

- **Backward compatibility**: Old import paths (`ran_models::admin::purchases::ShopPurchaseBase`, `ran_db_core::shopmanager::purchases::PurchasesTrait`) must continue to resolve. The re-export strategy is critical.
- **Route ordering in actix-web**: `/entries/user/{user_num}` must be registered before `/entries/{id}` to avoid the path parameter `{id}` capturing "user" as a value.
- **JWT Claims extraction**: The dispatch POST handler extracts `claims.sub` from JWT extensions — must match the existing pattern exactly.
- **Pagination defaults**: Entries default page=1, dispatches default page=0. Must preserve these to match existing behavior.

## SQL Stored Procedures Needed

**None** — this is a pure Rust refactoring. All existing stored procedures are reused:
- `sp_GetShopPurchases` (entries list)
- `sp_GetShopPurchaseById` (entry detail)
- `sp_GetUserShopPurchases` (user entries)
- `sp_GetItemDispatchList` (dispatch list)
- `sp_GetDispatchItemById` (dispatch detail)
- `sp_DispatchItem` (create dispatch)
- Raw SQL on `ShopItemMap` (catalog)
