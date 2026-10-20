USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetItemDispatchItemOptions]
    @Search NVARCHAR(100) = NULL,
    @Page INT = 0,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        SIM.ProductNum,
        SIM.ItemMain,
        SIM.ItemSub,
        SIM.ItemName,
        COUNT(*) OVER() AS TotalRecords
    FROM dbo.ShopItemMap AS SIM
    WHERE
        @Search IS NULL
        OR SIM.ItemName LIKE '%' + @Search + '%'
    ORDER BY SIM.ItemName ASC
    OFFSET @Page ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
