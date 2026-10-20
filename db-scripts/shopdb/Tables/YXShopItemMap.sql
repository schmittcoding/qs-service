USE RanShop;
GO

-- Drop the table if it already exists
IF OBJECT_ID('dbo.YXShopItemMap', 'U') IS NOT NULL
    DROP TABLE dbo.YXShopItemMap;
GO

CREATE TABLE dbo.YXShopItemMap (
    ItemID              INT            NOT NULL PRIMARY KEY IDENTITY (1,1),
    ProductNum          INT            NOT NULL,              -- Synced from ShopItemMap.ProductNum
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
    CONSTRAINT UQ_YXShopItemMap_ProductNum UNIQUE (ProductNum)
);
GO


-- CREATE TYPE PurchaseItemType AS TABLE (
--     ProductNum INT,
--     Quantity INT
-- );
-- GO

-- Added 27 Nov 2025, Schmitt
ALTER TABLE RanShop.dbo.YXShopItemMap
ADD PurchaseLimitPerAccount INT NULL

-- Added 02 Dec 2025, Schmitt
ALTER TABLE RanShop.dbo.YXShopItemMap
ADD NewItemImage NVARCHAR(MAX) NULL

-- Added 02 Dec 2025, Schmitt
ALTER TABLE RanShop.dbo.YXShopItemMap
ADD ItemTag NVARCHAR(15) NULL

-- Added 02 Feb 2026, Schmitt
ALTER TABLE RanShop.dbo.YXShopItemMap
ADD PurchaseLimitType SMALLINT NULL

ALTER TABLE RanShop.dbo.YXShopItemMap
ADD ShopType TINYINT NULL DEFAULT(1)