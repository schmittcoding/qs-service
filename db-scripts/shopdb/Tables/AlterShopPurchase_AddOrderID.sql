-- Migration: Add OrderID to ShopPurchase
-- Links each per-unit item bank insertion back to the ShopOrder it originated from.
-- NULL-able so existing rows (pre-migration) remain valid.

USE RanShop
GO

ALTER TABLE dbo.ShopPurchase
    ADD OrderID BIGINT NULL
        REFERENCES dbo.ShopOrder(OrderID);
GO

CREATE INDEX IX_ShopPurchase_OrderID
    ON dbo.ShopPurchase(OrderID)
    WHERE OrderID IS NOT NULL;
GO
