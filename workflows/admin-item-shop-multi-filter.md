# Workflow: Admin Item Shop Multi-Value Filtering

## Feature Summary

Upgrade the `GET /admin/v1/item-shop` endpoint to accept multiple values for `category`, `status`, and `purchase_limit` filters. Today each filter accepts a single scalar (`Option<i16>`); the goal is to switch to `Option<Vec<i32>>` so the admin UI can show items matching any of several selected values (e.g., "show items in categories 1, 3, 5"). The stored procedure will be modified to accept JSON arrays via `OPENJSON` and apply `IN`-based filtering. A new `purchase_limit` filter is also being added (currently not present).

## Agents Involved

1. **SQLGen** — Modify `sp_GetPaginatedShopData` to accept JSON arrays and apply multi-value `IN` filtering
2. **CodeGen** — Add `AdminItemShopFilterParams` DTO, update `ItemShopTrait::get_admin_shop_items` signature and impl, update the handler
3. **OpenAPI Validator** — Ensure utoipa annotations correctly document the new multi-value query parameters and filter behavior
4. **Reviewer** — Review all changes for correctness, security, pattern consistency, and edge case handling

## Implementation Workflow

### Phase 1 — SQLGen
**Depends on:** Architecture analysis (completed)

**Prompt:**

Modify the stored procedure `sp_GetPaginatedShopData` in the ShopDB to support multi-value filtering via JSON arrays.

**Context:**
The gs-api is a Rust backend for RAN Online. All DB access goes through async traits calling stored procedures in 5 MSSQL databases. ShopDB holds the item shop data in `dbo.YXShopItemMap`. The endpoint currently supports single-value filters for `category` (ItemCategory SMALLINT) and `status` (ItemStatus SMALLINT). You are adding multi-value support and a new `purchase_limit` filter (PurchaseLimitPerAccount INT NULL).

**Reference pattern:**
The codebase already uses `OPENJSON` for JSON array transport in:
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_AddItemImages.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_DeleteItemImages.sql`

Follow this exact pattern: declare temp tables, parse JSON arrays with `OPENJSON`, use `IN` and `EXISTS` clauses to filter.

**Current state:**
File: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql`

Current signature:
```
@Search   NVARCHAR(100) = NULL,
@Category INT NULL = NULL,
@Status   INT NULL = NULL,
@Page     INT = 0,
@PageSize INT = 10
```

Current filter logic: scalar `=` per column. Returns `YXShopItemMap.*` plus computed `DiscountedPrice`, `LimitTypeText`, `TotalRecords`.

**Your task:**

1. Modify the stored procedure to accept three new JSON parameters:
   - `@CategoryJson NVARCHAR(MAX) = NULL` — JSON array of category IDs, e.g., `[1,2,3]`
   - `@StatusJson NVARCHAR(MAX) = NULL` — JSON array of status values, e.g., `[0,1]`
   - `@PurchaseLimitJson NVARCHAR(MAX) = NULL` — JSON array of purchase limit thresholds, e.g., `[5, 10, -1]` where `-1` is the sentinel for "unlimited (NULL)"

2. For each JSON parameter, declare a temp table and parse it with `OPENJSON`:
   ```sql
   DECLARE @Categories TABLE (Value INT PRIMARY KEY);
   IF @CategoryJson IS NOT NULL AND LEN(@CategoryJson) > 2
       INSERT INTO @Categories(Value)
       SELECT DISTINCT CAST([value] AS INT) FROM OPENJSON(@CategoryJson);
   ```

3. Implement the filter logic:
   - Filters use AND between columns (category AND status AND purchase_limit).
   - Within each column, use OR / `IN` semantics (match any of the provided values).
   - Use `NOT EXISTS (SELECT 1 FROM @Statuses) OR I.ItemStatus IN (SELECT Value FROM @Statuses)` pattern to handle "no filter applied" (empty table).
   - For `PurchaseLimitPerAccount` (nullable INT), support:
     - `-1` sentinel to mean "unlimited / NULL" → `(-1 IN (SELECT Value FROM @PurchaseLimits) AND I.PurchaseLimitPerAccount IS NULL)`
     - Numeric values → `I.PurchaseLimitPerAccount IN (SELECT Value FROM @PurchaseLimits)`
     - Combined: `I.PurchaseLimitPerAccount IN (SELECT Value FROM @PurchaseLimits) OR (-1 IN (SELECT Value FROM @PurchaseLimits) AND I.PurchaseLimitPerAccount IS NULL)`

4. Keep the existing `@Search` logic, `DiscountedPrice` computation, `LimitTypeText` case statement, `COUNT(*) OVER()` for pagination, and `OFFSET/FETCH` ordering.

5. Testing guidance (manual, for your own verification):
   - Test with empty arrays: `@CategoryJson = "[]"` should behave like `@CategoryJson = NULL` (no filter).
   - Test with single value: `@CategoryJson = "[1]"` should filter to category 1 items.
   - Test with multiple: `@CategoryJson = "[1,2,3]"` should return items in any of those categories.
   - Test with purchase_limit sentinel: `@PurchaseLimitJson = "[-1]"` should return only unlimited items (where PurchaseLimitPerAccount IS NULL).
   - Test combined: `@CategoryJson = "[1,2]"` AND `@StatusJson = "[1]"` AND `@PurchaseLimitJson = "[5, -1]"` should apply all three filters with AND logic.

**Output:**
Save the modified procedure to `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql`. Include:
- The new CREATE OR ALTER statement with the three JSON parameters.
- The temp table declarations and `OPENJSON` parsing logic.
- The updated WHERE clause with `NOT EXISTS` and `IN` patterns.
- A brief comment block explaining the -1 sentinel for purchase_limit NULL semantics.

---

### Phase 2 — CodeGen
**Depends on:** SQLGen output (modified SP signature)

**Prompt:**

Update the Rust models, database trait, and HTTP handler to support multi-value filtering for the admin item shop endpoint.

**Context:**
The gs-api has 3 crates: `ran-models` (DTOs, error types), `ran-db-core` (database traits + impls), `ran-access-point` (HTTP routes, handlers). The modified `sp_GetPaginatedShopData` now accepts three JSON parameters for multi-value filtering.

**Current state:**
- Handler: `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs` → `get_shop_items()` function, registered under `/item-shop` scope.
- DB trait: `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs` → `ItemShopTrait::get_admin_shop_items()` with single-value params.
- Query DTO: Currently uses shared `PaginationParams` from `ran-models/src/lib.rs` — **do NOT modify**; create a new admin-specific DTO instead.

**Your task:**

1. **Create `AdminItemShopFilterParams` struct** in `/Users/schmitt/development/ran-online/gs/gs-api/ran-models/src/item_shop.rs` (or create the file if it doesn't exist):
   ```rust
   #[derive(Deserialize, Serialize, serde::Deserialize)]
   pub struct AdminItemShopFilterParams {
       pub page: Option<i32>,
       pub limit: Option<i32>,
       pub search: Option<String>,
       
       #[serde(default)]
       pub category: Option<Vec<i32>>,
       #[serde(default)]
       pub status: Option<Vec<i32>>,
       #[serde(default)]
       pub purchase_limit: Option<Vec<i32>>,
   }
   ```
   - Use `i32` (not `i16`) to match the SP's `INT` parameters directly.
   - Include `#[serde(default)]` on vector fields to allow repeated query keys (`?category=1&category=2`) or missing params.
   - Do NOT derive `utoipa::IntoParams` — the handler will document params manually in the `#[utoipa::path(...)]` macro (following the established pattern in `characters/mod.rs`).

2. **Update `ItemShopTrait::get_admin_shop_items` signature** in `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs`:
   ```rust
   async fn get_admin_shop_items(
       &self,
       page: i32,
       limit: i32,
       search: Option<String>,
       category_ids: Option<Vec<i32>>,
       status_values: Option<Vec<i32>>,
       purchase_limit_values: Option<Vec<i32>>,
   ) -> Result<BaseDataCountResult>;
   ```

3. **Implement the trait method:**
   - Normalize each `Option<Vec<i32>>`: convert empty `Vec` to `None` so the SP sees NULL instead of `"[]"`.
   - For each non-None vector, serialize to JSON using `serde_json::to_string(&vec).context("Failed to serialize filters")?`. This produces strings like `"[1,2,3]"`.
   - Bind the three JSON strings to the SP call as `Option<String>` (MSSQL will receive them as `NVARCHAR(MAX)`).
   - Call the SP with the new parameter names: `EXEC sp_GetPaginatedShopData @Page = @P1, @PageSize = @P2, @Search = @P3, @CategoryJson = @P4, @StatusJson = @P5, @PurchaseLimitJson = @P6, ...`.
   - Row mapping: continue using the existing `.try_get()` pattern to extract `GetItemShopData` rows (no changes to the response shape).
   - Return `BaseDataCountResult` as before.

4. **Update the handler** in `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs`:
   - **Follow the established pattern** used in `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/characters/mod.rs`.
   - Import `serde_qs::Config` at the top of the file.
   - Update the function signature to accept `req: HttpRequest` parameter instead of `web::Query<AdminItemShopFilterParams>`:
     ```rust
     pub async fn get_shop_items(
         db: web::Data<Arc<DBState>>,
         req: HttpRequest,
     ) -> impl Responder
     ```
   - Deserialize query params manually:
     ```rust
     let qs = req.query_string();
     let config = Config::default();
     
     let params: AdminItemShopFilterParams = match config.deserialize_str(qs) {
         Ok(p) => p,
         Err(e) => return HttpResponse::BadRequest().body(format!("Invalid query: {}", e)),
     };
     ```
   - Extract query params: `let page = params.page.unwrap_or(0);` etc.
   - Normalize empty vectors to `None` as a safety check before calling the trait:
     ```rust
     let category = params.category.as_ref().filter(|v| !v.is_empty()).cloned();
     let status = params.status.as_ref().filter(|v| !v.is_empty()).cloned();
     let purchase_limit = params.purchase_limit.as_ref().filter(|v| !v.is_empty()).cloned();
     ```
   - Call `db.get_admin_shop_items(page, limit, search, category, status, purchase_limit).await?`.
   - Return the response unchanged (same shape as before).

5. **Register the new DTO** in `ran-models/src/lib.rs` if it's in a new submodule, e.g., `pub mod item_shop;` (check if already present first).

6. **Document the endpoint** in the `#[utoipa::path(...)]` macro above the handler. Follow the pattern in `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/characters/mod.rs`:
   ```rust
   #[utoipa::path(
       get,
       path = "/admin/v1/item-shop",
       operation_id = "admin_get_item_shop",
       tag = "Admin - Item Shop",
       security(("bearer_jwt" = [])),
       params(
           ("page" = Option<i32>, Query, description = "Page number"),
           ("limit" = Option<i32>, Query, description = "Items per page"),
           ("search" = Option<String>, Query, description = "Search by item name"),
           ("category" = Option<Vec<i32>>, Query, description = "Filter by category IDs"),
           ("status" = Option<Vec<i32>>, Query, description = "Filter by status values"),
           ("purchase_limit" = Option<Vec<i32>>, Query, description = "Filter by purchase limit thresholds")
       ),
       responses(
           (status = 200, description = "List of items with pagination"),
           (status = 400, description = "Invalid query parameters"),
           (status = 500, description = "Internal server error")
       )
   )]
   ```
   - Manually specify each query parameter in the `params(...)` section.
   - Multi-value params are documented as `Option<Vec<T>>`.

7. **Verify no other callers** of `ItemShopTrait::get_admin_shop_items` exist by running:
   ```bash
   cargo grep "get_admin_shop_items" -p ran-db-core -p ran-access-point
   ```
   If there are other callers outside the admin route, update them to use the new signature.

**Output:**
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-models/src/item_shop.rs` — new `AdminItemShopFilterParams` struct with proper derives.
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs` — updated `ItemShopTrait` signature and impl with JSON serialization and normalization.
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs` — updated handler using the new DTO, validation, and trait call.
- Run `cargo check -p ran-models -p ran-db-core -p ran-access-point` and confirm no errors.

---

### Phase 3 — OpenAPI Validator
**Depends on:** CodeGen output (new DTOs and handler annotations)

**Prompt:**

Validate that the updated admin item shop endpoint has correct utoipa OpenAPI documentation for the new multi-value filters.

**Context:**
The `GET /admin/v1/item-shop` endpoint was updated to accept multi-value query parameters for `category`, `status`, and `purchase_limit` (each `Option<Vec<i32>>`). The handler uses `AdminItemShopFilterParams` DTO with `#[derive(utoipa::IntoParams)]` to document the query schema.

**Your task:**

1. Verify the handler in `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs` has proper utoipa documentation:
   - The function should have `#[utoipa::path(...)]` macro with all query parameters manually documented in the `params(...)` section.
   - Check that each filter parameter is documented: `page`, `limit`, `search`, `category`, `status`, `purchase_limit`.
   - Multi-value params should be documented as `Option<Vec<i32>>` to indicate they accept repeated keys.
   - The `responses(...)` clause should cover 200 (success), 400 (bad request), and 500 (error).

2. Verify `AdminItemShopFilterParams` in `/Users/schmitt/development/ran-online/gs/gs-api/ran-models/src/item_shop.rs`:
   - Must have `#[derive(Deserialize, Serialize)]` to support `serde_qs` deserialization.
   - Vector fields should have `#[serde(default)]` to allow them to be omitted or repeated in query strings.
   - Optional fields are correctly marked as `Option<T>`.

3. Run `cargo check -p ran-access-point` and verify:
   - The utoipa `#[utoipa::path(...)]` macro expands without errors.
   - The handler is registered in the admin scope and will be included in the OpenAPI spec.
   - No compilation errors from the manual query parameter documentation.

4. If the endpoint is wired into the main OpenAPI registry (check `ran-access-point/src/main.rs` or wherever `utoipa::openapi()` is called), verify:
   - The route scope `/admin/v1/item-shop` and the handler are included in the `.paths()`.
   - The manual `params(...)` documentation in the macro is correctly reflected in the OpenAPI spec schema.

5. Flag any missing annotations or incomplete schemas. Common issues:
   - Response type not documented (404, 400, 500 responses missing).
   - Query params missing doc comments.
   - Array schema not correctly rendered in OpenAPI (check that `Option<Vec<i32>>` renders as an optional array, not a string).

**Output:**
Report (as text in the console or a flagged code comment):
- ✅ Handler has utoipa macro with correct path and query/response annotations.
- ✅ `AdminItemShopFilterParams` derives `IntoParams` and `ToSchema`.
- ✅ OpenAPI spec includes the new multi-value filters as array parameters.
- ⚠️ Any missing or incomplete annotations (if found).

If issues are found, make minimal fixes inline (e.g., add `#[utoipa::path(...)]` if missing, add doc comments).

---

### Phase 4 — Reviewer
**Depends on:** All prior phases complete (SP modified, code written, OpenAPI validated)

**Prompt:**

Review the multi-value filtering feature for correctness, security, pattern consistency, and edge case handling.

**Context:**
The `GET /admin/v1/item-shop` endpoint was updated to support multi-value query parameters for filtering by category, status, and purchase_limit. The feature involves:
- Modified SQL stored procedure with `OPENJSON` JSON array parsing.
- New `AdminItemShopFilterParams` DTO in `ran-models/src/item_shop.rs`.
- Updated `ItemShopTrait::get_admin_shop_items` in `ran-db-core/src/shopmanager/item_shop.rs` with JSON serialization.
- Updated handler in `ran-access-point/src/routes/admin/item_shop.rs` using the new DTO.

**Files to review:**
- `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-models/src/item_shop.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs`
- `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs`

**Review checklist:**

**SQL correctness:**
- [ ] The temp table declarations and `OPENJSON` parsing are syntactically correct (no typos in table names, column names, data types).
- [ ] The `NOT EXISTS ... OR IN ...` filter logic correctly applies AND between columns and OR within each column.
- [ ] The `-1` sentinel for NULL `PurchaseLimitPerAccount` is properly handled: `(-1 IN (...) AND IS NULL) OR IN (...)`.
- [ ] Edge cases: empty arrays (`"[]"`), NULL parameters, duplicate values in arrays (deduped by `DISTINCT`).
- [ ] The `OFFSET/FETCH` pagination still works correctly with the new CTE.
- [ ] No SQL injection risk: `OPENJSON` is parameterized; user input flows only through JSON strings, not string concatenation.

**Rust correctness:**
- [ ] `AdminItemShopFilterParams` has correct Serde derives (`Deserialize`, `Serialize`) only — no utoipa derives needed.
- [ ] `Option<Vec<i32>>` fields have `#[serde(default)]` to allow repeated query keys (`?category=1&category=2`) or missing params.
- [ ] Handler uses `req.query_string()` + `serde_qs::Config` to deserialize params (matching established pattern from `characters/mod.rs`).
- [ ] Deserialization error handling: bad query strings return `HttpResponse::BadRequest()` with a clear message.
- [ ] Empty vector normalization: code correctly converts `Some(vec![])` to `None` before calling the trait.
- [ ] JSON serialization: `serde_json::to_string(&vec)` is correctly used and error-handled with `.context()`.
- [ ] The trait method signature matches the handler call site (parameter names, types, order).
- [ ] The handler correctly extracts and defaults `page` and `limit` (e.g., `params.page.unwrap_or(0)`).
- [ ] No unwrap/panic/expect in the feature code (only `?` or explicit error handling).

**Pattern consistency:**
- [ ] The new DTO follows the existing naming convention (`AdminXxxParams` or similar).
- [ ] The trait method naming is consistent (`get_admin_shop_items` matches pattern).
- [ ] Row mapping in the trait impl continues to use `.try_get()` with `anyhow::Context` as elsewhere.
- [ ] Handler error responses match the existing format (`json!({"status": "error", "message": "..."})`).
- [ ] No new dependencies added (serde_json, actix-web, validator already present).

**Security & validation:**
- [ ] Input validation on list length: if `validator` rules are used, `#[validate(length(max = 50))]` prevents huge payloads.
- [ ] Null/None handling: filters with empty vectors are treated as "no filter," not "match nothing."
- [ ] SQL injection: no risk (all user input via parameterized JSON).
- [ ] Authorization: handler is under `admin_route()` scope, so JWT auth is enforced by middleware (no new auth logic needed).
- [ ] Error messages: user-facing errors are safe (no leakage of internal schema details).

**Edge cases:**
- [ ] Missing query params: defaults apply (page=0, limit=10 or similar), filters off.
- [ ] Non-numeric values in query: Serde deserialization error → 400 Bad Request (automatic, no handler work).
- [ ] Duplicate values in a filter list: SQL `DISTINCT` dedupes; harmless.
- [ ] Very large filter lists (>1000 values): validate with `length(max = 50)` to prevent DOS.
- [ ] Concurrent access: stored procedure uses `NOLOCK` hints; no concurrency issues.
- [ ] NULL values in the purchase_limit list: JSON parser treats JSON `null` as a JSON value, not a numeric `null` (good — avoids confusion).

**Testing observations:**
- [ ] Handler should be tested with:
   - No filters: `GET /admin/v1/item-shop`
   - Single filter: `GET /admin/v1/item-shop?category=1`
   - Multiple same filter: `GET /admin/v1/item-shop?category=1&category=2`
   - Multiple different filters: `GET /admin/v1/item-shop?category=1&status=0&purchase_limit=5`
   - Empty filter (e.g., `?category=`): should be treated as no filter, not error.
   - Invalid input (e.g., `?category=abc`): 400 Bad Request.
   - Over limit (e.g., 51 category values): 400 or 500 depending on implementation.

**Output:**
Provide a summary:
- ✅ Code follows established codebase patterns (serde_qs deserialization, manual utoipa path docs, multi-value filter handling).
- ✅ Code is correct, secure, and consistent with patterns.
- ⚠️ Minor issues found: [list and fix].
- ❌ Major issues found: [block merge and describe].

Run `cargo check`, `cargo clippy` on all three crates to confirm no warnings introduced. Verify that the handler pattern exactly mirrors `characters/mod.rs` in structure and style.

---

## Files Expected to Change

| File | Change |
| --- | --- |
| `db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql` | Modified: new signature with `@CategoryJson`, `@StatusJson`, `@PurchaseLimitJson` parameters; new `OPENJSON` parsing and `IN`-based filtering logic. |
| `ran-models/src/item_shop.rs` | New or modified: `AdminItemShopFilterParams` struct with `page`, `limit`, `search`, `category`, `status`, `purchase_limit` fields. |
| `ran-db-core/src/shopmanager/item_shop.rs` | Modified: `ItemShopTrait::get_admin_shop_items` signature updated to accept `Option<Vec<i32>>` for filters; impl serializes vecs to JSON and calls updated SP. |
| `ran-access-point/src/routes/admin/item_shop.rs` | Modified: handler `get_shop_items` updated to use `AdminItemShopFilterParams`, normalize empties to `None`, validate, pass to trait. |

## Risk Areas

- **Database performance**: The table `YXShopItemMap` currently has only a `PRIMARY KEY (ProductNum)` index. The new `IN` filters on `ItemCategory`, `ItemStatus`, `PurchaseLimitPerAccount` will scan without additional indexing. Consider creating a composite index: `CREATE INDEX IX_YXShopItemMap_Filter ON dbo.YXShopItemMap (ItemStatus, ItemCategory, PurchaseLimitPerAccount) INCLUDE (ItemName, ItemPrice, ProductNum)`. This is not blocking but recommended for larger datasets.
- **NULL semantics for `purchase_limit`**: The sentinel value `-1` is used to represent "unlimited (NULL)". Ensure product/admin team approves this convention. If they want a different approach (e.g., separate "unlimited" checkbox), the filter logic will need adjustment.
- **Empty arrays**: The code normalizes `Some(vec![])` to `None` to avoid filtering to zero results. Confirm this matches admin UX expectations (unchecked checkboxes mean "show all").
- **Concurrent filter updates**: If the admin UI submits requests while the list of categories/statuses is changing in the DB, the SP will still work (it filters against current state). No lock/consistency issue.
- **Breaking change**: The `ItemShopTrait::get_admin_shop_items` signature changed. Verify no other code paths call this method outside the admin route. Run `cargo grep "get_admin_shop_items"` to confirm.

## SQL Stored Procedures Needed

| Procedure | Database | Input Parameters | Output Columns |
| --- | --- | --- | --- |
| `sp_GetPaginatedShopData` | ShopDB | `@Page INT`, `@PageSize INT`, `@Search NVARCHAR(100)`, `@CategoryJson NVARCHAR(MAX)`, `@StatusJson NVARCHAR(MAX)`, `@PurchaseLimitJson NVARCHAR(MAX)` | All columns from `YXShopItemMap` plus computed: `DiscountedPrice DECIMAL(18,2)`, `LimitTypeText NVARCHAR(20)`, `TotalRecords INT` |
