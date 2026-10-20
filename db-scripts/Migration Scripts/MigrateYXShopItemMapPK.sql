-- Migration: Replace YXShopItemMap IDENTITY PK with surrogate ItemID
-- and sync ProductNum to match ShopItemMap.ProductNum
--
-- WHY: YXShopItemMap and ShopItemMap both auto-increment ProductNum independently.
-- When items are manually inserted into ShopItemMap (game side), the sequences
-- diverge, causing sp_DispatchItem to join against the wrong ShopItemMap row.
--
-- PREREQUISITES:
--   1. Run the pre-migration checks below BEFORE executing the migration.
--   2. Back up the RanShop database.
--   3. Stop the web API to prevent concurrent writes.
--
-- PRE-MIGRATION CHECKS:
--   -- Check for duplicate (ItemMain, ItemSub) in either table.
--   -- These MUST be resolved manually before running this migration.
--   SELECT ItemMain, ItemSub, COUNT(*) AS Cnt
--   FROM dbo.YXShopItemMap GROUP BY ItemMain, ItemSub HAVING COUNT(*) > 1;
--
--   SELECT ItemMain, ItemSub, COUNT(*) AS Cnt
--   FROM dbo.ShopItemMap GROUP BY ItemMain, ItemSub HAVING COUNT(*) > 1;
--
--   -- Preview the current divergence:
--   SELECT YX.ProductNum AS YX_PN, SIM.ProductNum AS SIM_PN, YX.ItemMain, YX.ItemSub
--   FROM dbo.YXShopItemMap YX
--   LEFT JOIN dbo.ShopItemMap SIM
--       ON SIM.ItemMain = YX.ItemMain AND SIM.ItemSub = YX.ItemSub;

USE RanShop;
GO

BEGIN TRANSACTION;
GO

-- =============================================
-- STEP 1: Copy existing data into temp table
-- =============================================
SELECT *
INTO dbo.YXShopItemMapTempNew
FROM dbo.YXShopItemMap;
GO


-- =============================================
-- STEP 2: Drop FK from YXShopItemImages
-- =============================================
IF EXISTS (
    SELECT 1 FROM sys.foreign_keys
    WHERE name = 'FK_ShopItemImages_ProductNum'
)
BEGIN
    ALTER TABLE dbo.YXShopItemImages
    DROP CONSTRAINT FK_ShopItemImages_ProductNum;
END
GO

-- =============================================
-- STEP 3: Drop old YXShopItemMap
-- =============================================
DROP TABLE dbo.YXShopItemMap;
GO

-- =============================================
-- STEP 4: Create new YXShopItemMap with ItemID
--         as PK and ProductNum as plain INT
-- =============================================
CREATE TABLE dbo.YXShopItemMap (
    ItemID              INT            NOT NULL PRIMARY KEY IDENTITY (1,1),
    ProductNum          INT            NOT NULL,
    ItemMain            INT            NOT NULL,
    ItemSub             INT            NOT NULL,
    ItemName            NVARCHAR(255)  NOT NULL,
    ItemPrice           MONEY          NOT NULL,
    ItemCategory        SMALLINT       NOT NULL,
    ItemDescription     NVARCHAR(MAX)  NULL,
    ItemDiscount        FLOAT          NULL,
    ItemDiscountType    SMALLINT       NULL,
    ItemImage           VARBINARY(MAX) NULL,
    ItemStatus          SMALLINT       NULL,
    ItemStock           INT            NOT NULL,
    PurchaseLimitPerAccount INT        NULL,
    NewItemImage        NVARCHAR(MAX)  NULL,
    ItemTag             NVARCHAR(15)   NULL,
    PurchaseLimitType   SMALLINT       NULL,
    ShopType            TINYINT        NULL
);
GO

-- =============================================
-- STEP 5: Insert missing items into ShopItemMap
--         (items in temp that have no match)
-- =============================================
INSERT INTO dbo.ShopItemMap (ItemMain, ItemSub, ItemName, ItemPrice)
SELECT T.ItemMain, T.ItemSub, T.ItemName, CAST(T.ItemPrice AS FLOAT)
FROM dbo.YXShopItemMapTempNew T
WHERE NOT EXISTS (
    SELECT 1 FROM dbo.ShopItemMap SIM
    WHERE SIM.ItemMain = T.ItemMain AND SIM.ItemSub = T.ItemSub
);
GO

-- =============================================
-- STEP 6: Seed new table with synced ProductNum
--         from ShopItemMap
-- =============================================
INSERT INTO dbo.YXShopItemMap (
    ProductNum, ItemMain, ItemSub, ItemName, ItemPrice,
    ItemCategory, ItemDescription, ItemDiscount, ItemDiscountType,
    ItemImage, ItemStatus, ItemStock, PurchaseLimitPerAccount,
    NewItemImage, ItemTag, PurchaseLimitType, ShopType
)
SELECT
    SIM.ProductNum,
    SIM.ItemMain, SIM.ItemSub, T.ItemName, T.ItemPrice,
    T.ItemCategory, T.ItemDescription, T.ItemDiscount, T.ItemDiscountType,
    T.ItemImage, T.ItemStatus, T.ItemStock, T.PurchaseLimitPerAccount,
    T.NewItemImage, T.ItemTag, T.PurchaseLimitType, T.ShopType
FROM dbo.YXShopItemMapTempNew T
INNER JOIN dbo.ShopItemMap SIM ON SIM.ProductNum = T.ProductNum
GO

-- =============================================
-- STEP 7: Recreate FK from YXShopItemImages
-- =============================================
-- IF OBJECT_ID('dbo.YXShopItemImages', 'U') IS NOT NULL
-- BEGIN
--     ALTER TABLE dbo.YXShopItemImages
--     ADD CONSTRAINT FK_ShopItemImages_ProductNum
--         FOREIGN KEY (ProductNum) REFERENCES dbo.YXShopItemMap(ProductNum)
--         ON DELETE CASCADE;
-- END
-- GO

-- =============================================
-- STEP 8: Drop temp table
-- =============================================
DROP TABLE dbo.YXShopItemMapTempNew;
GO

COMMIT TRANSACTION;
GO

-- =============================================
-- POST-MIGRATION VERIFICATION
-- =============================================
PRINT 'Migration complete. Running verification...';

DECLARE @Mismatches INT;
SELECT @Mismatches = COUNT(*)
FROM dbo.YXShopItemMap YX
INNER JOIN dbo.ShopItemMap SIM
    ON SIM.ItemMain = YX.ItemMain AND SIM.ItemSub = YX.ItemSub
WHERE YX.ProductNum <> SIM.ProductNum;

PRINT 'ProductNum mismatches: ' + CAST(@Mismatches AS VARCHAR(10));
PRINT 'ProductNum IsIdentity: ' + CAST(COLUMNPROPERTY(OBJECT_ID('dbo.YXShopItemMap'), 'ProductNum', 'IsIdentity') AS VARCHAR(1));
PRINT 'ItemID IsIdentity: ' + CAST(COLUMNPROPERTY(OBJECT_ID('dbo.YXShopItemMap'), 'ItemID', 'IsIdentity') AS VARCHAR(1));
GO
