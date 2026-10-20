USE RanShop
GO

ALTER DATABASE [RanShop] SET COMPATIBILITY_LEVEL = 150;
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_DispatchItem]
    @DispatchBy      UNIQUEIDENTIFIER,
    @DispatchNotes   NVARCHAR(255) = NULL,
    @ItemsJson       NVARCHAR(MAX),   -- JSON array of items: [{ "product_num": 123 }]
    @CharsJson       NVARCHAR(MAX)    -- JSON array of chars: [{ "cha_num": 999 }]
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @DispatchID UNIQUEIDENTIFIER = NEWID();
    DECLARE @Now        DATETIME        = GETDATE();

    ----------------------------------------------
    -- STEP 0: Parse JSON into temp tables
    ----------------------------------------------
    CREATE TABLE #Items (
        ProductNum INT,
        Quantity   INT,
        PRIMARY KEY(ProductNum)
    );

    CREATE TABLE #Chars (
        ChaNum INT PRIMARY KEY
    );

    -- Parse items
    INSERT INTO #Items (ProductNum, Quantity)
    SELECT product_num, quantity
    FROM OPENJSON(@ItemsJson)
         WITH (
            product_num INT '$.product_num',
            quantity    INT '$.quantity'
        )
    WHERE product_num IS NOT NULL AND quantity > 0;

    IF NOT EXISTS (SELECT 1 FROM #Items)
    BEGIN
        RAISERROR('No valid items found in JSON.',16,1);
        RETURN;
    END

    -- Parse characters
    INSERT INTO #Chars (ChaNum)
    SELECT cha_num
    FROM OPENJSON(@CharsJson)
         WITH (cha_num INT '$.cha_num')
    WHERE cha_num IS NOT NULL;

    IF NOT EXISTS (SELECT 1 FROM #Chars)
    BEGIN
        RAISERROR('No valid character entries found in JSON.',16,1);
        RETURN;
    END

    BEGIN TRY
        BEGIN TRANSACTION;

        ----------------------------------------------
        -- STEP 1: Create main dispatch record
        ----------------------------------------------
        INSERT INTO dbo.ItemDispatchLog (
            DispatchID, DispatchBy, DispatchNotes, CreatedAt
        )
        VALUES (@DispatchID, @DispatchBy, @DispatchNotes, @Now);


        ----------------------------------------------
        -- STEP 2: Insert item records from ShopItemMap
        ----------------------------------------------
        INSERT INTO dbo.ItemDispatchItemLog (
            DispatchItemID,
            DispatchID,
            DispatchProductNum,
            DispatchItemMain,
            DispatchItemSub,
            DispatchItemName,
            CreatedAt
        )
        SELECT
            NEWID(),
            @DispatchID,
            SIM.ProductNum,
            SIM.ItemMain,
            SIM.ItemSub,
            SIM.ItemName,
            @Now
        FROM #Items I
        JOIN dbo.PurchaseNumber n ON n.number <= I.Quantity
        INNER JOIN RanShop.dbo.ShopItemMap SIM
            ON SIM.ProductNum = I.ProductNum;


        ----------------------------------------------
        -- STEP 3: Insert character records from ChaInfo
        ----------------------------------------------
        INSERT INTO dbo.ItemDispatchChaLog (
            DispatchChaID,
            DispatchID,
            DispatchUserNum,
            DispatchChaNum,
            DispatchChaName,
            CreatedAt
        )
        SELECT
            NEWID(),
            @DispatchID,
            CI.UserNum,
            CI.ChaNum,
            CI.ChaName,
            @Now
        FROM #Chars C
        INNER JOIN RanGameS1.dbo.ChaInfo CI
            ON CI.ChaNum = C.ChaNum;


        ----------------------------------------------
        -- STEP 4: Insert into ShopPurchase (User Bank)
        ----------------------------------------------
        INSERT INTO dbo.ShopPurchase (
            UserUID,
            ProductNum,
            ItemName,
            Remarks
        )
        SELECT
            UI.UserID,
            SIM.ProductNum,
            SIM.ItemName,
            @DispatchID
        FROM #Chars C
        INNER JOIN RanGameS1.dbo.ChaInfo CI
            ON CI.ChaNum = C.ChaNum
        CROSS JOIN #Items I
        JOIN dbo.PurchaseNumber n ON n.number <= I.Quantity
        INNER JOIN RanShop.dbo.ShopItemMap SIM
            ON SIM.ProductNum = I.ProductNum
        INNER JOIN RanUser.dbo.GSUserInfo UI
            ON UI.UserNum = CI.UserNum;


        ----------------------------------------------
        -- Done
        ----------------------------------------------
        COMMIT TRANSACTION;

        SELECT 'SUCCESS' AS Message, @DispatchID AS DispatchID;

    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION;

        DECLARE @ErrorMessage NVARCHAR(4000),
                @ErrorSeverity INT,
                @ErrorState INT;

        SELECT
            @ErrorMessage = ERROR_MESSAGE(),
            @ErrorSeverity = ERROR_SEVERITY(),
            @ErrorState = ERROR_STATE();

        RAISERROR(@ErrorMessage, @ErrorSeverity, @ErrorState);
    END CATCH
END
GO
