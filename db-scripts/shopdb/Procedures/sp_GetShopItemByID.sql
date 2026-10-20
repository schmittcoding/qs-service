USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetShopItemByID]
    @ProductNum     INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        *,
        -- Calculated discount price
        CASE
            WHEN ItemDiscountType = 1 THEN
                ItemPrice - (ItemPrice * ItemDiscount / 100)
            WHEN ItemDiscountType = 2 THEN
                ItemPrice - ItemDiscount
            ELSE ItemPrice
        END AS DiscountedPrice,
        -- Purchase limits
        CASE PurchaseLimitType
            WHEN 0 THEN 'Lifetime'
            WHEN 1 THEN 'Yearly'
            WHEN 2 THEN 'Monthly'
            WHEN 3 THEN 'Daily'
            ELSE NULL
        END AS LimitTypeText,
        COUNT(*) OVER() AS TotalRecords
    FROM dbo.YXShopItemMap
    WHERE ProductNum = @ProductNum;
END
GO
