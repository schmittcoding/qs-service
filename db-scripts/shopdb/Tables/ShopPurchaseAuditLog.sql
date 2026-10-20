USE RanShop;
GO

-- Drop the table if it already exists
IF OBJECT_ID('dbo.ShopPurchaseAuditLog', 'U') IS NOT NULL
    DROP TABLE dbo.ShopPurchaseAuditLog;
GO

CREATE TABLE dbo.ShopPurchaseAuditLog (
    AuditID BIGINT IDENTITY(1,1) PRIMARY KEY,

    -- Reference
    TransactionID BIGINT NULL,
    PurchaseItemID BIGINT NULL,

    -- Event Details
    EventType VARCHAR(50) NOT NULL,
    -- 'Purchase', 'Refund', 'GameInsertion', 'StatusChange', 'Error'
    EventDate DATETIME NOT NULL DEFAULT GETDATE(),
    EventStatus VARCHAR(20) NULL, -- 'Success', 'Failed', 'Pending'

    -- Context
    UserUID VARCHAR(255) NULL,
    PerformedBy VARCHAR(255) NULL, -- Admin username if manual action
    IPAddress VARCHAR(45) NULL,

    -- Details
    Description NVARCHAR(500) NULL,
    ErrorMessage NVARCHAR(MAX) NULL,
    ErrorCode VARCHAR(50) NULL,

    -- Raw Data (for debugging)
    RequestData NVARCHAR(MAX) NULL,
    ResponseData NVARCHAR(MAX) NULL,

    -- Indexes
    INDEX IX_AuditLog_Transaction (TransactionID, EventDate DESC),
    INDEX IX_AuditLog_Date (EventDate DESC),
    INDEX IX_AuditLog_User (UserUID, EventDate DESC),
    INDEX IX_AuditLog_EventType (EventType, EventDate DESC),
    INDEX IX_AuditLog_Status (EventStatus, EventDate DESC)
);
GO
