USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetPaginatedShopData]
    @Search         NVARCHAR(100) = NULL,
    @Category       NVARCHAR(MAX) = NULL,
    @Status         NVARCHAR(MAX) = NULL,
    @PurchaseLimit  NVARCHAR(MAX) = NULL,
    @ShopType       TINYINT = 1,
    @Page           INT = 0,
    @PageSize       INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    IF (@Page < 0) SET @Page = 0;
    IF (@PageSize <= 0) SET @PageSize = 20;

    -- Parse comma-separated strings into temp tables for filtering
    -- PurchaseLimitPerAccount supports -1 as a sentinel for NULL (unlimited)
    DECLARE @Categories TABLE (Value INT);
    DECLARE @Statuses TABLE (Value INT);
    DECLARE @PurchaseLimits TABLE (Value INT);

    IF @Category IS NOT NULL
        INSERT INTO @Categories (Value)
        SELECT Value FROM dbo.fn_SplitInts(@Category, ',');

    IF @Status IS NOT NULL
        INSERT INTO @Statuses (Value)
        SELECT Value FROM dbo.fn_SplitInts(@Status, ',');

    IF @PurchaseLimit IS NOT NULL
        INSERT INTO @PurchaseLimits (Value)
        SELECT Value FROM dbo.fn_SplitInts(@PurchaseLimit, ',');

    ;WITH FilteredItemShop AS (
        SELECT *
        FROM dbo.YXShopItemMap I WITH (NOLOCK)
        WHERE
            I.ShopType = @ShopType AND
            (@Search IS NULL OR I.ItemName LIKE '%' + @Search + '%') AND
            -- String filters: if the temp table is empty, no filter is applied (all values match)
            (NOT EXISTS (SELECT 1 FROM @Categories) OR I.ItemCategory IN (SELECT Value FROM @Categories)) AND
            (NOT EXISTS (SELECT 1 FROM @Statuses) OR I.ItemStatus IN (SELECT Value FROM @Statuses)) AND
            -- Purchase limit filter: handle -1 sentinel for NULL (unlimited) and numeric values
            (NOT EXISTS (SELECT 1 FROM @PurchaseLimits) OR
             I.PurchaseLimitType IN (SELECT Value FROM @PurchaseLimits WHERE Value > 0) OR
             (-1 IN (SELECT Value FROM @PurchaseLimits) AND I.PurchaseLimitType IS NULL))
    )
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
    FROM FilteredItemShop
    ORDER BY ProductNum DESC
    OFFSET (@Page * @PageSize) ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
