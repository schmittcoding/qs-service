USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

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
    @PurchaseLimitType       SMALLINT      = NULL,
    @ShopType                TINYINT       = 1
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @ShopItemMapProductNum INT;

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

        -- IF @ShopItemMapProductNum <> @ProductNum
        -- BEGIN
        --     UPDATE dbo.YXShopItemMap
        --     SET ProductNum = @ShopItemMapProductNum
        --     WHERE ProductNum = @ProductNum;

        --     SET @ProductNum = @ShopItemMapProductNum;
        -- END

        UPDATE dbo.ShopItemMap
        SET 
            ItemMain = @ItemMain,
            ItemSub = @ItemSub,
            ItemName = @ItemName,
            ItemPrice = @ItemPrice
        WHERE ProductNum = @ProductNum;

        UPDATE dbo.YXShopItemMap
        SET
            ItemMain = @ItemMain,
            ItemSub = @ItemSub,
            ItemName = @ItemName,
            ItemPrice = @ItemPrice,
            ItemCategory = @ItemCategory,
            ItemDescription = @ItemDescription,
            ItemDiscount = @ItemDiscount,
            ItemDiscountType = @ItemDiscountType,
            NewItemImage = @NewItemImage,
            ItemStatus = @ItemStatus,
            ItemStock = @ItemStock,
            ItemTag = @ItemTag,
            PurchaseLimitPerAccount = @PurchaseLimitPerAccount,
            PurchaseLimitType = @PurchaseLimitType,
            ShopType = @ShopType
        WHERE ProductNum = @ProductNum;

        COMMIT TRANSACTION;

        SELECT * FROM RanShop.dbo.YXShopItemMap WHERE ProductNum = @ProductNum;
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
