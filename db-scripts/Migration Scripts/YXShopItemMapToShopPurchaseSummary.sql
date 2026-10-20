-- ============================================================================
-- MIGRATION SCRIPT: Populate ShopPurchaseSummary from Historical ShopPurchase Data
-- Using UserNum and UserUID (UNIQUEIDENTIFIER) from RanWeb.dbo.WebUser
-- All historical purchases will be counted towards LIFETIME limits (Type 0)
-- ============================================================================
-- Date: 2026-02-03
-- Table: ShopPurchase (PurKey, UserUID VARCHAR(20), ProductNum, PurDate)
-- Lookup: RanWeb.dbo.WebUser (UserNum, UserID UNIQUEIDENTIFIER, UserName VARCHAR)
-- ============================================================================

USE RanShop;
GO

SET NOCOUNT ON;
GO

PRINT '========================================================================';
PRINT 'Starting Migration: Historical Purchase Data to ShopPurchaseSummary';
PRINT 'Using UserNum and UserUID from RanWeb.dbo.WebUser';
PRINT 'All historical purchases will count as LIFETIME purchases (Type 0)';
PRINT 'Start Time: ' + CONVERT(VARCHAR, GETDATE(), 120);
PRINT '========================================================================';
PRINT '';

-- ============================================================================
-- STEP 1: Verify Tables and Columns Exist
-- ============================================================================
PRINT 'STEP 1: Verifying required tables and columns exist...';

IF OBJECT_ID('dbo.ShopPurchase', 'U') IS NULL
BEGIN
    PRINT 'ERROR: ShopPurchase table does not exist!';
    RETURN;
END

IF OBJECT_ID('dbo.ShopPurchaseSummary', 'U') IS NULL
BEGIN
    PRINT 'ERROR: ShopPurchaseSummary table does not exist!';
    PRINT 'Please run the table structure update script first.';
    RETURN;
END

IF OBJECT_ID('dbo.YXShopItemMap', 'U') IS NULL
BEGIN
    PRINT 'ERROR: YXShopItemMap table does not exist!';
    RETURN;
END

IF OBJECT_ID('RanWeb.dbo.WebUser', 'U') IS NULL
BEGIN
    PRINT 'ERROR: RanWeb.dbo.WebUser table does not exist!';
    RETURN;
END

-- -- Verify WebUser table structure
-- IF NOT EXISTS (
--     SELECT 1 FROM sys.columns
--     WHERE object_id = OBJECT_ID('RanWeb.dbo.WebUser')
--     AND name IN ('UserNum', 'UserID', 'UserName')
--     HAVING COUNT(*) = 3
-- )
-- BEGIN
--     PRINT 'ERROR: RanWeb.dbo.WebUser missing required columns (UserNum, UserID, UserName)!';
--     RETURN;
-- END

PRINT 'All required tables and columns exist.';
PRINT '';

-- ============================================================================
-- STEP 2: Verify ShopPurchase Table Structure
-- ============================================================================
PRINT 'STEP 2: Verifying ShopPurchase table structure...';

DECLARE @MissingColumns TABLE (ColumnName VARCHAR(50));

IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID('dbo.ShopPurchase') AND name = 'PurKey')
    INSERT INTO @MissingColumns VALUES ('PurKey');

IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID('dbo.ShopPurchase') AND name = 'UserUID')
    INSERT INTO @MissingColumns VALUES ('UserUID');

IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID('dbo.ShopPurchase') AND name = 'ProductNum')
    INSERT INTO @MissingColumns VALUES ('ProductNum');

IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID('dbo.ShopPurchase') AND name = 'PurDate')
    INSERT INTO @MissingColumns VALUES ('PurDate');

IF EXISTS (SELECT 1 FROM @MissingColumns)
BEGIN
    PRINT 'ERROR: ShopPurchase table is missing required columns:';
    SELECT ColumnName FROM @MissingColumns;
    RETURN;
END

PRINT 'ShopPurchase table structure verified.';
PRINT '';

-- ============================================================================
-- STEP 3: Check Data Statistics
-- ============================================================================
PRINT 'STEP 3: Gathering statistics on existing data...';

DECLARE @TotalHistoricalRecords INT;
DECLARE @RecordsWithNullProductNum INT;
DECLARE @RecordsWithNullUserUID INT;
DECLARE @RecordsWithUnknownUser INT;
DECLARE @UniqueUsers INT;
DECLARE @UniqueProducts INT;
DECLARE @ItemsWithLimits INT;
DECLARE @ExistingSummaryRecords INT;
DECLARE @OldestPurchase DATETIME;
DECLARE @NewestPurchase DATETIME;

SELECT @TotalHistoricalRecords = COUNT(*) FROM dbo.ShopPurchase;
SELECT @RecordsWithNullProductNum = COUNT(*) FROM dbo.ShopPurchase WHERE ProductNum IS NULL;
SELECT @RecordsWithNullUserUID = COUNT(*) FROM dbo.ShopPurchase WHERE UserUID IS NULL OR LTRIM(RTRIM(UserUID)) = '';

-- Check for users not in WebUser table
SELECT @RecordsWithUnknownUser = COUNT(*)
FROM dbo.ShopPurchase SP
LEFT JOIN RanWeb.dbo.WebUser WU ON SP.UserUID = WU.UserName
WHERE WU.UserName IS NULL
    AND SP.UserUID IS NOT NULL
    AND LTRIM(RTRIM(SP.UserUID)) <> '';

SELECT @UniqueUsers = COUNT(DISTINCT UserUID)
FROM dbo.ShopPurchase
WHERE UserUID IS NOT NULL
    AND EXISTS (SELECT 1 FROM RanWeb.dbo.WebUser WHERE UserName = ShopPurchase.UserUID);

SELECT @UniqueProducts = COUNT(DISTINCT ProductNum)
FROM dbo.ShopPurchase
WHERE ProductNum IS NOT NULL;

SELECT @ItemsWithLimits = COUNT(*)
FROM dbo.YXShopItemMap
WHERE PurchaseLimitPerAccount IS NOT NULL;

SELECT @ExistingSummaryRecords = COUNT(*) FROM dbo.ShopPurchaseSummary;

SELECT @OldestPurchase = MIN(PurDate), @NewestPurchase = MAX(PurDate)
FROM dbo.ShopPurchase;

PRINT 'Data Statistics:';
PRINT '  - Historical ShopPurchase Records: ' + CAST(@TotalHistoricalRecords AS VARCHAR);
PRINT '  - Records with NULL ProductNum: ' + CAST(@RecordsWithNullProductNum AS VARCHAR);
PRINT '  - Records with NULL/Empty UserUID: ' + CAST(@RecordsWithNullUserUID AS VARCHAR);
PRINT '  - Records with Unknown Users: ' + CAST(@RecordsWithUnknownUser AS VARCHAR);
PRINT '  - Valid Unique Users (in WebUser): ' + CAST(@UniqueUsers AS VARCHAR);
PRINT '  - Unique Products Purchased: ' + CAST(@UniqueProducts AS VARCHAR);
PRINT '  - Items with Purchase Limits: ' + CAST(@ItemsWithLimits AS VARCHAR);
PRINT '  - Existing Summary Records: ' + CAST(@ExistingSummaryRecords AS VARCHAR);
PRINT '  - Date Range: ' + ISNULL(CONVERT(VARCHAR, @OldestPurchase, 120), 'N/A') +
      ' to ' + ISNULL(CONVERT(VARCHAR, @NewestPurchase, 120), 'N/A');
PRINT '';

IF @TotalHistoricalRecords = 0
BEGIN
    PRINT 'WARNING: No historical purchase records found.';
    PRINT '';
END

IF @RecordsWithUnknownUser > 0
BEGIN
    PRINT 'WARNING: ' + CAST(@RecordsWithUnknownUser AS VARCHAR) +
          ' records have UserUIDs not found in RanWeb.dbo.WebUser.';
    PRINT 'These records will be SKIPPED during migration.';
    PRINT '';

    -- Show sample of unknown users
    PRINT 'Sample of unknown users (first 10):';
    SELECT DISTINCT TOP 10 SP.UserUID
    FROM dbo.ShopPurchase SP
    LEFT JOIN RanWeb.dbo.WebUser WU ON SP.UserUID = WU.UserName
    WHERE WU.UserName IS NULL
        AND SP.UserUID IS NOT NULL
        AND LTRIM(RTRIM(SP.UserUID)) <> '';
    PRINT '';
END

-- ============================================================================
-- STEP 4: Backup Existing Summary Data (if any)
-- ============================================================================
PRINT 'STEP 4: Creating backup of existing ShopPurchaseSummary data...';

IF @ExistingSummaryRecords > 0
BEGIN
    DECLARE @BackupTableName NVARCHAR(200) = 'ShopPurchaseSummary_Backup_' +
        REPLACE(REPLACE(REPLACE(CONVERT(VARCHAR, GETDATE(), 120), ' ', '_'), ':', ''), '-', '');

    DECLARE @BackupSQL NVARCHAR(MAX) =
        'SELECT * INTO dbo.' + @BackupTableName + ' FROM dbo.ShopPurchaseSummary';

    EXEC sp_executesql @BackupSQL;

    PRINT 'Backup created: ' + @BackupTableName;
    PRINT 'Records backed up: ' + CAST(@ExistingSummaryRecords AS VARCHAR);
END
ELSE
BEGIN
    PRINT 'No existing records to backup.';
END
PRINT '';

-- ============================================================================
-- STEP 5: Show Items with Limits
-- ============================================================================
PRINT 'STEP 5: Items with purchase limits that will be affected (Top 20):';
PRINT '';

SELECT TOP 20
    M.ProductNum,
    M.ItemName,
    M.PurchaseLimitPerAccount,
    CASE ISNULL(M.PurchaseLimitType, 999)
        WHEN 0 THEN 'Lifetime'
        WHEN 1 THEN 'Yearly'
        WHEN 2 THEN 'Monthly'
        WHEN 3 THEN 'Daily'
        WHEN 999 THEN 'NULL (No Type)'
        ELSE 'Unknown'
    END AS LimitType,
    COUNT(SP.PurKey) AS HistoricalPurchases,
    COUNT(DISTINCT SP.UserUID) AS UniqueUsers
FROM dbo.YXShopItemMap M
LEFT JOIN dbo.ShopPurchase SP ON M.ProductNum = SP.ProductNum
WHERE M.PurchaseLimitPerAccount IS NOT NULL
GROUP BY M.ProductNum, M.ItemName, M.PurchaseLimitPerAccount, M.PurchaseLimitType
ORDER BY HistoricalPurchases DESC;

PRINT '';

-- ============================================================================
-- STEP 6: Create Temporary Staging Table
-- ============================================================================
PRINT 'STEP 6: Creating temporary staging table...';

IF OBJECT_ID('tempdb..#MigrationStaging', 'U') IS NOT NULL
    DROP TABLE #MigrationStaging;

CREATE TABLE #MigrationStaging (
    UserNum INT NOT NULL,
    UserUID UNIQUEIDENTIFIER NOT NULL,
    UserName VARCHAR(255) NOT NULL,
    ProductNum INT NOT NULL,
    TotalQuantityPurchased INT NOT NULL,
    PurchaseCount INT NOT NULL,
    FirstPurchaseDate DATETIME NOT NULL,
    LastPurchaseDate DATETIME NOT NULL,
    PRIMARY KEY (UserNum, ProductNum)
);

PRINT 'Staging table created successfully.';
PRINT '';

-- ============================================================================
-- STEP 7: Extract and Aggregate Historical Purchase Data
-- ============================================================================
PRINT 'STEP 7: Extracting and aggregating historical purchase data...';
PRINT 'Joining ShopPurchase with RanWeb.dbo.WebUser to get UserNum and UserID...';
PRINT 'This may take a while for large datasets...';
PRINT '';

DECLARE @CurrentDate DATE = CAST(GETDATE() AS DATE);
DECLARE @CurrentYear INT = YEAR(GETDATE());
DECLARE @CurrentMonthYear INT = YEAR(GETDATE()) * 100 + MONTH(GETDATE());

BEGIN TRY
    -- Aggregate purchases by user and product, joining with WebUser
    INSERT INTO #MigrationStaging (
        UserNum,
        UserUID,
        UserName,
        ProductNum,
        TotalQuantityPurchased,
        PurchaseCount,
        FirstPurchaseDate,
        LastPurchaseDate
    )
    SELECT
        WU.UserNum,
        WU.UserID AS UserUID,
        WU.UserName,
        SP.ProductNum,
        COUNT(*) AS TotalQuantityPurchased,
        COUNT(DISTINCT CAST(SP.PurDate AS DATE)) AS PurchaseCount,
        MIN(SP.PurDate) AS FirstPurchaseDate,
        MAX(SP.PurDate) AS LastPurchaseDate
    FROM dbo.ShopPurchase SP
    INNER JOIN RanWeb.dbo.WebUser WU ON SP.UserUID = WU.UserName
    INNER JOIN dbo.YXShopItemMap IM ON SP.ProductNum = IM.ProductNum
    WHERE IM.PurchaseLimitPerAccount IS NOT NULL
        AND SP.ProductNum IS NOT NULL
        AND SP.UserUID IS NOT NULL
        AND LTRIM(RTRIM(SP.UserUID)) <> ''
    GROUP BY WU.UserNum, WU.UserID, WU.UserName, SP.ProductNum;

    DECLARE @StagingRecords INT = @@ROWCOUNT;
    PRINT 'Successfully aggregated ' + CAST(@StagingRecords AS VARCHAR) + ' user-product combinations.';
    PRINT '';

    IF @StagingRecords = 0
    BEGIN
        PRINT 'WARNING: No records were aggregated. Possible reasons:';
        PRINT '  - No historical purchases for items with limits';
        PRINT '  - ProductNum or UserUID is NULL in ShopPurchase';
        PRINT '  - UserUIDs in ShopPurchase not found in RanWeb.dbo.WebUser';
        PRINT '  - No items have PurchaseLimitPerAccount set';
        PRINT '';
    END

END TRY
BEGIN CATCH
    PRINT 'ERROR during data aggregation:';
    PRINT ERROR_MESSAGE();
    PRINT 'Error Line: ' + CAST(ERROR_LINE() AS VARCHAR);
    PRINT '';
    RETURN;
END CATCH

-- ============================================================================
-- STEP 8: Display Sample of Migrated Data
-- ============================================================================
PRINT 'STEP 8: Sample of data to be migrated (Top 20 records):';
PRINT '';

SELECT TOP 20
    MS.UserName,
    MS.UserNum,
    CAST(MS.UserUID AS VARCHAR(36)) AS UserUID,
    MS.ProductNum,
    IM.ItemName,
    MS.TotalQuantityPurchased,
    IM.PurchaseLimitPerAccount AS [Limit],
    CASE
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount
        THEN '🚫 BLOCKED'
        ELSE CAST(IM.PurchaseLimitPerAccount - MS.TotalQuantityPurchased AS VARCHAR) + ' left'
    END AS Status,
    MS.PurchaseCount AS TxnDays,
    CONVERT(VARCHAR, MS.FirstPurchaseDate, 120) AS FirstPurchase,
    CONVERT(VARCHAR, MS.LastPurchaseDate, 120) AS LastPurchase
FROM #MigrationStaging MS
INNER JOIN dbo.YXShopItemMap IM ON MS.ProductNum = IM.ProductNum
ORDER BY MS.TotalQuantityPurchased DESC;

PRINT '';

-- ============================================================================
-- STEP 9: Show Users Who Will Hit Their Limits
-- ============================================================================
PRINT 'STEP 9: Users who have reached or exceeded their limits:';
PRINT '';

DECLARE @BlockedCount INT;

SELECT @BlockedCount = COUNT(*)
FROM #MigrationStaging MS
INNER JOIN dbo.YXShopItemMap IM ON MS.ProductNum = IM.ProductNum
WHERE MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount;

IF @BlockedCount > 0
BEGIN
    SELECT TOP 30
        MS.UserName,
        MS.UserNum,
        MS.ProductNum,
        IM.ItemName,
        MS.TotalQuantityPurchased AS Purchased,
        IM.PurchaseLimitPerAccount AS [Limit],
        MS.TotalQuantityPurchased - IM.PurchaseLimitPerAccount AS OverLimit,
        CONVERT(VARCHAR, MS.LastPurchaseDate, 120) AS LastPurchase
    FROM #MigrationStaging MS
    INNER JOIN dbo.YXShopItemMap IM ON MS.ProductNum = IM.ProductNum
    WHERE MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount
    ORDER BY OverLimit DESC, MS.UserName;

    PRINT '';
    PRINT 'Total user-product combinations BLOCKED: ' + CAST(@BlockedCount AS VARCHAR);
    PRINT '';
    PRINT '⚠️  WARNING: These users will NOT be able to purchase these items!';
END
ELSE
BEGIN
    PRINT '✓ No users have exceeded their limits.';
END

PRINT '';

-- ============================================================================
-- STEP 10: Summary Statistics
-- ============================================================================
PRINT 'STEP 10: Pre-Migration Summary Statistics';
PRINT '';

PRINT '--- Purchase Distribution by Limit Usage ---';
SELECT
    CASE
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 2 THEN '5. Over 200%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount THEN '4. 100-200%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.75 THEN '3. 75-99%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.5 THEN '2. 50-74%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.25 THEN '1. 25-49%'
        ELSE '0. Under 25%'
    END AS UsageRange,
    COUNT(*) AS UserProductCombos,
    SUM(MS.TotalQuantityPurchased) AS TotalItems
FROM #MigrationStaging MS
INNER JOIN dbo.YXShopItemMap IM ON MS.ProductNum = IM.ProductNum
GROUP BY
    CASE
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 2 THEN '5. Over 200%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount THEN '4. 100-200%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.75 THEN '3. 75-99%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.5 THEN '2. 50-74%'
        WHEN MS.TotalQuantityPurchased >= IM.PurchaseLimitPerAccount * 0.25 THEN '1. 25-49%'
        ELSE '0. Under 25%'
    END
ORDER BY UsageRange DESC;

PRINT '';

-- ============================================================================
-- STEP 11: Confirmation Prompt
-- ============================================================================
DECLARE @StagingCount INT;
SELECT @StagingCount = COUNT(*) FROM #MigrationStaging;

PRINT '========================================================================';
PRINT 'READY TO MIGRATE';
PRINT '========================================================================';
PRINT 'Migration will:';
PRINT '  • Process ' + CAST(@StagingCount AS VARCHAR) + ' user-product combinations';
PRINT '  • Use UserNum and UserID from RanWeb.dbo.WebUser';
PRINT '  • Count all historical purchases as LIFETIME limits (Type 0)';
PRINT '  • Block ' + CAST(@BlockedCount AS VARCHAR) + ' combinations (already at limit)';
PRINT '';
PRINT 'Historical purchases only affect LIFETIME limits.';
PRINT 'Yearly/Monthly/Daily limits start fresh.';
PRINT '';
PRINT 'Press Ctrl+C NOW to abort. Continuing in 5 seconds...';
PRINT '';

WAITFOR DELAY '00:00:05';

-- ============================================================================
-- STEP 12: Merge into ShopPurchaseSummary
-- ============================================================================
PRINT 'STEP 12: Merging historical data into ShopPurchaseSummary...';

DECLARE @InsertedCount INT = 0;
DECLARE @MergeStartTime DATETIME = GETDATE();

BEGIN TRY
    BEGIN TRANSACTION;

    MERGE dbo.ShopPurchaseSummary AS target
    USING #MigrationStaging AS source
    ON target.UserNum = source.UserNum AND target.ProductNum = source.ProductNum
    WHEN MATCHED THEN
        UPDATE SET
            -- Keep UserUID updated
            UserUID = source.UserUID,

            -- Add historical lifetime purchases
            TotalQuantityPurchased = target.TotalQuantityPurchased + source.TotalQuantityPurchased,
            TotalAmountSpent = target.TotalAmountSpent,
            PurchaseCount = target.PurchaseCount + source.PurchaseCount,

            -- Update dates
            FirstPurchaseDate = CASE
                WHEN source.FirstPurchaseDate < target.FirstPurchaseDate
                THEN source.FirstPurchaseDate
                ELSE target.FirstPurchaseDate
            END,
            LastPurchaseDate = CASE
                WHEN source.LastPurchaseDate > target.LastPurchaseDate
                THEN source.LastPurchaseDate
                ELSE target.LastPurchaseDate
            END,

            -- Keep period counters as-is
            CurrentYearQuantity = target.CurrentYearQuantity,
            CurrentYear = target.CurrentYear,
            CurrentYearSpent = target.CurrentYearSpent,
            CurrentMonthQuantity = target.CurrentMonthQuantity,
            CurrentMonthYear = target.CurrentMonthYear,
            CurrentMonthSpent = target.CurrentMonthSpent,
            CurrentDayQuantity = target.CurrentDayQuantity,
            CurrentDay = target.CurrentDay,
            CurrentDaySpent = target.CurrentDaySpent,

            LastUpdated = GETDATE()
    WHEN NOT MATCHED THEN
        INSERT (
            UserNum,
            UserUID,
            ProductNum,
            TotalQuantityPurchased,
            TotalAmountSpent,
            PurchaseCount,
            FirstPurchaseDate,
            LastPurchaseDate,
            CurrentYearQuantity,
            CurrentYear,
            CurrentYearSpent,
            CurrentMonthQuantity,
            CurrentMonthYear,
            CurrentMonthSpent,
            CurrentDayQuantity,
            CurrentDay,
            CurrentDaySpent,
            LastUpdated
        )
        VALUES (
            source.UserNum,
            source.UserUID,
            source.ProductNum,
            source.TotalQuantityPurchased,
            0, -- No amount data in ShopPurchase
            source.PurchaseCount,
            source.FirstPurchaseDate,
            source.LastPurchaseDate,
            0, @CurrentYear, 0,
            0, @CurrentMonthYear, 0,
            0, @CurrentDate, 0,
            GETDATE()
        );

    SET @InsertedCount = @@ROWCOUNT;

    COMMIT TRANSACTION;

    DECLARE @MergeEndTime DATETIME = GETDATE();
    DECLARE @MergeDuration INT = DATEDIFF(SECOND, @MergeStartTime, @MergeEndTime);

    PRINT 'Successfully processed ' + CAST(@InsertedCount AS VARCHAR) +
          ' records in ' + CAST(@MergeDuration AS VARCHAR) + ' seconds.';
    PRINT '';

END TRY
BEGIN CATCH
    IF @@TRANCOUNT > 0
        ROLLBACK TRANSACTION;

    PRINT '========================================================================';
    PRINT 'ERROR OCCURRED!';
    PRINT '========================================================================';
    PRINT 'Error: ' + ERROR_MESSAGE();
    PRINT 'Line: ' + CAST(ERROR_LINE() AS VARCHAR);
    PRINT '';
    PRINT 'Migration aborted. Changes rolled back.';
    RETURN;
END CATCH

-- ============================================================================
-- STEP 13: Verification
-- ============================================================================
PRINT 'STEP 13: Post-Migration Verification';
PRINT '';

DECLARE @FinalRecords INT, @TotalQty BIGINT, @UsersAffected INT, @ProductsAffected INT, @Blocked INT;

SELECT
    @FinalRecords = COUNT(*),
    @TotalQty = SUM(TotalQuantityPurchased),
    @UsersAffected = COUNT(DISTINCT UserNum),
    @ProductsAffected = COUNT(DISTINCT ProductNum)
FROM dbo.ShopPurchaseSummary;

SELECT @Blocked = COUNT(*)
FROM dbo.ShopPurchaseSummary S
INNER JOIN dbo.YXShopItemMap M ON S.ProductNum = M.ProductNum
WHERE M.PurchaseLimitPerAccount IS NOT NULL
    AND ISNULL(M.PurchaseLimitType, 0) = 0
    AND S.TotalQuantityPurchased >= M.PurchaseLimitPerAccount;

PRINT '========================================================================';
PRINT 'MIGRATION COMPLETED SUCCESSFULLY!';
PRINT '========================================================================';
PRINT '';
PRINT 'STATISTICS:';
PRINT '  • Total Summary Records: ' + CAST(@FinalRecords AS VARCHAR);
PRINT '  • Total Items Counted: ' + CAST(@TotalQty AS VARCHAR);
PRINT '  • Users Affected: ' + CAST(@UsersAffected AS VARCHAR);
PRINT '  • Products Affected: ' + CAST(@ProductsAffected AS VARCHAR);
PRINT '  • Combinations Blocked: ' + CAST(@Blocked AS VARCHAR);
PRINT '';

-- ============================================================================
-- STEP 14: Detailed Reports
-- ============================================================================
PRINT 'STEP 14: Post-Migration Reports';
PRINT '';

PRINT '--- Top 15 Users by Historical Purchases ---';
SELECT TOP 15
    WU.UserName,
    S.UserNum,
    COUNT(DISTINCT S.ProductNum) AS LimitedItems,
    SUM(S.TotalQuantityPurchased) AS TotalPurchased,
    MAX(S.LastPurchaseDate) AS LastPurchase,
    SUM(CASE WHEN S.TotalQuantityPurchased >= M.PurchaseLimitPerAccount THEN 1 ELSE 0 END) AS ItemsBlocked
FROM dbo.ShopPurchaseSummary S
INNER JOIN RanWeb.dbo.WebUser WU ON S.UserNum = WU.UserNum
INNER JOIN dbo.YXShopItemMap M ON S.ProductNum = M.ProductNum
WHERE M.PurchaseLimitPerAccount IS NOT NULL
GROUP BY WU.UserName, S.UserNum
ORDER BY TotalPurchased DESC;

PRINT '';

PRINT '--- Top 15 Items by Historical Purchases ---';
SELECT TOP 15
    M.ProductNum,
    M.ItemName,
    M.PurchaseLimitPerAccount,
    SUM(S.TotalQuantityPurchased) AS TotalPurchased,
    COUNT(DISTINCT S.UserNum) AS UniqueUsers,
    COUNT(CASE WHEN S.TotalQuantityPurchased >= M.PurchaseLimitPerAccount THEN 1 END) AS UsersBlocked
FROM dbo.ShopPurchaseSummary S
INNER JOIN dbo.YXShopItemMap M ON S.ProductNum = M.ProductNum
WHERE M.PurchaseLimitPerAccount IS NOT NULL
GROUP BY M.ProductNum, M.ItemName, M.PurchaseLimitPerAccount
ORDER BY TotalPurchased DESC;

PRINT '';

-- ============================================================================
-- STEP 15: Cleanup
-- ============================================================================
PRINT 'STEP 15: Cleanup...';
DROP TABLE #MigrationStaging;
PRINT 'Temporary table dropped.';
PRINT '';

PRINT '========================================================================';
PRINT 'MIGRATION COMPLETE';
PRINT 'Completed: ' + CONVERT(VARCHAR, GETDATE(), 120);
PRINT '========================================================================';
PRINT '';
PRINT 'NEXT STEPS:';
PRINT '  1. Test: EXEC sp_get_user_purchase_summary @UserName=''testuser''';
PRINT '  2. Test: EXEC sp_get_shop_items_with_limits @UserName=''testuser''';
PRINT '  3. Test purchase for blocked user (should fail)';
PRINT '  4. Test purchase for valid user (should succeed)';
PRINT '';
PRINT '========================================================================';

GO
