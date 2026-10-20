USE RanShop;
GO

-- Drop the table if it already exists
IF OBJECT_ID('dbo.ShopPurchaseSummary', 'U') IS NOT NULL
    DROP TABLE dbo.ShopPurchaseSummary;
GO

CREATE TABLE dbo.ShopPurchaseSummary (
    UserUID                 UNIQUEIDENTIFIER NOT NULL,
    UserNum                 BIGINT NOT NULL,
    ProductNum              INT NOT NULL,

    -- Lifetime Statistics
    TotalQuantityPurchased  INT NOT NULL DEFAULT 0,
    TotalAmountSpent        DECIMAL(10,2) NOT NULL DEFAULT 0,
    PurchaseCount           INT NOT NULL DEFAULT 0, -- Number of transactions
    FirstPurchaseDate       DATETIME NULL,
    LastPurchaseDate        DATETIME NULL,

    -- Current Year Tracking
    CurrentYearQuantity     INT NOT NULL DEFAULT 0,
    CurrentYear             INT NOT NULL DEFAULT YEAR(GETDATE()),
    CurrentYearSpent        DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Current Month Tracking
    CurrentMonthQuantity    INT NOT NULL DEFAULT 0,
    CurrentMonthYear        INT NOT NULL, -- Format: YYYYMM (e.g., 202602)
    CurrentMonthSpent       DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Current Day Tracking
    CurrentDayQuantity      INT NOT NULL DEFAULT 0,
    CurrentDay              DATE NOT NULL,
    CurrentDaySpent         DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Metadata
    LastUpdated             DATETIME NOT NULL DEFAULT GETDATE(),

    CONSTRAINT PK_ShopPurchaseSummary PRIMARY KEY (UserUID, ProductNum),
    INDEX IX_ShopPurchaseSummary_LastPurchase (LastPurchaseDate DESC),
    INDEX IX_ShopPurchaseSummary_Year (CurrentYear),
    INDEX IX_ShopPurchaseSummary_Month (CurrentMonthYear),
    INDEX IX_ShopPurchaseSummary_Day (CurrentDay)
);
