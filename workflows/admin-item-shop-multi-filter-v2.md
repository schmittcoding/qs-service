# Workflow: Admin Item Shop Multi-Value Filtering (Revised)

## Feature Summary

Upgrade the `GET /admin/v1/item-shop` endpoint to accept multiple values for `category`, `status`, and `purchase_limit` filters using the **established codebase pattern**: comma-separated strings (not JSON arrays). Follow the exact implementation of `sp_GetCharacters` and the `get_characters` endpoint, which already handles multi-value `@School` filtering this way.

## Agents Involved

1. **SQLGen** — Modify `sp_GetPaginatedShopData` to accept comma-separated string parameters and use a split function for filtering
2. **CodeGen** — Update `AdminItemShopFilterParams` DTO, update `ItemShopTrait::get_admin_shop_items` to convert `Vec<i32>` to comma-separated strings, update handler
3. **Reviewer** — Review all changes for correctness, security, and pattern consistency

## Implementation Workflow

### Phase 1 — SQLGen
**Depends on:** Architecture analysis

**Prompt:**

Modify the stored procedure `sp_GetPaginatedShopData` in ShopDB to support multi-value filtering using **comma-separated strings**, following the exact pattern from `sp_GetCharacters` in GameDB.

**Context:**
The gs-api uses comma-separated strings with split functions for multi-value filtering, not JSON arrays. The `sp_GetCharacters` procedure (in `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql`) shows the pattern:
- Accepts `@School NVARCHAR(MAX) = NULL` as a comma-separated string
- Uses `fn_SplitInts(@School, ',')` to parse it into an integer table
- Filters with: `(NOT EXISTS (SELECT 1 FROM @SchoolTable) OR CI.ChaSchool IN (SELECT SchoolID FROM @SchoolTable))`

**Current state:**
File: `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql`

Current parameters: `@Search`, `@Page`, `@PageSize`, plus JSON parameters `@CategoryJson`, `@StatusJson`, `@PurchaseLimitJson` (which should be replaced).

**Your task:**

1. **Check for a split function in ShopDB**:
   - Does ShopDB have a `fn_SplitInts` function? Search `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Functions/`
   - If it exists, use it. If not, copy it from `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/gamedb/Functions/fn_SplitInts.sql` and create it in ShopDB.

2. **Replace JSON parameters with comma-separated strings**:
   - Remove: `@CategoryJson`, `@StatusJson`, `@PurchaseLimitJson`
   - Add: `@Category NVARCHAR(MAX) = NULL`, `@Status NVARCHAR(MAX) = NULL`, `@PurchaseLimit NVARCHAR(MAX) = NULL`
   - Each contains a comma-separated list of integer IDs, e.g., "1,2,3"

3. **Rewrite the filter logic** to match the characters pattern:
   ```sql
   DECLARE @Categories TABLE (Value INT);
   DECLARE @Statuses TABLE (Value INT);
   DECLARE @PurchaseLimits TABLE (Value INT);

   IF @Category IS NOT NULL
       INSERT INTO @Categories (Value)
       SELECT Value FROM dbo.fn_SplitInts(@Category, ',');

   IF @Status IS NOT NULL
       INSERT INTO @Statuses (Value)
       SELECT Value FROM dbo.fn_SplitInts(@Status, ',');

   IF @PurchaseLimit IS NOT NULL
       INSERT INTO @PurchaseLimits (Value)
       SELECT Value FROM dbo.fn_SplitInts(@PurchaseLimit, ',');

   -- In the WHERE clause:
   (NOT EXISTS (SELECT 1 FROM @Categories) OR I.ItemCategory IN (SELECT Value FROM @Categories)) AND
   (NOT EXISTS (SELECT 1 FROM @Statuses) OR I.ItemStatus IN (SELECT Value FROM @Statuses)) AND
   (NOT EXISTS (SELECT 1 FROM @PurchaseLimits) OR
    I.PurchaseLimitPerAccount IN (SELECT Value FROM @PurchaseLimits WHERE Value > 0) OR
    (-1 IN (SELECT Value FROM @PurchaseLimits) AND I.PurchaseLimitPerAccount IS NULL))
   ```

4. **Keep the rest unchanged**: Search, discount calculation, pagination, etc.

**Output:**
- Updated `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql` with the new string-based parameters
- If ShopDB doesn't have `fn_SplitInts`, also create `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/shopdb/Functions/fn_SplitInts.sql` by copying from GameDB

---

### Phase 2 — CodeGen
**Depends on:** SQLGen output

**Prompt:**

Update the Rust models, database trait, and HTTP handler to pass comma-separated strings to the stored procedure, following the pattern from `get_characters`.

**Context:**
The modified `sp_GetPaginatedShopData` now accepts three comma-separated string parameters: `@Category`, `@Status`, `@PurchaseLimit`. The Rust code must convert `Vec<i32>` to comma-separated strings before passing them to the SP.

**Your task:**

1. **Verify `AdminItemShopFilterParams`** in `/Users/schmitt/development/ran-online/gs/gs-api/ran-models/src/admin/mod.rs`:
   - Should have `#[derive(Deserialize)]` only
   - Vector fields have `#[serde(default)]` for repeated query keys
   - Already in place; no changes needed

2. **Update `ItemShopTrait::get_admin_shop_items` in `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs`**:
   - Change signature to accept `Option<Vec<i32>>` for each filter (category_ids, status_values, purchase_limit_values)
   - Convert each `Option<Vec<i32>>` to a comma-separated string using the pattern from `get_characters`:
     ```rust
     let category_list = category_ids
         .as_ref()
         .map(|v| 
             v.iter()
             .map(|s| s.to_string())
             .collect::<Vec<_>>().join(",")
         );
     ```
   - Pass these three strings to the SP as `@Category = @P4, @Status = @P5, @PurchaseLimit = @P6`
   - SP query: `EXEC sp_GetPaginatedShopData @Search = @P1, @Page = @P2, @PageSize = @P3, @Category = @P4, @Status = @P5, @PurchaseLimit = @P6`

3. **Update the handler** in `/Users/schmitt/development/ran-online/gs/gs-api/ran-access-point/src/routes/admin/item_shop.rs`:
   - Extract and normalize empty vectors to `None` (same as before)
   - Pass the three `Option<Vec<i32>>` directly to the trait method
   - Handler uses `serde_qs` deserialization (already in place)

**Output:**
- Updated `/Users/schmitt/development/ran-online/gs/gs-api/ran-db-core/src/shopmanager/item_shop.rs` with string conversion logic
- Run `cargo check -p ran-models -p ran-db-core -p ran-access-point` and confirm no errors

---

### Phase 3 — Reviewer
**Depends on:** CodeGen output

**Prompt:**

Review the revised multi-value filtering implementation for correctness, security, and pattern consistency with the `get_characters` endpoint.

**Focus areas:**
- ✅ String conversion logic matches `get_characters` pattern (lines 117-123)
- ✅ SP parameter names match the new signature (`@Category`, `@Status`, `@PurchaseLimit`)
- ✅ `NOT EXISTS (SELECT 1 FROM @TempTable) OR IN` filter logic is correct
- ✅ `-1` sentinel for NULL purchase_limit is handled properly
- ✅ Error handling uses `.context()` for DB calls
- ✅ No SQL injection risk
- ✅ Handler follows `serde_qs` pattern from characters endpoint

**Output:**
- Summary of changes and verification status
- Any warnings or issues found
- Confirmation that the pattern now matches `get_characters` exactly

---

## Files to Modify

| File | Change |
| --- | --- |
| `db-scripts/shopdb/Procedures/sp_GetPaginatedShopData.sql` | Replace JSON params with comma-separated strings; use split function |
| `db-scripts/shopdb/Functions/fn_SplitInts.sql` | Create if it doesn't exist (copy from GameDB) |
| `ran-db-core/src/shopmanager/item_shop.rs` | Convert `Vec<i32>` to comma-separated strings in the trait impl |
| `ran-access-point/src/routes/admin/item_shop.rs` | Already correct (uses serde_qs); no changes needed |

## Key Difference from V1

- **V1 used JSON arrays** (`OPENJSON`) — caused "too many arguments" error
- **V2 uses comma-separated strings** — matches the established `get_characters` pattern with `fn_SplitInts`
- This aligns with how the codebase consistently handles multi-value filters across all endpoints
