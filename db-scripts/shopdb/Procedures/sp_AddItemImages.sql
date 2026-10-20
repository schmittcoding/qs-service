USE RanShop;
GO

-- Added 14 Mar 2026, Insert gallery images for a shop item
CREATE OR ALTER PROCEDURE sp_AddItemImages
    @ProductNum INT,
    @ImagesJson NVARCHAR(MAX)  -- JSON array: [{"url": "...", "sort_order": 0}, ...]
AS
BEGIN
    SET NOCOUNT ON;

    INSERT INTO dbo.YXShopItemImages (ProductNum, ImageUrl, SortOrder)
    SELECT
        @ProductNum,
        j.[url],
        j.[sort_order]
    FROM OPENJSON(@ImagesJson)
    WITH (
        [url]        NVARCHAR(MAX) '$.url',
        [sort_order] SMALLINT      '$.sort_order'
    ) AS j;

    -- Return all images for this product
    SELECT ImageID, ProductNum, ImageUrl, SortOrder, CreatedAt
    FROM dbo.YXShopItemImages
    WHERE ProductNum = @ProductNum
    ORDER BY SortOrder;
END
GO
