# Workflow: Shop Item Game Sync (Stored Procedure Only)

## Feature Summary

The `POST add_shop_item` and `PUT edit_shop_item` admin endpoints currently use inline SQL to create/update items only in `YXShopItemMap` (web-facing shop inventory). The game server reads from the `ShopItemMap` table, which must be manually seeded — creating a disconnect between web and game. 

This workflow consolidates the logic entirely into two stored procedures: `sp_AddShopItem` and `sp_EditShopItem`. Each procedure:
1. **Checks** if the item (by `ItemMain` + `ItemSub`) exists in `ShopItemMap`
2. **Inserts** it if missing (with required fields: `ItemMain`, `ItemSub`, `ItemName`, `ItemPrice`)
3. **Creates/updates** the item in `YXShopItemMap` 
4. **Returns** the inserted/updated row

All within a single atomic transaction. The Rust handlers remain largely unchanged — they just call the procedure instead of inline SQL.

## Agents Involved

1. **Architect** — Analyze current inline SQL, schema design, transaction flow, edge cases.
2. **SQLGen** — Create/modify `sp_AddShopItem` and `sp_EditShopItem` procedures with built-in game map sync.
3. **CodeGen** — Update Rust trait methods to call the new procedures instead of inline SQL.
4. **Reviewer** — Review SQL for correctness, transaction safety, and idempotency; review Rust changes.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are analyzing a feature to consolidate shop item creation/editing logic into stored procedures, with built-in game server synchronization.

**Current State:**
- Route handlers: `ran-access-point/src/routes/admin/item_shop.rs`
  - `add_shop_item` (POST /admin/v1/item-shop) — line 222
  - `edit_shop_item` (PUT /admin/v1/item-shop/{id}) — line 173
- DB trait & implementation: `ran-db-core/src/shopmanager/item_shop.rs`
  - `add_shop_item()` method (lines 212–266) — uses inline INSERT SQL
  - `edit_shop_item()` method (lines 155–210) — uses inline UPDATE SQL
- Models: `ran-models/src/item_shop.rs::ItemShopBase` contains required fields: `item_main`, `item_sub`, `item_name`, `item_price`
- Target database: ShopDB (`RanShop` in MSSQL)

**Problem:**
- Current implementation only touches `YXShopItemMap` (web shop)
- Game server reads from `ShopItemMap` (requires manual seeding)
- Need to sync on every add/edit operation

**Solution Requirements:**
1. Create `sp_AddShopItem` procedure that:
   - Takes all fields from `ItemShopBase` plus image URL
   - Checks if `(ItemMain, ItemSub)` exists in `ShopItemMap`; if not, inserts
   - Inserts into `YXShopItemMap` with all fields
   - Returns the inserted row (for `ItemShopData` mapping)
   - Uses a transaction for atomicity

2. Create `sp_EditShopItem` procedure that:
   - Takes ProductNum and all fields from `ItemShopBase` plus image URL
   - Checks if `(ItemMain, ItemSub)` exists in `ShopItemMap`; if not, inserts
   - Updates `YXShopItemMap` with all fields
   - Returns the updated row (for `ItemShopData` mapping)
   - Uses a transaction for atomicity

**Your Task:**
1. Review the current inline SQL in `ran-db-core/src/shopmanager/item_shop.rs`:
   - `add_shop_item()` method (lines 219–238): Extract the INSERT query; identify all parameters
   - `edit_shop_item()` method (lines 162–181): Extract the UPDATE query; identify all parameters
2. Map parameters to `ItemShopBase` fields (lines 88–116 in `ran-models/src/item_shop.rs`)
3. Check `db-scripts/shopdb/Procedures/` for existing patterns (naming, error handling, transaction structure)
4. Examine `ShopItemMap` table structure via existing procedures like `sp_DispatchItem.sql`, `sp_GetItemDispatchItemOptions.sql`
5. Design the procedures:
   - Procedure signature: List all input parameters with types
   - Step 1: Check/insert into `ShopItemMap` (wrapped in TRY-CATCH)
   - Step 2: INSERT/UPDATE `YXShopItemMap` (wrapped in BEGIN TRANSACTION...COMMIT)
   - Step 3: OUTPUT inserted/updated row
   - Error handling: ROLLBACK on failure, RAISERROR with clear message
6. Identify edge cases:
   - What if `ItemMain` or `ItemSub` changes during an edit? Should we still check/insert the new combination?
   - What if the `YXShopItemMap` insert/update fails? Should the entire procedure fail?
   - Are there unique constraints on `ShopItemMap`? How to avoid duplicates?
7. Produce a summary:
   - Both procedure signatures (with all parameters)
   - Parameter mapping from `ItemShopBase` fields
   - Transaction flow diagram (text-based)
   - Edge case resolutions

**Output Format:**
Create a file `workflows/shop-item-sync-analysis.md` with the above details.

---

### Phase 2 — SQLGen

**Depends on:** Architect output

**Prompt:**

Create two stored procedures in `db-scripts/shopdb/Procedures/` to handle shop item creation/editing with built-in game map synchronization.

**Architecture Summary:** [Include summary from Architect phase]

**Your Task:**

1. **Create `sp_AddShopItem` procedure:**
   - Location: `db-scripts/shopdb/Procedures/sp_AddShopItem.sql`
   - Parameters: All fields from the current inline INSERT (from Architect output)
     - `@ItemMain INT`
     - `@ItemSub INT`
     - `@ItemName NVARCHAR(100)`
     - `@ItemPrice FLOAT`
     - `@ItemCategory INT`
     - `@ItemDescription NVARCHAR(500)`
     - `@ItemDiscount FLOAT = NULL`
     - `@ItemDiscountType SMALLINT = NULL`
     - `@NewItemImage NVARCHAR(MAX) = ''`
     - `@ItemStatus SMALLINT`
     - `@ItemStock INT`
     - `@ItemTag NVARCHAR(MAX) = ''`
     - `@PurchaseLimitPerAccount INT = NULL`
     - `@PurchaseLimitType SMALLINT = NULL`
   - Logic:
     ```
     BEGIN TRANSACTION
       IF NOT EXISTS (ItemMain, ItemSub in ShopItemMap):
         INSERT ShopItemMap with (ItemMain, ItemSub, ItemName, ItemPrice)
       INSERT YXShopItemMap with all fields
       OUTPUT inserted.*
     COMMIT
     ```
   - Error handling: TRY-CATCH, ROLLBACK on failure, RAISERROR with meaningful message

2. **Create `sp_EditShopItem` procedure:**
   - Location: `db-scripts/shopdb/Procedures/sp_EditShopItem.sql`
   - Parameters: Same as `sp_AddShopItem`, plus `@ProductNum INT` (the row to update)
   - Logic:
     ```
     BEGIN TRANSACTION
       IF NOT EXISTS (ItemMain, ItemSub in ShopItemMap):
         INSERT ShopItemMap with (ItemMain, ItemSub, ItemName, ItemPrice)
       UPDATE YXShopItemMap with all fields WHERE ProductNum = @ProductNum
       OUTPUT inserted.*
     COMMIT
     ```
   - Error handling: TRY-CATCH, ROLLBACK on failure, RAISERROR with meaningful message

3. **Follow existing SQL patterns:**
   - Use `SET ANSI_NULLS ON` and `SET QUOTED_IDENTIFIER ON`
   - Use `SET NOCOUNT ON` at the start
   - Include a comment block describing purpose, parameters, and returns
   - Use parameterized queries (all `@P` parameters)
   - Handle NULL values gracefully (e.g., `@ItemDiscount IS NULL`)

4. **Test notes:**
   - Both procedures should return exactly the same row structure as the old inline SQL (for Rust `ItemShopData::from_row()` to work)
   - `ShopItemMap` insert should be idempotent: if the item already exists (same ItemMain & ItemSub), skip without error
   - `YXShopItemMap` insert/update should always succeed (no duplicate constraints there)

5. **Output Summary:**
   | Procedure | Database | Parameters | Returns | Purpose |
   | --- | --- | --- | --- | --- |
   | `sp_AddShopItem` | ShopDB | ItemMain, ItemSub, ItemName, ..., PurchaseLimitType | Row (same as YXShopItemMap) | Insert item to both YXShopItemMap and ShopItemMap atomically |
   | `sp_EditShopItem` | ShopDB | ProductNum, ItemMain, ItemSub, ItemName, ..., PurchaseLimitType | Row (same as YXShopItemMap) | Update item in YXShopItemMap; sync to ShopItemMap if (ItemMain, ItemSub) new |

---

### Phase 3 — CodeGen

**Depends on:** Architect output, SQLGen output (procedure names and signatures)

**Prompt:**

Update the Rust code to call the new stored procedures instead of inline SQL.

**Current Code:**
- Trait definition: `ran-db-core/src/shopmanager/item_shop.rs` (lines 8–30)
- `add_shop_item()` implementation: lines 212–266
- `edit_shop_item()` implementation: lines 155–210
- Handlers: `ran-access-point/src/routes/admin/item_shop.rs` (no changes needed to handlers)

**Your Task:**

1. **Update `add_shop_item()` method** in `ran-db-core/src/shopmanager/item_shop.rs`:
   - Replace the inline INSERT SQL with a call to `sp_AddShopItem`
   - Query: `client.query("EXEC sp_AddShopItem @P1 = @P1, @P2 = @P2, ...", &[&item_data.item_main, &item_data.item_sub, ...])`
   - Extract parameters from `item_data: ItemShopBase` and `image_url: String` in the same order as the procedure signature
   - `.into_row().await?` to get the result
   - Map the row to `ItemShopData::from_row(&row)` (no changes to the model needed)
   - Return `Ok(ItemShopData)`

2. **Update `edit_shop_item()` method** in `ran-db-core/src/shopmanager/item_shop.rs`:
   - Replace the inline UPDATE SQL with a call to `sp_EditShopItem`
   - Query: `client.query("EXEC sp_EditShopItem @P1 = @P1, @P2 = @P2, ...", &[&product_num, &item_data.item_main, ...])`
   - Include `product_num` as the first parameter
   - Extract remaining parameters from `item_data: ItemShopBase` and `image_url: String`
   - `.into_row().await?` to get the result
   - Map the row to `ItemShopData::from_row(&row)`
   - Return `Ok(ItemShopData)`

3. **No handler changes needed:**
   - `add_shop_item` handler (line 222 in `ran-access-point/src/routes/admin/item_shop.rs`) remains unchanged
   - `edit_shop_item` handler (line 173) remains unchanged
   - Both will automatically benefit from the new procedures

4. **Compile check:**
   ```bash
   cargo check -p ran-db-core
   cargo check -p ran-access-point
   ```

---

### Phase 4 — Reviewer

**Depends on:** All prior phases complete

**Prompt:**

Review SQL and Rust changes for correctness, atomicity, pattern consistency, and error handling.

**Files to Review:**
1. `db-scripts/shopdb/Procedures/sp_AddShopItem.sql` (new)
2. `db-scripts/shopdb/Procedures/sp_EditShopItem.sql` (new)
3. `ran-db-core/src/shopmanager/item_shop.rs` (modified `add_shop_item()` and `edit_shop_item()` methods)

**SQL Checklist:**
- [ ] Both procedures check `(ItemMain, ItemSub)` in `ShopItemMap` before inserting?
- [ ] Check-insert is idempotent (no error if item already exists)?
- [ ] `ShopItemMap` insert includes: `ItemMain`, `ItemSub`, `ItemName`, `ItemPrice`?
- [ ] `YXShopItemMap` insert/update includes all required fields from `ItemShopBase`?
- [ ] OUTPUT clause returns the correct row format (matches `ItemShopData::from_row()`)?
- [ ] Transaction boundaries: BEGIN TRANSACTION → check/insert → insert/update → COMMIT?
- [ ] Error handling: TRY-CATCH wraps the transaction; ROLLBACK on error; RAISERROR with message?
- [ ] NULL handling: Optional fields (`@ItemDiscount`, `@ItemDiscountType`, etc.) accept NULL?
- [ ] Parameter naming consistent: all `@ParameterName` format?
- [ ] Follows existing SQL patterns in `db-scripts/shopdb/Procedures/`?

**Rust Checklist:**
- [ ] `add_shop_item()` calls `sp_AddShopItem` with correct parameter order?
- [ ] `edit_shop_item()` calls `sp_EditShopItem` with correct parameter order (ProductNum first)?
- [ ] Parameters extracted from `item_data: ItemShopBase` and `image_url: String` correctly?
- [ ] `.into_row().await?` used to fetch result?
- [ ] `ItemShopData::from_row(&row)` maps the result correctly?
- [ ] `.context()` used for all error messages?
- [ ] No `.unwrap()` calls?
- [ ] Trait signature unchanged (method name, parameter types, return type)?

**Commands to Run:**
```bash
cargo check -p ran-db-core -q
cargo check -p ran-access-point -q
cargo clippy --all-targets -q
```

**Output:**
If all checks pass, produce a sign-off summary:
- Files reviewed: [list]
- Issues found: [none, or describe]
- Pattern adherence: ✓
- Atomicity verified: ✓ (both operations in single transaction)
- Ready to merge: [yes/no]

---

## Files Expected to Change

| File | Crate | Type | Change |
| --- | --- | --- | --- |
| `db-scripts/shopdb/Procedures/sp_AddShopItem.sql` | N/A | NEW | Stored procedure for atomic add with game map sync |
| `db-scripts/shopdb/Procedures/sp_EditShopItem.sql` | N/A | NEW | Stored procedure for atomic edit with game map sync |
| `ran-db-core/src/shopmanager/item_shop.rs` | ran-db-core | MODIFIED | Replace inline SQL with procedure calls in `add_shop_item()` and `edit_shop_item()` |

## Risk Areas

- **Transaction atomicity:** Both operations (ShopItemMap sync + YXShopItemMap insert/update) happen together. If either fails, both rollback. This is safer than the best-effort approach.
- **ItemMain/ItemSub changes on edit:** If an admin edits an item and changes ItemMain or ItemSub to a new combination, the old ShopItemMap entry is left behind. Mitigation: ShopItemMap is read-only by game; old entries are harmless. A future cleanup task could remove orphaned entries.
- **No ProductNum link in ShopItemMap:** ShopItemMap may not have a ProductNum column. Verify this won't cause issues when the game reads the data.

## Stored Procedures Needed

| Procedure | Database | Key Parameters | Returns | Purpose |
| --- | --- | --- | --- | --- |
| `sp_AddShopItem` | ShopDB | ItemMain, ItemSub, ItemName, ItemPrice, ItemCategory, ... | Row (all YXShopItemMap columns) | Atomically insert into ShopItemMap (if new) and YXShopItemMap |
| `sp_EditShopItem` | ShopDB | ProductNum, ItemMain, ItemSub, ItemName, ItemPrice, ... | Row (all YXShopItemMap columns) | Atomically sync to ShopItemMap (if new) and update YXShopItemMap |

## Benefits of Stored Procedure-Only Approach

✅ **Single atomic transaction** — Both operations succeed or both rollback together
✅ **Simpler Rust code** — No extra sync calls; just replace inline SQL with procedure calls
✅ **Better consistency** — Game map is always in sync at the moment the web item is committed
✅ **Idempotent design** — Duplicate `(ItemMain, ItemSub)` in `ShopItemMap` is safely skipped
✅ **Single point of control** — All logic for both operations lives in SQL, easier to audit/modify
