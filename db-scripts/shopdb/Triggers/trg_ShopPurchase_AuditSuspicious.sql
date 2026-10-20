USE RanShop;
GO

CREATE OR ALTER TRIGGER [dbo].[trg_ShopPurchase_AuditSuspicious]
ON [dbo].[ShopPurchase]
AFTER INSERT
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Host       NVARCHAR(128) = HOST_NAME();
    DECLARE @App        NVARCHAR(128) = APP_NAME();
    DECLARE @Login      NVARCHAR(128) = SUSER_SNAME();

    -- Rule 1: Inserts NOT coming from the expected application context
    -- This catches direct table manipulation bypassing the API
    IF @App NOT IN ('gs-api', '.Net SqlClient Data Provider')
    BEGIN
        INSERT INTO dbo.ShopSuspiciousActivityLog
            (AlertType, Severity, UserNum, ProductNum, Quantity,
             TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
        SELECT
            'DIRECT_INSERT',
            4, -- Critical
            NULL, -- ShopPurchase uses UserUID (username), not UserNum
            i.ProductNum,
            1, -- Each row = 1 unit
            'ShopPurchase',
            'INSERT',
            (SELECT
                i.UserUID AS UserUID,
                i.ProductNum AS ProductNum,
                i.ItemName AS ItemName,
                i.PurDate AS PurDate,
                @Host AS ConnectionHost,
                @App AS ConnectionApp,
                @Login AS ConnectionLogin
            FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
            @Host, @App, @Login
        FROM inserted i;
    END

    -- Rule 2: Items inserted that don't exist in the shop catalog
    INSERT INTO dbo.ShopSuspiciousActivityLog
        (AlertType, Severity, ProductNum,
         TableName, Operation, Details, ConnectionHost, ConnectionApp, ConnectionLogin)
    SELECT
        'UNKNOWN_PRODUCT',
        4, -- Critical
        i.ProductNum,
        'ShopPurchase',
        'INSERT',
        (SELECT
            i.UserUID AS UserUID,
            i.ProductNum AS ProductNum,
            i.ItemName AS ItemName,
            @Host AS ConnectionHost,
            @App AS ConnectionApp,
            @Login AS ConnectionLogin
        FOR JSON PATH, WITHOUT_ARRAY_WRAPPER),
        @Host, @App, @Login
    FROM inserted i
    LEFT JOIN dbo.ShopItemMap item ON item.ProductNum = i.ProductNum
    WHERE item.ProductNum IS NULL;
END
GO
