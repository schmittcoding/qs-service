USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetShopPurchases]
    @SearchNum INT = NULL,
    @Search NVARCHAR(50) = NULL,
    @Status INT = NULL,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    -- Compute pagination offsets
    DECLARE @Offset INT = @Page * @PageSize;

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
    INTO #FilteredPurchases
    FROM dbo.ShopPurchase SP WITH (NOLOCK)
    LEFT JOIN RanShop.dbo.ShopItemMap I WITH (NOLOCK)
        ON I.ProductNum = SP.ProductNum
    INNER JOIN RanUser.dbo.GSUserInfo UI WITH (NOLOCK)
        ON UI.UserID = SP.UserUID
    WHERE
        (@Search IS NULL
            OR I.ItemName LIKE @Search + '%'
            OR UI.UserNum = @SearchNum
            OR UI.UserID = @Search)
        AND (@Status IS NULL OR SP.PurFlag = @Status);

    SELECT COUNT(*) AS TotalRecords FROM #FilteredPurchases;

    SELECT *
    FROM #FilteredPurchases
    ORDER BY PurchaseDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;

    DROP TABLE #FilteredPurchases;
END
