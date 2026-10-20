USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetUserOrders]
    @UserNum    INT,
    @Page       INT = 1,
    @PageSize   INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- User-facing: omits UserUID/UserNum (caller already knows who they are)
    -- and PointsBefore/PointsAfter (internal financial tracking, not needed in list view).
    SELECT
        OrderID,
        OrderDate,
        TotalItems,
        TotalQuantity,
        TotalAmount,
        TotalDiscount,
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
