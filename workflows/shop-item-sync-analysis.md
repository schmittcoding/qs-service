# Shop Item Sync — Design Analysis

## Section 1: Current Schema Summary

### `RanShop.dbo.YXShopItemMap` (web/admin-managed table)

Source: `db-scripts/shopdb/Tables/YXShopItemMap.sql`

| Column                    | Type             | Nullable | Notes                              |
| ------------------------- | ---------------- | -------- | ---------------------------------- |
| `ProductNum`              | `INT IDENTITY`   | NO       | PK, auto-increment                 |
| `ItemMain`                | `INT`            | NO       | Game item main id                  |
| `ItemSub`                 | `INT`            | NO       | Game item sub id                   |
| `ItemName`                | `NVARCHAR(255)`  | NO       |                                    |
| `ItemPrice`               | `MONEY`          | NO       | Maps to `f64` in Rust              |
| `ItemCategory`            | `SMALLINT`       | NO       |                                    |
| `ItemDescription`         | `NVARCHAR(MAX)`  | YES      |                                    |
| `ItemDiscount`            | `FLOAT`          | YES      |                                    |
| `ItemDiscountType`        | `SMALLINT`       | YES      | 1=Percentage, 2=Fixed              |
| `ItemImage`               | `VARBINARY(MAX)` | YES      | Legacy, unused                     |
| `ItemStatus`              | `SMALLINT`       | YES      |                                    |
| `ItemStock`               | `INT`            | NO       |                                    |
| `PurchaseLimitPerAccount` | `INT`            | YES      |                                    |
| `NewItemImage`            | `NVARCHAR(MAX)`  | YES      | Current image URL                  |
| `ItemTag`                 | `NVARCHAR(15)`   | YES      |                                    |
| `PurchaseLimitType`       | `SMALLINT`       | YES      | 0/1/2/3                            |

Constraint: `PK_YXShopItemMap` on `ProductNum`.

### `RanShop.dbo.ShopItemMap` (game-side / dispatcher table)

Game-managed table. Confirmed columns from existing procedures:

| Column       | Type (inferred) | Notes                              |
| ------------ | --------------- | ---------------------------------- |
| `ProductNum` | `INT`           | Used in sp_DispatchItem line 96    |
| `ItemMain`   | `INT`           | Matched by dispatchers             |
| `ItemSub`    | `INT`           | Matched by dispatchers             |
| `ItemName`   | `NVARCHAR(...)` | Stored for dispatch logs           |

**Important:** No FK linking YXShopItemMap to ShopItemMap. Relationship is logical, by matching `(ItemMain, ItemSub)`. The two tables are independent; no declared unique constraint on `(ItemMain, ItemSub)` is visible in existing SQL.

---

## Section 2: Parameter Mapping

### `ItemShopBase` → SQL parameter → `YXShopItemMap` column

| Rust field (`ItemShopBase`)                  | Rust type       | SQL Param                  | SQL Type        | Column                    |
| -------------------------------------------- | --------------- | -------------------------- | --------------- | ------------------------- |
| `item_main`                                  | `i32`           | `@ItemMain`                | `INT`           | `ItemMain`                |
| `item_sub`                                   | `i32`           | `@ItemSub`                 | `INT`           | `ItemSub`                 |
| `item_name`                                  | `String`        | `@ItemName`                | `NVARCHAR(255)` | `ItemName`                |
| `item_price`                                 | `f64`           | `@ItemPrice`               | `FLOAT`         | `ItemPrice`               |
| `item_category`                              | `i16`           | `@ItemCategory`            | `SMALLINT`      | `ItemCategory`            |
| `item_description`                           | `String`        | `@ItemDescription`         | `NVARCHAR(MAX)` | `ItemDescription`         |
| `item_discount`                              | `Option<f64>`   | `@ItemDiscount`            | `FLOAT`         | `ItemDiscount`            |
| `item_discount_type`                         | `Option<i16>`   | `@ItemDiscountType`        | `SMALLINT`      | `ItemDiscountType`        |
| (handler arg) `image_url`                    | `String`        | `@NewItemImage`            | `NVARCHAR(MAX)` | `NewItemImage`            |
| `item_status`                                | `i16`           | `@ItemStatus`              | `SMALLINT`      | `ItemStatus`              |
| `item_stock`                                 | `i32`           | `@ItemStock`               | `INT`           | `ItemStock`               |
| `item_tag`                                   | `String`        | `@ItemTag`                 | `NVARCHAR(15)`  | `ItemTag`                 |
| `item_purchase_limit`                        | `Option<i32>`   | `@PurchaseLimitPerAccount` | `INT`           | `PurchaseLimitPerAccount` |
| `item_purchase_limit_type`                   | `Option<i16>`   | `@PurchaseLimitType`       | `SMALLINT`      | `PurchaseLimitType`       |
| (edit only) `product_num`                    | `i32`           | `@ProductNum`              | `INT`           | `ProductNum` (WHERE)      |

**Type note:** Use `INT` (not `SMALLINT`) for `@ItemMain` and `@ItemSub` — matches Rust `i32`. Use `FLOAT` for `@ItemPrice` even though the table column is `MONEY` — Tiberius marshals `f64` to `FLOAT`, and SQL Server will convert on insert.

---

## Section 3: Procedure Signatures

### `sp_AddShopItem`

```sql
CREATE OR ALTER PROCEDURE [dbo].[sp_AddShopItem]
    @ItemMain                INT,
    @ItemSub                 INT,
    @ItemName                NVARCHAR(255),
    @ItemPrice               FLOAT,
    @ItemCategory            SMALLINT,
    @ItemDescription         NVARCHAR(MAX) = NULL,
    @ItemDiscount            FLOAT         = NULL,
    @ItemDiscountType        SMALLINT      = NULL,
    @NewItemImage            NVARCHAR(MAX) = N'',
    @ItemStatus              SMALLINT,
    @ItemStock               INT,
    @ItemTag                 NVARCHAR(15)  = NULL,
    @PurchaseLimitPerAccount INT           = NULL,
    @PurchaseLimitType       SMALLINT      = NULL
AS
BEGIN
    SET NOCOUNT ON;
    -- Body in Section 4
END
```

### `sp_EditShopItem`

```sql
CREATE OR ALTER PROCEDURE [dbo].[sp_EditShopItem]
    @ProductNum              INT,
    @ItemMain                INT,
    @ItemSub                 INT,
    @ItemName                NVARCHAR(255),
    @ItemPrice               FLOAT,
    @ItemCategory            SMALLINT,
    @ItemDescription         NVARCHAR(MAX) = NULL,
    @ItemDiscount            FLOAT         = NULL,
    @ItemDiscountType        SMALLINT      = NULL,
    @NewItemImage            NVARCHAR(MAX) = N'',
    @ItemStatus              SMALLINT,
    @ItemStock               INT,
    @ItemTag                 NVARCHAR(15)  = NULL,
    @PurchaseLimitPerAccount INT           = NULL,
    @PurchaseLimitType       SMALLINT      = NULL
AS
BEGIN
    SET NOCOUNT ON;
    -- Body in Section 4
END
```

---

## Section 4: Transaction & Error Handling Design

### Flow (sp_AddShopItem)

```
SET NOCOUNT ON
DECLARE @InsertedItems TABLE (all YXShopItemMap columns)

BEGIN TRY
    BEGIN TRANSACTION

    -- Step 1: Sync to game-side ShopItemMap (idempotent)
    IF NOT EXISTS (
        SELECT 1 FROM RanShop.dbo.ShopItemMap
        WHERE ItemMain = @ItemMain AND ItemSub = @ItemSub
    )
    BEGIN
        INSERT INTO RanShop.dbo.ShopItemMap (ItemMain, ItemSub, ItemName)
        VALUES (@ItemMain, @ItemSub, @ItemName)
    END

    -- Step 2: Insert into web-side YXShopItemMap
    INSERT INTO dbo.YXShopItemMap (
        ItemMain, ItemSub, ItemName, ItemPrice, ItemCategory,
        ItemDescription, ItemDiscount, ItemDiscountType, NewItemImage,
        ItemStatus, ItemStock, ItemTag,
        PurchaseLimitPerAccount, PurchaseLimitType
    )
    OUTPUT inserted.* INTO @InsertedItems
    VALUES (
        @ItemMain, @ItemSub, @ItemName, @ItemPrice, @ItemCategory,
        @ItemDescription, @ItemDiscount, @ItemDiscountType, @NewItemImage,
        @ItemStatus, @ItemStock, @ItemTag,
        @PurchaseLimitPerAccount, @PurchaseLimitType
    )

    COMMIT TRANSACTION

    SELECT * FROM @InsertedItems
END TRY
BEGIN CATCH
    IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION

    DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE(),
            @ErrorSeverity INT           = ERROR_SEVERITY(),
            @ErrorState INT              = ERROR_STATE()
    RAISERROR(@ErrorMessage, @ErrorSeverity, @ErrorState)
END CATCH
```

### Flow (sp_EditShopItem)

Same pattern as sp_AddShopItem, but Step 2 uses UPDATE:

```
Step 2: Update YXShopItemMap
    UPDATE dbo.YXShopItemMap
    SET ItemMain = @ItemMain, ItemSub = @ItemSub, ItemName = @ItemName, ...
    OUTPUT inserted.* INTO @InsertedItems
    WHERE ProductNum = @ProductNum
```

### Why a buffer table (@InsertedItems) instead of OUTPUT directly

Using a table variable ensures a stable result set shape for Rust's `.into_row()` and allows clean error handling in the CATCH block.

### Conventions applied (from existing stored procedures)

- `SET NOCOUNT ON` at the top
- `BEGIN TRY / BEGIN TRANSACTION` wrapping all DML
- `BEGIN CATCH` rolls back if `@@TRANCOUNT > 0`, then re-raises with `ERROR_MESSAGE()`, `ERROR_SEVERITY()`, `ERROR_STATE()`
- `USE RanShop` header
- `SET ANSI_NULLS ON` and `SET QUOTED_IDENTIFIER ON`
- `CREATE OR ALTER PROCEDURE [dbo].[...]`

---

## Section 5: Edge Case Resolutions

| Edge case                                                                | Resolution                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Admin edits and changes `(ItemMain, ItemSub)` to a new combo             | Step 1 IF NOT EXISTS runs on every call (add OR edit), so a new combo is inserted into `ShopItemMap` before the YXShopItemMap UPDATE.                                                                                                |
| Duplicate `(ItemMain, ItemSub)` already in `ShopItemMap`                 | `IF NOT EXISTS` guard makes the insert silently skip. Idempotent.                                                                                                                                                                    |
| Old `(ItemMain, ItemSub)` after edit                                     | NOT removed from `ShopItemMap`. The game-side table can retain orphan combos — this is acceptable because dispatch uses `ProductNum` from `YXShopItemMap` to find a matching row in `ShopItemMap`, and we never delete game items.   |
| NULL optional values (`@ItemDiscount`, `@ItemDiscountType`, limits)      | Passed through as-is. Defaulted to `NULL` in signature. Tiberius marshals `None` to SQL NULL.                                                                                                                                       |
| `@NewItemImage` empty string                                             | Defaulted to `N''` to match current handler behavior.                                                                                                                                                                               |
| Transaction rollback on partial failure                                  | `BEGIN TRANSACTION` covers both Step 1 and Step 2. CATCH rolls back atomically. Both succeed or both revert.                                                                                                                         |
| `ShopItemMap` schema unknown (which columns are NOT NULL?)               | The plan inserts only `ItemMain`, `ItemSub`, `ItemName`. Before deployment, confirm via `sp_help 'RanShop.dbo.ShopItemMap'` that no other NOT NULL columns without defaults exist.                                                    |
| Race: two admins add same `(ItemMain, ItemSub)` simultaneously            | `IF NOT EXISTS` is not atomic by itself. Under default READ COMMITTED two parallel calls could both pass the check. Mitigation: wrap with `WITH (HOLDLOCK, UPDLOCK)` to lock the range until commit. Keep simple for now; revisit if collisions occur. |
| `OUTPUT inserted.*` returns 0 rows when `ProductNum` not found (edit only) | Currently the Rust code returns error — preserve this behavior. Procedure will SELECT 0 rows from `@InsertedItems` and Rust's `.into_row()` returns `None`.                                                                          |

---

## Section 6: Output Column Mapping

The Rust caller deserializes via `ItemShopData::from_row(row)`, which requires these columns:

| Output column             | Required Rust type |
| ------------------------- | ------------------ |
| `ProductNum`              | `i32`              |
| `NewItemImage`            | `&str`             |
| `ItemMain`                | `i32`              |
| `ItemSub`                 | `i32`              |
| `ItemName`                | `&str`             |
| `ItemPrice`               | `f64`              |
| `ItemCategory`            | `i16`              |
| `ItemDescription`         | `&str`             |
| `ItemDiscount`            | `Option<f64>`      |
| `ItemDiscountType`        | `Option<i16>`      |
| `ItemStatus`              | `i16`              |
| `ItemStock`               | `i32`              |
| `ItemTag`                 | `&str`             |
| `PurchaseLimitPerAccount` | `Option<i32>`      |
| `PurchaseLimitType`       | `Option<i16>`      |

The procedure's `SELECT * FROM @InsertedItems` produces all of these since `@InsertedItems` mirrors `YXShopItemMap`.

**Note on `ItemPrice` type casting:** The column is `MONEY` in the table. Tiberius's `try_get::<f64, _>()` may not handle `MONEY` correctly. The procedure should cast explicitly in the SELECT: `CAST(ItemPrice AS FLOAT) AS ItemPrice` to ensure Rust receives a true `FLOAT`.

---

## Open Questions & Risks

1. **ShopItemMap exact schema** — We only know 4 columns for certain. Run `sp_help 'RanShop.dbo.ShopItemMap'` to confirm no other NOT NULL columns exist without defaults. If they do, the INSERT will fail.

2. **Unique constraint on `(ItemMain, ItemSub)`** — Consider adding one to `ShopItemMap` to prevent future duplicates. The procedure's IF NOT EXISTS provides soft idempotency; a real constraint provides hard guarantee.

3. **`ItemPrice` type drift** — Confirm whether the existing inline SQL `try_get::<f64, _>("ItemPrice")` works (returns a value) or silently returns default. If broken, the explicit `CAST(... AS FLOAT)` in the SELECT closes the bug.

4. **Race condition on duplicate check** — The IF NOT EXISTS + INSERT is not atomic. Under concurrent load, two admins could both pass the check and both insert. For now, accept this as acceptable (duplicate rows are harmless; a future migration can deduplicate). If strict uniqueness is needed, use `WITH (HOLDLOCK, UPDLOCK)` on the EXISTS check.

---

## Summary for SQLGen

- Procedure names: `sp_AddShopItem`, `sp_EditShopItem`
- Both live in `db-scripts/shopdb/Procedures/`
- Parameter list: Section 3
- Logic pattern: Section 4
- Error handling: TRY-CATCH with ROLLBACK and RAISERROR
- Output: `SELECT * FROM @InsertedItems` (all YXShopItemMap columns)
- Critical cast: `CAST(ItemPrice AS FLOAT)` in the SELECT

**Before merge:** Verify ShopItemMap schema. The procedures assume only `ItemMain`, `ItemSub`, `ItemName` columns can be inserted without other NOT NULL constraints.
