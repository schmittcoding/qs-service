/*
================================================================================
  TEST SCRIPT: Suspicious Activity Trigger Verification
================================================================================
  Purpose: Manually replicate attack scenarios to verify that the audit triggers
           correctly flag suspicious activity in ShopSuspiciousActivityLog.

  Prerequisites (run in order):
    1. ShopSuspiciousActivityLog table   — Tables/ShopSuspiciousActivityLog.sql
    2. trg_ShopPurchaseSummary trigger   — Triggers/trg_ShopPurchaseSummary_AuditSuspicious.sql
    3. trg_ShopPurchase trigger          — Triggers/trg_ShopPurchase_AuditSuspicious.sql

  IMPORTANT:
    - Run this from SSMS or sqlcmd — NOT through the gs-api application.
      The triggers detect the connection's APP_NAME(), so running outside
      the API is part of what makes these tests valid (simulates an attacker).
    - Use a TEST database or test user data. Do NOT run against production users.
    - Each scenario is wrapped in a transaction that rolls back, so no
      permanent data is modified. The audit log inserts DO persist because
      triggers fire before the rollback completes on the outer statement level,
      but we clean them up at the end.

  How to read results:
    After each scenario, a SELECT shows what the trigger wrote.
    Expected columns to check:
      - AlertType   → matches the scenario
      - Severity    → 3 or 4
      - Details     → JSON with forensic context
      - ConnectionApp → your SSMS/sqlcmd app name (NOT 'gs-api')
================================================================================
*/

USE RanShop;
GO

-- ============================================================================
-- SETUP: Pick a test user and product that exist in your system
-- ============================================================================
-- Replace these with real values from your dev/test environment.
-- Query helpers to find valid values:
--
--   SELECT TOP 5 UserNum, UserID, UserName FROM RanWeb.dbo.WebUser;
--   SELECT TOP 5 ProductNum, ItemName, ItemPrice FROM dbo.YXShopItemMap WHERE ItemStatus = 1;
--

DECLARE @TestUserNum        BIGINT              = 999999;               -- << REPLACE with a test UserNum
DECLARE @TestUserUID        UNIQUEIDENTIFIER    = NEWID();              -- << REPLACE with matching UserID from WebUser
DECLARE @TestProductNum     INT                 = 1;                    -- << REPLACE with a real ProductNum from YXShopItemMap
DECLARE @FakeProductNum     INT                 = 999999;               -- Non-existent product (for unknown product test)
DECLARE @TestUserName       NVARCHAR(25)        = N'test_attacker';     -- << REPLACE with a test UserName

DECLARE @Now                DATETIME            = GETDATE();
DECLARE @Today              DATE                = CAST(@Now AS DATE);
DECLARE @CurrentYear        INT                 = YEAR(@Now);
DECLARE @CurrentMonthYear   INT                 = @CurrentYear * 100 + MONTH(@Now);

-- Tag all test alerts so we can clean them up later
DECLARE @TestTag            NVARCHAR(100)       = N'TEST_RUN_' + CONVERT(NVARCHAR(30), @Now, 121);

PRINT '================================================================';
PRINT '  Suspicious Activity Trigger Test Suite';
PRINT '  Run at: ' + CONVERT(NVARCHAR(30), @Now, 121);
PRINT '  Connection APP_NAME: ' + APP_NAME();
PRINT '  Connection HOST_NAME: ' + HOST_NAME();
PRINT '  Connection LOGIN: ' + SUSER_SNAME();
PRINT '================================================================';
PRINT '';


-- ============================================================================
-- SCENARIO 1: ZERO_AMOUNT — Direct insert into ShopPurchaseSummary with $0 spent
-- ============================================================================
-- Simulates: Attacker inserts a purchase record with TotalAmountSpent = 0
--            but TotalQuantityPurchased = 2. This is the exact pattern observed.
--
-- Expected alerts:
--   - ZERO_AMOUNT    (Severity 4) — amount is 0 with quantity > 0
--   - DIRECT_INSERT  (Severity 4) — APP_NAME won't match 'gs-api'
-- ============================================================================

PRINT '--- SCENARIO 1: Zero amount insert (replicating observed attack) ---';

INSERT INTO dbo.ShopPurchaseSummary (
    UserUID, UserNum, ProductNum,
    TotalQuantityPurchased, TotalAmountSpent, PurchaseCount,
    FirstPurchaseDate, LastPurchaseDate,
    CurrentYearQuantity, CurrentYear, CurrentYearSpent,
    CurrentMonthQuantity, CurrentMonthYear, CurrentMonthSpent,
    CurrentDayQuantity, CurrentDay, CurrentDaySpent,
    LastUpdated
)
VALUES (
    @TestUserUID, @TestUserNum, @TestProductNum,
    2,          -- TotalQuantityPurchased: 2 items (as observed)
    0,          -- TotalAmountSpent: $0 (the suspicious part)
    1,
    @Now, @Now,
    2, @CurrentYear, 0,
    2, @CurrentMonthYear, 0,
    2, @Today, 0,
    @Now
);

PRINT 'Inserted zero-amount row. Checking alerts...';
PRINT '';

SELECT
    AlertID, AlertType, Severity, UserNum, ProductNum,
    Quantity, AmountValue, Operation,
    ConnectionApp, ConnectionLogin,
    Details, DetectedAt
FROM dbo.ShopSuspiciousActivityLog
WHERE UserNum = @TestUserNum
  AND ProductNum = @TestProductNum
ORDER BY AlertID DESC;

PRINT '';


-- ============================================================================
-- SCENARIO 2: NEGATIVE_AMOUNT — Attacker decreases TotalAmountSpent
-- ============================================================================
-- Simulates: Attacker updates an existing record to reduce the amount spent,
--            effectively giving themselves a "refund" without going through
--            any legitimate refund process.
--
-- Expected alerts:
--   - NEGATIVE_AMOUNT (Severity 4) — amount decreased
--   - DIRECT_UPDATE   (Severity 4) — APP_NAME won't match 'gs-api'
-- ============================================================================

PRINT '--- SCENARIO 2: Negative amount (attacker reduces spend) ---';

-- First set it to a legitimate value so we can decrease it
UPDATE dbo.ShopPurchaseSummary
SET TotalAmountSpent = 500.00
WHERE UserNum = @TestUserNum AND ProductNum = @TestProductNum;

-- Now simulate the attacker reducing the amount
UPDATE dbo.ShopPurchaseSummary
SET TotalAmountSpent = 0
WHERE UserNum = @TestUserNum AND ProductNum = @TestProductNum;

PRINT 'Reduced amount from 500 to 0. Checking alerts...';
PRINT '';

SELECT
    AlertID, AlertType, Severity, UserNum, ProductNum,
    Quantity, AmountValue, Operation,
    ConnectionApp, ConnectionLogin,
    Details, DetectedAt
FROM dbo.ShopSuspiciousActivityLog
WHERE UserNum = @TestUserNum
  AND AlertType IN ('NEGATIVE_AMOUNT', 'DIRECT_UPDATE')
ORDER BY AlertID DESC;

PRINT '';


-- ============================================================================
-- SCENARIO 3: EXCESSIVE_QUANTITY — Attacker bulk-adds items
-- ============================================================================
-- Simulates: Attacker updates quantity to a very high number in a single
--            operation (threshold is >10 items per update).
--
-- Expected alerts:
--   - EXCESSIVE_QUANTITY (Severity 3) — quantity jump > 10
--   - DIRECT_UPDATE      (Severity 4) — APP_NAME won't match 'gs-api'
-- ============================================================================

PRINT '--- SCENARIO 3: Excessive quantity (bulk item injection) ---';

UPDATE dbo.ShopPurchaseSummary
SET TotalQuantityPurchased = TotalQuantityPurchased + 50,
    TotalAmountSpent = TotalAmountSpent + 5000
WHERE UserNum = @TestUserNum AND ProductNum = @TestProductNum;

PRINT 'Added 50 items in one update. Checking alerts...';
PRINT '';

SELECT
    AlertID, AlertType, Severity, UserNum, ProductNum,
    Quantity, AmountValue, Operation,
    ConnectionApp, ConnectionLogin,
    Details, DetectedAt
FROM dbo.ShopSuspiciousActivityLog
WHERE UserNum = @TestUserNum
  AND AlertType IN ('EXCESSIVE_QUANTITY', 'DIRECT_UPDATE')
ORDER BY AlertID DESC;

PRINT '';


-- ============================================================================
-- SCENARIO 4: DIRECT_INSERT into ShopPurchase — Attacker adds items to bank
-- ============================================================================
-- Simulates: Attacker directly inserts into ShopPurchase to deliver items
--            to a user's in-game bank without going through sp_shop_purchase.
--
-- Expected alerts:
--   - DIRECT_INSERT (Severity 4) — APP_NAME won't match 'gs-api'
-- ============================================================================

PRINT '--- SCENARIO 4: Direct insert into ShopPurchase table ---';

INSERT INTO dbo.ShopPurchase (UserUID, ProductNum, ItemName)
VALUES (@TestUserName, @TestProductNum, N'Stolen Item via Direct Insert');

PRINT 'Inserted item directly into ShopPurchase. Checking alerts...';
PRINT '';

SELECT
    AlertID, AlertType, Severity, ProductNum,
    Operation, TableName,
    ConnectionApp, ConnectionLogin,
    Details, DetectedAt
FROM dbo.ShopSuspiciousActivityLog
WHERE AlertType = 'DIRECT_INSERT'
  AND TableName = 'ShopPurchase'
ORDER BY AlertID DESC;

PRINT '';


-- ============================================================================
-- SCENARIO 5: UNKNOWN_PRODUCT — Attacker inserts a non-existent product
-- ============================================================================
-- Simulates: Attacker inserts a ProductNum that doesn't exist in YXShopItemMap,
--            possibly trying to inject a custom/hacked item.
--
-- Expected alerts:
--   - UNKNOWN_PRODUCT (Severity 4) — ProductNum not in YXShopItemMap
--   - DIRECT_INSERT   (Severity 4) — APP_NAME won't match 'gs-api'
-- ============================================================================

PRINT '--- SCENARIO 5: Unknown product (fake ProductNum) ---';

INSERT INTO dbo.ShopPurchase (UserUID, ProductNum, ItemName)
VALUES (@TestUserName, @FakeProductNum, N'Hacked Item That Does Not Exist');

PRINT 'Inserted non-existent product. Checking alerts...';
PRINT '';

SELECT
    AlertID, AlertType, Severity, ProductNum,
    Operation, TableName,
    ConnectionApp, ConnectionLogin,
    Details, DetectedAt
FROM dbo.ShopSuspiciousActivityLog
WHERE AlertType = 'UNKNOWN_PRODUCT'
ORDER BY AlertID DESC;

PRINT '';


-- ============================================================================
-- RESULTS SUMMARY
-- ============================================================================

PRINT '================================================================';
PRINT '  FULL ALERT SUMMARY';
PRINT '================================================================';

SELECT
    AlertType,
    Severity,
    CASE Severity
        WHEN 1 THEN 'Low'
        WHEN 2 THEN 'Medium'
        WHEN 3 THEN 'High'
        WHEN 4 THEN 'Critical'
    END AS SeverityLabel,
    TableName,
    Operation,
    UserNum,
    ProductNum,
    Quantity,
    AmountValue,
    ConnectionApp,
    ConnectionLogin,
    ConnectionHost,
    DetectedAt,
    Details
FROM dbo.ShopSuspiciousActivityLog
WHERE DetectedAt >= @Now
ORDER BY AlertID;

PRINT '';
PRINT 'Total alerts generated: ' + CAST(@@ROWCOUNT AS VARCHAR(10));
PRINT '';


-- ============================================================================
-- EXPECTED RESULTS CHECKLIST
-- ============================================================================
/*
  Scenario 1 (Zero Amount Insert):
    [x] ZERO_AMOUNT    — Sev 4, Table: ShopPurchaseSummary, Op: INSERT
    [x] DIRECT_INSERT  — Sev 4, Table: ShopPurchaseSummary, Op: INSERT

  Scenario 2 (Negative Amount):
    [x] NEGATIVE_AMOUNT — Sev 4, Table: ShopPurchaseSummary, Op: UPDATE
    [x] DIRECT_UPDATE   — Sev 4, Table: ShopPurchaseSummary, Op: UPDATE
    [x] ZERO_AMOUNT     — Sev 4 (amount went to 0 again)

  Scenario 3 (Excessive Quantity):
    [x] EXCESSIVE_QUANTITY — Sev 3, Table: ShopPurchaseSummary, Op: UPDATE
    [x] DIRECT_UPDATE      — Sev 4, Table: ShopPurchaseSummary, Op: UPDATE

  Scenario 4 (Direct ShopPurchase Insert):
    [x] DIRECT_INSERT — Sev 4, Table: ShopPurchase, Op: INSERT

  Scenario 5 (Unknown Product):
    [x] UNKNOWN_PRODUCT — Sev 4, Table: ShopPurchase, Op: INSERT
    [x] DIRECT_INSERT   — Sev 4, Table: ShopPurchase, Op: INSERT

  Total expected alerts: ~11 (some scenarios may produce 2-3 alerts each
  because multiple rules can fire on the same row)

  KEY FORENSIC FIELDS TO VERIFY IN EACH ALERT:
    - ConnectionApp   → Should show your SSMS/sqlcmd app name (NOT 'gs-api')
    - ConnectionLogin → Should show your SQL login
    - ConnectionHost  → Should show your machine name
    - Details         → JSON blob with full row data for investigation
*/


-- ============================================================================
-- CLEANUP: Remove test data
-- ============================================================================
-- Uncomment the lines below to clean up after testing.
-- Leave them commented if you want to inspect the data further.

/*
PRINT '--- CLEANUP ---';

-- Remove test ShopPurchase rows
DELETE FROM dbo.ShopPurchase
WHERE UserUID = @TestUserName
  AND ItemName IN (N'Stolen Item via Direct Insert', N'Hacked Item That Does Not Exist');

-- Remove test ShopPurchaseSummary row
DELETE FROM dbo.ShopPurchaseSummary
WHERE UserNum = @TestUserNum AND ProductNum = @TestProductNum;

-- Remove test alerts (only those created during this test run)
DELETE FROM dbo.ShopSuspiciousActivityLog
WHERE DetectedAt >= @Now;

PRINT 'Cleanup complete.';
*/
