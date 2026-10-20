USE RanShop;
GO

-- Added 14 Mar 2026, Delete specific gallery images by their UUIDs
CREATE OR ALTER PROCEDURE sp_DeleteItemImages
    @ProductNum INT,
    @ImageIdsJson NVARCHAR(MAX)  -- JSON array: ["uuid-1", "uuid-2"]
AS
BEGIN
    SET NOCOUNT ON;

    DELETE FROM dbo.YXShopItemImages
    WHERE ProductNum = @ProductNum
      AND ImageID IN (
          SELECT CAST([value] AS UNIQUEIDENTIFIER)
          FROM OPENJSON(@ImageIdsJson)
      );
END
GO
