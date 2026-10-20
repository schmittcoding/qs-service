USE RanShop;
GO

-- Added 14 Mar 2026, Get gallery images for a shop item
CREATE OR ALTER PROCEDURE sp_GetItemImages
    @ProductNum INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT ImageID, ProductNum, ImageUrl, SortOrder, CreatedAt
    FROM dbo.YXShopItemImages
    WHERE ProductNum = @ProductNum
    ORDER BY SortOrder;
END
GO
