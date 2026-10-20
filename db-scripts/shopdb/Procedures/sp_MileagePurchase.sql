USE RanShop
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_mileage_purchase]
    @UserNum                BIGINT,
    @PurchaseItemsJson      NVARCHAR(MAX)
AS
BEGIN
    SET NOCOUNT ON;

    BEGIN TRY
        BEGIN TRANSACTION;

        DECLARE @CurrentDate            DATE = CAST(GETDATE() AS DATE);
        DECLARE @CurrentDateTime        DATETIME = GETDATE();
        DECLARE @CurrentYear            INT = YEAR(@CurrentDateTime);
        DECLARE @CurrentMonthYear       INT = @CurrentYear * 100 + MONTH(@CurrentDateTime);
        DECLARE @TransactionGUID        UNIQUEIDENTIFIER = NEWID();

        -- Parse JSON into a table variable
        DECLARE @PurchaseItems TABLE (
            ProductNum                  INT,
            Quantity                    INT,
            ItemMain                    INT,
            ItemSub                     INT,
            ItemName                    NVARCHAR(MAX),
            ItemCategory                SMALLINT,
            ItemDescription             NVARCHAR(MAX),
            ItemPrice                   DECIMAL(18,4),
            ItemStock                   INT,
            ItemStatus                  SMALLINT,

            ItemDiscount                DECIMAL(18,4) NULL,
            ItemDiscountType            SMALLINT NULL,

            PurchaseLimitPerAccount     INT NULL,
            PurchaseLimitType           SMALLINT NULL,

            DiscountedPrice             DECIMAL(18,4),
            LineTotal                   DECIMAL(18,4)
        );

        INSERT INTO @PurchaseItems (
            ProductNum, Quantity, ItemMain, ItemSub, ItemName, ItemCategory,
            ItemDescription, ItemPrice, ItemStock, ItemStatus,
            ItemDiscount, ItemDiscountType, PurchaseLimitPerAccount, PurchaseLimitType
        )
        SELECT
            P.productNum,
            P.quantity,
            I.ItemMain,
            I.ItemSub,
            I.ItemName,
            I.ItemCategory,
            I.ItemDescription,
            I.ItemPrice,
            I.ItemStock,
            I.ItemStatus,
            I.ItemDiscount,
            I.ItemDiscountType,
            I.PurchaseLimitPerAccount,
            I.PurchaseLimitType
        FROM dbo.YXShopItemMap I
        INNER JOIN OPENJSON(@PurchaseItemsJson)
            WITH(productNum INT 'strict $.product_num', quantity INT 'strict $.quantity') P
        ON P.productNum = I.ProductNum
        WHERE I.ShopType = 2;

        -- Check if any products were not found
        IF NOT EXISTS (SELECT 1 FROM @PurchaseItems)
        BEGIN
            RAISERROR('No valid mileage shop products found in purchase request', 16, 1);
            RETURN;
        END

        -- Calculate discounted prices
        UPDATE @PurchaseItems
        SET
            DiscountedPrice = CASE
                WHEN ItemDiscountType = 1 THEN
                    CASE WHEN ItemPrice - (ItemPrice * ISNULL(ItemDiscount, 0) / 100) < 0
                        THEN 0
                        ELSE ItemPrice - (ItemPrice * ISNULL(ItemDiscount, 0) / 100)
                    END
                WHEN ItemDiscountType = 2 THEN
                    CASE WHEN ItemPrice - ISNULL(ItemDiscount, 0) < 0
                        THEN 0
                        ELSE ItemPrice - ISNULL(ItemDiscount, 0)
                    END
                ELSE ISNULL(ItemPrice, 0)
            END,
            LineTotal = CASE
                WHEN ItemDiscountType = 1 THEN
                    CASE WHEN ItemPrice - (ItemPrice * ISNULL(ItemDiscount, 0) / 100) < 0
                        THEN 0
                        ELSE (ItemPrice - (ItemPrice * ISNULL(ItemDiscount, 0) / 100)) * Quantity
                    END
                WHEN ItemDiscountType = 2 THEN
                    CASE WHEN ItemPrice - ISNULL(ItemDiscount, 0) < 0
                        THEN 0
                        ELSE (ItemPrice - ISNULL(ItemDiscount, 0)) * Quantity
                    END
                ELSE ISNULL(ItemPrice, 0) * Quantity
            END;

        DECLARE @InvalidProducts VARCHAR(MAX);

        -- Validation 1: Check item status
        SET @InvalidProducts = NULL;
        SELECT @InvalidProducts = STRING_AGG(CAST(ProductNum AS VARCHAR), ',')
        FROM @PurchaseItems
        WHERE ItemStatus <> 1;

        IF @InvalidProducts IS NOT NULL
        BEGIN
            RAISERROR('Purchase failed: Inactive product(s): %s.', 16, 1, @InvalidProducts);
            RETURN;
        END

        -- Validation 2: Check stock availability
        SET @InvalidProducts = NULL;
        SELECT @InvalidProducts = STRING_AGG(
            CAST(ProductNum AS VARCHAR) + ' (requested: ' + CAST(Quantity AS VARCHAR) +
            ', available: ' + CAST(ItemStock AS VARCHAR) + ')',
            ', '
        )
        FROM @PurchaseItems
        WHERE Quantity > ItemStock;

        IF @InvalidProducts IS NOT NULL
        BEGIN
            RAISERROR('Purchase failed: Insufficient stock for product(s): %s.', 16, 1, @InvalidProducts);
            RETURN;
        END

        -- Calculate total cost
        DECLARE @TotalCost DECIMAL(18,4);
        SELECT @TotalCost = SUM(LineTotal) FROM @PurchaseItems

        -- Get current user mileage points
        DECLARE @CurrentMileagePoints   DECIMAL(18,4);
        DECLARE @UserName               NVARCHAR(25);
        DECLARE @UserID                 UNIQUEIDENTIFIER;

        SELECT @CurrentMileagePoints = MileagePoints, @UserName = UserName, @UserID = UserID
        FROM RanWeb.dbo.WebUser WITH (UPDLOCK)
        WHERE UserNum = @UserNum;

        IF @CurrentMileagePoints IS NULL
        BEGIN
            RAISERROR('User not found', 16, 1);
            RETURN;
        END

        -- Validation 3: Check if user has enough mileage points
        IF @CurrentMileagePoints < @TotalCost
        BEGIN
            DECLARE @CurrentPointsStr VARCHAR(20) = CAST(@CurrentMileagePoints AS VARCHAR(20));
            DECLARE @TotalCostStr VARCHAR(20) = CAST(@TotalCost AS VARCHAR(20));

            RAISERROR('Insufficient mileage points (Available: %s, Required: %s)', 16, 1, @CurrentPointsStr, @TotalCostStr);
            RETURN;
        END

        -- Validation 4: Check purchase limits (against MileagePurchaseSummary)
        SET @InvalidProducts = NULL;
        SELECT @InvalidProducts = STRING_AGG(
            CAST(P.ProductNum AS VARCHAR) + ' (' +
            CASE
                WHEN P.PurchaseLimitType = 0 THEN 'lifetime limit'
                WHEN P.PurchaseLimitType = 1 THEN 'yearly limit'
                WHEN P.PurchaseLimitType = 2 THEN 'monthly limit'
                WHEN P.PurchaseLimitType = 3 THEN 'daily limit'
                ELSE 'unknown limit'
            END + ')',
            ', '
        )
        FROM @PurchaseItems P
        LEFT JOIN dbo.MileagePurchaseSummary S
            ON S.UserNum = @UserNum AND S.ProductNum = P.ProductNum
        WHERE P.PurchaseLimitPerAccount IS NOT NULL
        AND P.PurchaseLimitType IS NOT NULL
        AND (
            (P.PurchaseLimitType = 0 AND
                (ISNULL(S.TotalQuantityPurchased, 0) + P.Quantity) > P.PurchaseLimitPerAccount)
            OR
            (P.PurchaseLimitType = 1 AND
                (CASE WHEN S.CurrentYear = @CurrentYear
                    THEN S.CurrentYearQuantity ELSE 0 END + P.Quantity) > P.PurchaseLimitPerAccount)
            OR
            (P.PurchaseLimitType = 2 AND
                (CASE WHEN S.CurrentMonthYear = @CurrentMonthYear
                    THEN S.CurrentMonthQuantity ELSE 0 END + P.Quantity) > P.PurchaseLimitPerAccount)
            OR
            (P.PurchaseLimitType = 3 AND
                (CASE WHEN S.CurrentDay = @CurrentDate
                    THEN S.CurrentDayQuantity ELSE 0 END + P.Quantity) > P.PurchaseLimitPerAccount)
        );

        IF @InvalidProducts IS NOT NULL
        BEGIN
            RAISERROR('Purchase failed: Limit reached for product(s): %s.', 16, 1, @InvalidProducts);
            RETURN;
        END

        -- Validation 5: Check for zero/negative/NULL calculated prices
        SET @InvalidProducts = NULL;
        SELECT @InvalidProducts = STRING_AGG(CAST(ProductNum AS VARCHAR), ',')
        FROM @PurchaseItems
        WHERE DiscountedPrice <= 0 OR DiscountedPrice IS NULL;

        IF @InvalidProducts IS NOT NULL
        BEGIN
            RAISERROR('Purchase failed: Invalid price for product(s): %s.', 16, 1, @InvalidProducts);
            RETURN;
        END

        -- All validations passed

        -- Update stock levels
        UPDATE I
        SET I.ItemStock = I.ItemStock - P.Quantity
        FROM RanShop.dbo.YXShopItemMap I WITH (UPDLOCK)
        INNER JOIN @PurchaseItems P ON I.ProductNum = P.ProductNum

        -- Deduct mileage points from user (NOT WebPoints)
        UPDATE RanWeb.dbo.WebUser
        SET MileagePoints = MileagePoints - @TotalCost
        WHERE UserID = @UserID;

        DECLARE @RemainingPoints DECIMAL(18,4);
        SET @RemainingPoints = @CurrentMileagePoints - @TotalCost;

        -- Insert mileage order header
        DECLARE @OrderID        BIGINT;
        DECLARE @TotalDiscount  DECIMAL(18,4);
        SELECT @TotalDiscount = SUM((ItemPrice - DiscountedPrice) * Quantity) FROM @PurchaseItems;

        INSERT INTO dbo.MileageOrder (
            UserNum, UserUID, TotalItems, TotalQuantity,
            TotalAmount, TotalDiscount, PointsBefore, PointsAfter, OrderStatus
        )
        VALUES (
            @UserNum, @UserName,
            (SELECT COUNT(*) FROM @PurchaseItems),
            (SELECT SUM(Quantity) FROM @PurchaseItems),
            @TotalCost, ISNULL(@TotalDiscount, 0),
            @CurrentMileagePoints, @RemainingPoints, 1
        );
        SET @OrderID = SCOPE_IDENTITY();

        -- Insert items into in-game user's bank (same ShopPurchase table for delivery)
        INSERT INTO ShopPurchase (UserUID, ProductNum, ItemName, OrderID)
        SELECT @UserName, p.ProductNum, p.ItemName, @OrderID
        FROM @PurchaseItems p
        JOIN dbo.PurchaseNumber n ON n.number <= p.Quantity;

        -- Insert mileage order items
        INSERT INTO dbo.MileageOrderItem (
            OrderID, UserNum, ProductNum, ItemName, ItemMain, ItemSub,
            Quantity, UnitPrice, DiscountType, DiscountValue, FinalUnitPrice, LineTotal
        )
        SELECT
            @OrderID, @UserNum, ProductNum, ItemName, ItemMain, ItemSub,
            Quantity, ItemPrice, ItemDiscountType, ItemDiscount, DiscountedPrice, LineTotal
        FROM @PurchaseItems;

        -- Update mileage purchase summary
        MERGE dbo.MileagePurchaseSummary AS target
        USING @PurchaseItems AS source
        ON target.UserNum = @UserNum AND target.ProductNum = source.ProductNum
        WHEN MATCHED THEN
            UPDATE SET
                TotalQuantityPurchased = target.TotalQuantityPurchased + source.Quantity,
                TotalAmountSpent = target.TotalAmountSpent + source.LineTotal,
                PurchaseCount = target.PurchaseCount + 1,
                LastPurchaseDate = @CurrentDateTime,

                CurrentYearQuantity =
                    CASE WHEN target.CurrentYear = @CurrentYear
                    THEN target.CurrentYearQuantity + source.Quantity
                    ELSE source.Quantity END,
                CurrentYear = @CurrentYear,
                CurrentYearSpent =
                    CASE WHEN target.CurrentYear = @CurrentYear
                    THEN target.CurrentYearSpent + source.LineTotal
                    ELSE source.LineTotal END,

                CurrentMonthQuantity =
                    CASE WHEN target.CurrentMonthYear = @CurrentMonthYear
                    THEN target.CurrentMonthQuantity + source.Quantity
                    ELSE source.Quantity END,
                CurrentMonthYear = @CurrentMonthYear,
                CurrentMonthSpent =
                    CASE WHEN target.CurrentMonthYear = @CurrentMonthYear
                    THEN target.CurrentMonthSpent + source.LineTotal
                    ELSE source.LineTotal END,

                CurrentDayQuantity =
                    CASE WHEN target.CurrentDay = @CurrentDate
                    THEN target.CurrentDayQuantity + source.Quantity
                    ELSE source.Quantity END,
                CurrentDay = @CurrentDate,
                CurrentDaySpent =
                    CASE WHEN target.CurrentDay = @CurrentDate
                    THEN target.CurrentDaySpent + source.LineTotal
                    ELSE source.LineTotal END,

                LastUpdated = @CurrentDateTime
        WHEN NOT MATCHED THEN
            INSERT (
                UserUID, UserNum, ProductNum,
                TotalQuantityPurchased, TotalAmountSpent, PurchaseCount,
                FirstPurchaseDate, LastPurchaseDate,
                CurrentYearQuantity, CurrentYear, CurrentYearSpent,
                CurrentMonthQuantity, CurrentMonthYear, CurrentMonthSpent,
                CurrentDayQuantity, CurrentDay, CurrentDaySpent,
                LastUpdated
            )
            VALUES (
                @UserID, @UserNum, source.ProductNum,
                source.Quantity, source.LineTotal, 1,
                @CurrentDateTime, @CurrentDateTime,
                source.Quantity, @CurrentYear, source.LineTotal,
                source.Quantity, @CurrentMonthYear, source.LineTotal,
                source.Quantity, @CurrentDate, source.LineTotal,
                @CurrentDateTime
            );

        COMMIT TRANSACTION;

        -- Return success (no mileage earned on mileage purchases)
        SELECT
            @OrderID AS OrderID,
            @TotalCost AS TotalCost,
            @RemainingPoints AS RemainingPoints,
            'Success' AS Status;
    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0
            ROLLBACK TRANSACTION;

        DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE(),
                @ErrorSeverity INT = ERROR_SEVERITY(),
                @ErrorState INT = ERROR_STATE();
        RAISERROR (@ErrorMessage, @ErrorSeverity, @ErrorState);
    END CATCH
END
GO
