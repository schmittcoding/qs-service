USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

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
    @PurchaseLimitType       SMALLINT      = NULL,
    @ShopType                TINYINT       = 1
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @ShopItemMapProductNum INT;

    DECLARE @InsertedItems TABLE (
        ProductNum               INT,
        ItemMain                 INT,
        ItemSub                  INT,
        ItemName                 NVARCHAR(255),
        ItemPrice                FLOAT,
        ItemCategory             SMALLINT,
        ItemDescription          NVARCHAR(MAX),
        ItemDiscount             FLOAT,
        ItemDiscountType         SMALLINT,
        ItemImage                VARBINARY(MAX),
        ItemStatus               SMALLINT,
        ItemStock                INT,
        PurchaseLimitPerAccount  INT,
        NewItemImage             NVARCHAR(MAX),
        ItemTag                  NVARCHAR(15),
        PurchaseLimitType        SMALLINT,
        ShopType                 TINYINT
    );

    BEGIN TRY
        BEGIN TRANSACTION

        -- IF NOT EXISTS (
        --     SELECT 1 FROM RanShop.dbo.ShopItemMap
        --     WHERE ItemMain = @ItemMain AND ItemSub = @ItemSub
        -- )
        -- BEGIN
        --     INSERT INTO RanShop.dbo.ShopItemMap (ItemMain, ItemSub, ItemName, ItemPrice)
        --     VALUES (@ItemMain, @ItemSub, @ItemName, @ItemPrice);

        --     SET @ShopItemMapProductNum = SCOPE_IDENTITY();
        -- END
        -- ELSE
        -- BEGIN
        --     SELECT @ShopItemMapProductNum = ProductNum
        --     FROM RanShop.dbo.ShopItemMap
        --     WHERE ItemMain = @ItemMain AND ItemSub = @ItemSub;
        -- END

        INSERT INTO RanShop.dbo.ShopItemMap (ItemMain, ItemSub, ItemName, ItemPrice)
        VALUES (@ItemMain, @ItemSub, @ItemName, @ItemPrice);

        SET @ShopItemMapProductNum = SCOPE_IDENTITY();

        INSERT INTO dbo.YXShopItemMap (
            ProductNum,
            ItemMain, ItemSub, ItemName, ItemPrice, ItemCategory,
            ItemDescription, ItemDiscount, ItemDiscountType, NewItemImage,
            ItemStatus, ItemStock, ItemTag,
            PurchaseLimitPerAccount, PurchaseLimitType, ShopType
        )
        OUTPUT
            inserted.ProductNum,
            inserted.ItemMain,
            inserted.ItemSub,
            inserted.ItemName,
            CAST(inserted.ItemPrice AS FLOAT) AS ItemPrice,
            inserted.ItemCategory,
            inserted.ItemDescription,
            inserted.ItemDiscount,
            inserted.ItemDiscountType,
            inserted.ItemImage,
            inserted.ItemStatus,
            inserted.ItemStock,
            inserted.PurchaseLimitPerAccount,
            inserted.NewItemImage,
            inserted.ItemTag,
            inserted.PurchaseLimitType,
            inserted.ShopType
        INTO @InsertedItems
        VALUES (
            @ShopItemMapProductNum,
            @ItemMain, @ItemSub, @ItemName, @ItemPrice, @ItemCategory,
            @ItemDescription, @ItemDiscount, @ItemDiscountType, @NewItemImage,
            @ItemStatus, @ItemStock, @ItemTag,
            @PurchaseLimitPerAccount, @PurchaseLimitType, @ShopType
        );

        COMMIT TRANSACTION;

        SELECT * FROM @InsertedItems;
    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0
            ROLLBACK TRANSACTION;

        DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE(),
                @ErrorSeverity INT           = ERROR_SEVERITY(),
                @ErrorState INT              = ERROR_STATE();

        RAISERROR(@ErrorMessage, @ErrorSeverity, @ErrorState);
    END CATCH
END
GO
