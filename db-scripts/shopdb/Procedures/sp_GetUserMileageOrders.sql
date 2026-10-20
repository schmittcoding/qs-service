USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetUserMileageOrders]
    @UserNum    INT,
    @Page       INT = 1,
    @PageSize   INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

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
    FROM dbo.MileageOrder
    WHERE UserNum = @UserNum
    ORDER BY CreatedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
