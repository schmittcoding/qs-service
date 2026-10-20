USE RanShop;
GO

-- Added 14 Mar 2026, Gallery images for shop items
IF OBJECT_ID('dbo.YXShopItemImages', 'U') IS NULL
BEGIN
    CREATE TABLE dbo.YXShopItemImages (
        ImageID         UNIQUEIDENTIFIER NOT NULL PRIMARY KEY DEFAULT NEWID(),
        ProductNum      INT              NOT NULL,
        ImageUrl        NVARCHAR(MAX)    NOT NULL,
        SortOrder       SMALLINT         NOT NULL DEFAULT 0,
        CreatedAt       DATETIME         NOT NULL DEFAULT GETDATE(),

        CONSTRAINT FK_ShopItemImages_ProductNum
            FOREIGN KEY (ProductNum) REFERENCES dbo.YXShopItemMap(ProductNum)
            ON DELETE CASCADE
    );

    CREATE NONCLUSTERED INDEX IX_ShopItemImages_ProductNum
        ON dbo.YXShopItemImages (ProductNum);
END
GO
