USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetUserShopPurchases]
    @UserNum INT,
    @Status INT = NULL,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @UserID NVARCHAR(20);
    SELECT @UserID = UserID FROM RanUser.dbo.GSUserInfo WHERE UserNum = @UserNum;

    IF NOT EXISTS (
        SELECT UserNum FROM RanUser.dbo.GSUserInfo WHERE UserNum = @UserNum
    )
    BEGIN
        SELECT 'USER_NOT_FOUND' AS MESSAGE;
        RETURN;
    END

    -- Compute pagination offsets
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    ;WITH PurchaseSummary AS
    (
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
                WHEN SP.ItemName IS NOT NULL THEN SP.ItemName
                WHEN I.ItemName IS NOT NULL THEN I.ItemName
                ELSE NULL
            END AS ItemName
        FROM dbo.ShopPurchase SP
        LEFT JOIN RanShop.dbo.ShopItemMap I
            ON I.ProductNum = SP.ProductNum
        INNER JOIN RanUser.dbo.GSUserInfo UI
            ON UI.UserID = SP.UserUID
        WHERE
            UI.UserNum = @UserNum
            AND (@Status IS NULL OR SP.PurFlag = @Status)
    )
    SELECT PS.*,
        (SELECT COUNT(*) FROM PurchaseSummary) AS TotalRecords
    FROM PurchaseSummary AS PS
    ORDER BY PS.PurchaseDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
