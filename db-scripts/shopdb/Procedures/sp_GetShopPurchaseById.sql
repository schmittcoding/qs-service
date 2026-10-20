USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetShopPurchaseById]
    @PurchaseID INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        SP.PurKey AS PurchaseID,
        UI.UserNum,
        UI.UserID,
        SP.ProductNum,
        SP.PurDate AS PurchaseDate,
        SP.PurChgDate AS LastChangeDate,
        SP.PurFlag AS PurchaseStatus,
        I.ItemMain,
        I.ItemSub,
        CASE
            WHEN I.ItemName IS NOT NULL THEN I.ItemName
            WHEN SP.ItemName IS NOT NULL THEN SP.ItemName
            ELSE NULL
        END AS ItemName
    FROM dbo.ShopPurchase SP
    LEFT JOIN RanShop.dbo.ShopItemMap I
        ON I.ProductNum = SP.ProductNum
    INNER JOIN RanUser.dbo.GSUserInfo UI
        ON UI.UserID = SP.UserUID
    WHERE SP.PurKey = @PurchaseID;
END
