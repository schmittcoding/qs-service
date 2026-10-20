USE RanShop;
GO

CREATE OR ALTER TRIGGER [dbo].[trg_ShopPurchaseSummary_AuditSuspicious]
ON [dbo].[ShopPurchaseSummary]
AFTER INSERT, UPDATE
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Operation VARCHAR(10) = CASE
        WHEN EXISTS (SELECT 1 FROM deleted) THEN 'UPDATE'
        ELSE 'INSERT'
    END;

    -- Capture connection context for forensics
    DECLARE @Host       NVARCHAR(128) = HOST_NAME();
    DECLARE @App        NVARCHAR(128) = APP_NAME();
    DECLARE @Login      NVARCHAR(128) = SUSER_SNAME();

    -- Rule 1: Zero or negative TotalAmountSpent with items purchased
    INSERT INTO dbo.ShopSuspiciousActivityLog
        (AlertType, Severity, UserUID, UserNum, ProductNum, Quantity, AmountValue,
         TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
    SELECT
        'ZERO_AMOUNT',
        4, -- Critical
        i.UserUID,
        i.UserNum,
        i.ProductNum,
        i.TotalQuantityPurchased,
        i.TotalAmountSpent,
        'ShopPurchaseSummary',
        @Operation,
        (SELECT
            i.UserUID AS UserUID,
            i.UserNum AS UserNum,
            i.ProductNum AS ProductNum,
            i.TotalQuantityPurchased AS TotalQuantityPurchased,
            i.TotalAmountSpent AS TotalAmountSpent,
            i.PurchaseCount AS PurchaseCount,
            i.LastPurchaseDate AS LastPurchaseDate,
            item.ItemName AS ItemName,
            item.ItemPrice AS ItemPrice,
            item.ItemDiscount AS ItemDiscount,
            item.ItemDiscountType AS ItemDiscountType,
            item.ItemStatus AS ItemStatus,
            @Host AS ConnectionHost,
            @App AS ConnectionApp,
            @Login AS ConnectionLogin
        FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
        @Host, @App, @Login
    FROM inserted i
    LEFT JOIN dbo.YXShopItemMap item ON item.ProductNum = i.ProductNum
    WHERE i.TotalQuantityPurchased > 0
      AND i.TotalAmountSpent <= 0;

    -- Rule 2: Detect writes NOT coming from the expected application
    -- Adjust the APP_NAME pattern to match your API's connection string ApplicationName
    INSERT INTO dbo.ShopSuspiciousActivityLog
        (AlertType, Severity, UserUID, UserNum, ProductNum, Quantity, AmountValue,
         TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
    SELECT
        'DIRECT_' + @Operation,
        4, -- Critical: someone is writing directly to the table
        i.UserUID,
        i.UserNum,
        i.ProductNum,
        i.TotalQuantityPurchased,
        i.TotalAmountSpent,
        'ShopPurchaseSummary',
        @Operation,
        (SELECT
            i.UserUID AS UserUID,
            i.UserNum AS UserNum,
            i.ProductNum AS ProductNum,
            i.TotalQuantityPurchased AS TotalQuantityPurchased,
            i.TotalAmountSpent AS TotalAmountSpent,
            @Host AS ConnectionHost,
            @App AS ConnectionApp,
            @Login AS ConnectionLogin
        FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
        @Host, @App, @Login
    FROM inserted i
    WHERE @App NOT LIKE '%gs-api%'       -- Your API's ApplicationName in conn string
      AND @App NOT LIKE '%sp_shop_purchase%'; -- Allow SP context

    -- Rule 3: Suspiciously high quantity in a single transaction
    INSERT INTO dbo.ShopSuspiciousActivityLog
        (AlertType, Severity, UserUID, UserNum, ProductNum, Quantity, AmountValue,
         TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
    SELECT
        'EXCESSIVE_QUANTITY',
        3, -- High
        i.UserUID,
        i.UserNum,
        i.ProductNum,
        i.TotalQuantityPurchased,
        i.TotalAmountSpent,
        'ShopPurchaseSummary',
        @Operation,
        (SELECT
            i.UserUID AS UserUID,
            i.UserNum AS UserNum,
            i.ProductNum AS ProductNum,
            i.TotalQuantityPurchased AS TotalQuantityPurchased,
            i.TotalAmountSpent AS TotalAmountSpent,
            i.CurrentDayQuantity AS CurrentDayQuantity,
            @Host AS ConnectionHost,
            @App AS ConnectionApp,
            @Login AS ConnectionLogin
        FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
        @Host, @App, @Login
    FROM inserted i
    LEFT JOIN deleted d ON d.UserNum = i.UserNum AND d.ProductNum = i.ProductNum
    WHERE @Operation = 'UPDATE'
      AND (i.TotalQuantityPurchased - ISNULL(d.TotalQuantityPurchased, 0)) > 10; -- Threshold: >10 items in one go

    -- Rule 4: Amount decreased (possible tampering / negative spend)
    INSERT INTO dbo.ShopSuspiciousActivityLog
        (AlertType, Severity, UserUID, UserNum, ProductNum, Quantity, AmountValue,
         TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
    SELECT
        'NEGATIVE_AMOUNT',
        4, -- Critical
        i.UserUID,
        i.UserNum,
        i.ProductNum,
        i.TotalQuantityPurchased,
        i.TotalAmountSpent,
        'ShopPurchaseSummary',
        @Operation,
        (SELECT
            d.TotalAmountSpent AS PreviousAmount,
            i.TotalAmountSpent AS NewAmount,
            d.TotalQuantityPurchased AS PreviousQuantity,
            i.TotalQuantityPurchased AS NewQuantity,
            @Host AS ConnectionHost,
            @App AS ConnectionApp,
            @Login AS ConnectionLogin
        FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
        @Host, @App, @Login
    FROM inserted i
    INNER JOIN deleted d ON d.UserNum = i.UserNum AND d.ProductNum = i.ProductNum
    WHERE i.TotalAmountSpent < d.TotalAmountSpent;
END
GO
