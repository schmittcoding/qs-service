USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetAdminMileageOrders]
    @SearchNum  INT             = NULL,
    @Search     NVARCHAR(50)    = NULL,
    @Status     TINYINT         = NULL,
    @Page       INT             = 1,
    @PageSize   INT             = 10
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
    FROM dbo.MileageOrder
    WHERE
        (@Search IS NULL OR UserUID LIKE @Search + '%')
        AND (@SearchNum IS NULL OR UserNum = @SearchNum)
        AND (@Status IS NULL OR OrderStatus = @Status)
    ORDER BY CreatedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
