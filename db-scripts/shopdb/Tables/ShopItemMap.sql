-- ShopItemMap - Game-side Item Reference Table
--
-- IMPORTANT: This table is managed by the game server, not by this repository.
-- This file documents the expected schema for reference and integration.
--
-- The table is used by the game server's item dispatcher (sp_DispatchItem)
-- and our web shop procedures (sp_AddShopItem, sp_EditShopItem) for syncing
-- item metadata from the web shop to the game server.
--
-- DO NOT ALTER this table directly unless you understand the implications
-- for the game server's dispatcher and item lookup systems.

USE RanShop
GO

-- If this table doesn't exist, the game server setup is incomplete.
-- Contact the game server team to ensure ShopItemMap exists with the following schema:
--
-- Required columns (referenced by existing procedures):
--   ProductNum     INT          - Item identifier (referenced by sp_DispatchItem)
--   ItemMain       INT          - Game item main category
--   ItemSub        INT          - Game item sub-category
--   ItemName       NVARCHAR(n)  - Item display name
--   ItemPrice      FLOAT        - Item price (used for game sync)
--
-- The table likely has a PRIMARY KEY on ProductNum and possibly a UNIQUE constraint
-- on (ItemMain, ItemSub) to prevent duplicate item definitions.
--
-- Web procedures sp_AddShopItem and sp_EditShopItem INSERT rows:
--   INSERT INTO ShopItemMap (ItemMain, ItemSub, ItemName, ItemPrice)
--   VALUES (@ItemMain, @ItemSub, @ItemName, @ItemPrice)
--
-- If ShopItemMap has additional NOT NULL columns without defaults,
-- those procedures will fail on INSERT. Verify the actual schema exists
-- before deploying the web shop sync procedures.

IF OBJECT_ID('dbo.ShopItemMap', 'U') IS NULL
BEGIN
    PRINT 'WARNING: ShopItemMap table does not exist in RanShop database.'
    PRINT 'This table must be created by the game server team.'
    PRINT 'Web shop sync procedures (sp_AddShopItem, sp_EditShopItem) will fail without it.'
END
GO
