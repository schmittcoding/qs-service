USE RanShop
GO

CREATE TABLE dbo.MileageConfig (
    ConfigID        INT             IDENTITY(1,1)   PRIMARY KEY,
    MileageRate     DECIMAL(5,2)    NOT NULL DEFAULT 5.00,
    IsActive        BIT             NOT NULL DEFAULT 1,
    EffectiveFrom   DATETIME        NOT NULL DEFAULT GETDATE(),
    EffectiveTo     DATETIME        NULL,
    Description     NVARCHAR(200)   NULL,
    UpdatedBy       NVARCHAR(50)    NOT NULL,
    CreatedAt       DATETIME        NOT NULL DEFAULT GETDATE(),
    UpdatedAt       DATETIME        NOT NULL DEFAULT GETDATE()
);
GO

CREATE INDEX IX_MileageConfig_Active ON dbo.MileageConfig(IsActive, EffectiveFrom DESC);
GO

-- MileageRate: percentage of WebPoints spent that becomes mileage (e.g. 5.00 = 5%)
-- Only one row should have IsActive = 1 at any time.
-- EffectiveTo NULL = no end date (permanent until replaced).
-- For promotions: insert a new row with higher rate + EffectiveTo = event end date.

-- Seed default config
INSERT INTO dbo.MileageConfig (MileageRate, IsActive, Description, UpdatedBy)
VALUES (5.00, 1, 'Default mileage rate', 'system');
GO
