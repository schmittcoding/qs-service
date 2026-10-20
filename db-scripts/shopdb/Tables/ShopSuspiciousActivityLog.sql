USE RanShop;
GO

IF OBJECT_ID('dbo.ShopSuspiciousActivityLog', 'U') IS NULL
BEGIN
    CREATE TABLE dbo.ShopSuspiciousActivityLog (
        AlertID             BIGINT IDENTITY(1,1) PRIMARY KEY,
        AlertType           VARCHAR(50) NOT NULL,        -- 'ZERO_AMOUNT', 'DIRECT_INSERT', 'DIRECT_UPDATE', 'NEGATIVE_AMOUNT', 'EXCESSIVE_QUANTITY'
        Severity            TINYINT NOT NULL DEFAULT 1,  -- 1=Low, 2=Medium, 3=High, 4=Critical

        -- Who
        UserUID             UNIQUEIDENTIFIER NULL,
        UserNum             BIGINT NULL,

        -- What
        ProductNum          INT NULL,
        Quantity            INT NULL,
        AmountValue         DECIMAL(18,4) NULL,

        -- Context
        TableName           VARCHAR(100) NOT NULL,
        Operation           VARCHAR(10) NOT NULL,        -- 'INSERT', 'UPDATE', 'DELETE'
        Details             NVARCHAR(MAX) NULL,          -- JSON with full row data for forensics

        -- Connection info (who connected to DB)
        ConnectionHost      NVARCHAR(128) NULL,
        ConnectionApp       NVARCHAR(128) NULL,
        ConnectionLogin     NVARCHAR(128) NULL,

        -- When
        DetectedAt          DATETIME NOT NULL DEFAULT GETDATE(),

        -- Review status
        Reviewed            BIT NOT NULL DEFAULT 0,
        ReviewedBy          NVARCHAR(100) NULL,
        ReviewedAt          DATETIME NULL,
        ReviewNotes         NVARCHAR(MAX) NULL,

        INDEX IX_Alert_Type (AlertType),
        INDEX IX_Alert_Severity (Severity DESC, DetectedAt DESC),
        INDEX IX_Alert_DetectedAt (DetectedAt DESC),
        INDEX IX_Alert_UserNum (UserNum),
        INDEX IX_Alert_Reviewed (Reviewed, Severity DESC)
    );
END
GO
