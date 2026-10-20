USE RanShop;
GO

CREATE TABLE dbo.MileagePurchaseSummary (
    UserUID                 UNIQUEIDENTIFIER NOT NULL,
    UserNum                 BIGINT NOT NULL,
    ProductNum              INT NOT NULL,

    -- Lifetime Statistics
    TotalQuantityPurchased  INT NOT NULL DEFAULT 0,
    TotalAmountSpent        DECIMAL(10,2) NOT NULL DEFAULT 0,
    PurchaseCount           INT NOT NULL DEFAULT 0,
    FirstPurchaseDate       DATETIME NULL,
    LastPurchaseDate        DATETIME NULL,

    -- Current Year Tracking
    CurrentYearQuantity     INT NOT NULL DEFAULT 0,
    CurrentYear             INT NOT NULL DEFAULT YEAR(GETDATE()),
    CurrentYearSpent        DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Current Month Tracking
    CurrentMonthQuantity    INT NOT NULL DEFAULT 0,
    CurrentMonthYear        INT NOT NULL,
    CurrentMonthSpent       DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Current Day Tracking
    CurrentDayQuantity      INT NOT NULL DEFAULT 0,
    CurrentDay              DATE NOT NULL,
    CurrentDaySpent         DECIMAL(10,2) NOT NULL DEFAULT 0,

    -- Metadata
    LastUpdated             DATETIME NOT NULL DEFAULT GETDATE(),

    CONSTRAINT PK_MileagePurchaseSummary PRIMARY KEY (UserUID, ProductNum),
    INDEX IX_MileagePurchaseSummary_LastPurchase (LastPurchaseDate DESC),
    INDEX IX_MileagePurchaseSummary_Year (CurrentYear),
    INDEX IX_MileagePurchaseSummary_Month (CurrentMonthYear),
    INDEX IX_MileagePurchaseSummary_Day (CurrentDay)
);
