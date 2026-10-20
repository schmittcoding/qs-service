USE RanShop;
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetMileageShopItems]
    @UserNum            BIGINT,
    @IncludeInactive    BIT = 0
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @CurrentDate DATE = CAST(GETDATE() AS DATE);
    DECLARE @CurrentYear INT = YEAR(GETDATE());
    DECLARE @CurrentMonthYear INT = YEAR(GETDATE()) * 100 + MONTH(GETDATE());

    SELECT
        I.ProductNum,
        I.ItemName,
        I.ItemMain,
        I.ItemSub,
        I.ItemPrice,
        I.ItemCategory,
        I.ItemDescription,
        I.ItemStock,
        I.ItemStatus,
        I.ItemDiscountType,
        I.ItemDiscount,
        I.NewItemImage,
        I.ItemTag,
        I.ShopType,
        -- Calculated discount price
        CASE
            WHEN I.ItemDiscountType = 1 THEN
                I.ItemPrice - (I.ItemPrice * I.ItemDiscount / 100)
            WHEN I.ItemDiscountType = 2 THEN
                I.ItemPrice - I.ItemDiscount
            ELSE I.ItemPrice
        END AS DiscountedPrice,
        -- Purchase limits
        I.PurchaseLimitPerAccount,
        I.PurchaseLimitType,
        CASE I.PurchaseLimitType
            WHEN 0 THEN 'Lifetime'
            WHEN 1 THEN 'Yearly'
            WHEN 2 THEN 'Monthly'
            WHEN 3 THEN 'Daily'
            ELSE 'No Limit'
        END AS LimitTypeText,
        CASE
            WHEN I.PurchaseLimitPerAccount IS NULL THEN NULL
            WHEN I.PurchaseLimitType IS NULL THEN NULL
            WHEN I.PurchaseLimitType = 0 THEN
                I.PurchaseLimitPerAccount - ISNULL(S.TotalQuantityPurchased, 0)
            WHEN I.PurchaseLimitType = 1 THEN
                I.PurchaseLimitPerAccount -
                CASE WHEN S.CurrentYear = @CurrentYear
                     THEN S.CurrentYearQuantity ELSE 0 END
            WHEN I.PurchaseLimitType = 2 THEN
                I.PurchaseLimitPerAccount -
                CASE WHEN S.CurrentMonthYear = @CurrentMonthYear
                     THEN S.CurrentMonthQuantity ELSE 0 END
            WHEN I.PurchaseLimitType = 3 THEN
                I.PurchaseLimitPerAccount -
                CASE WHEN S.CurrentDay = @CurrentDate
                     THEN S.CurrentDayQuantity ELSE 0 END
            ELSE I.PurchaseLimitPerAccount
        END AS RemainingPurchaseLimit,
        ISNULL(S.TotalQuantityPurchased, 0) AS TotalPurchased,
        CASE WHEN S.CurrentYear = @CurrentYear
             THEN S.CurrentYearQuantity ELSE 0 END AS YearPurchased,
        CASE WHEN S.CurrentMonthYear = @CurrentMonthYear
             THEN S.CurrentMonthQuantity ELSE 0 END AS MonthPurchased,
        CASE WHEN S.CurrentDay = @CurrentDate
             THEN S.CurrentDayQuantity ELSE 0 END AS DayPurchased,
        S.LastPurchaseDate,
        -- Mileage shop items do not earn mileage
        CAST(0 AS DECIMAL(18,4)) AS MileagePointsToEarn,
        CAST(0 AS DECIMAL(5,2)) AS CurrentMileageRate
    FROM dbo.YXShopItemMap I
    LEFT JOIN dbo.MileagePurchaseSummary S
        ON S.UserNum = @UserNum AND S.ProductNum = I.ProductNum
    WHERE I.ShopType = 2
      AND (@IncludeInactive = 1 OR I.ItemStatus = 1)
    ORDER BY I.ProductNum;
END
GO
