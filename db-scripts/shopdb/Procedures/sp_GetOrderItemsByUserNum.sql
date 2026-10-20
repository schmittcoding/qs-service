USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetOrderItemsByUserNum]
    @UserNum    INT,
    @Page       INT = 1,
    @PageSize   INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    SELECT
        OI.OrderItemID,
        OI.OrderID,
        OI.UserNum,
        OI.ProductNum,
        OI.ItemName,
        OI.ItemMain,
        OI.ItemSub,
        OI.Quantity,
        OI.UnitPrice,
        OI.DiscountType,
        OI.DiscountValue,
        OI.FinalUnitPrice,
        OI.LineTotal,
        O.OrderDate,
        O.OrderStatus,
        COUNT(*) OVER() AS TotalRecords
    FROM dbo.ShopOrderItem OI
    INNER JOIN dbo.ShopOrder O ON O.OrderID = OI.OrderID
    WHERE OI.UserNum = @UserNum
    ORDER BY OI.OrderItemID DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
