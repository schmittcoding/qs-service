USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetOrdersByUserNum]
    @UserNum    INT,
    @Page       INT = 1,
    @PageSize   INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    SELECT
        OrderID,
        UserNum,
        UserUID,
        OrderDate,
        TotalItems,
        TotalQuantity,
        TotalAmount,
        TotalDiscount,
        PointsBefore,
        PointsAfter,
        OrderStatus,
        CreatedAt,
        COUNT(*) OVER() AS TotalRecords
    FROM dbo.ShopOrder
    WHERE UserNum = @UserNum
    ORDER BY CreatedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
