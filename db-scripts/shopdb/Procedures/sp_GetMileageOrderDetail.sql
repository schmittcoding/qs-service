USE RanShop
GO

-- Returns two result sets:
--   Set 1: MileageOrder header row
--   Set 2: MileageOrderItem rows for that order
-- When @UserNum is provided, the order must belong to that user (user-facing).
-- When @UserNum is NULL, no ownership check is applied (admin-facing).
CREATE OR ALTER PROCEDURE [dbo].[sp_GetMileageOrderDetail]
    @OrderID    BIGINT,
    @UserNum    INT = NULL
AS
BEGIN
    SET NOCOUNT ON;

    -- Result set 1: order header
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
        CreatedAt
    FROM dbo.MileageOrder
    WHERE OrderID = @OrderID
      AND (@UserNum IS NULL OR UserNum = @UserNum);

    -- Result set 2: line items (only if the order was found for that user)
    IF EXISTS (
        SELECT 1 FROM dbo.MileageOrder
        WHERE OrderID = @OrderID
          AND (@UserNum IS NULL OR UserNum = @UserNum)
    )
    BEGIN
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
            OI.LineTotal
        FROM dbo.MileageOrderItem OI
        WHERE OI.OrderID = @OrderID
        ORDER BY OI.OrderItemID;
    END
END
GO
