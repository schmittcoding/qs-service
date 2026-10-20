USE RanWeb
GO

-- Phase 1: Add EventCategory and CategoryConfig columns to EventDefinition
-- EventCategory: VARCHAR(20), NOT NULL, defaults to 'GvG' for backward compatibility
-- CategoryConfig: NVARCHAR(MAX), NULL, stores JSON blob for category-specific settings

IF NOT EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventDefinition' AND COLUMN_NAME = 'EventCategory'
)
BEGIN
    ALTER TABLE EventDefinition
    ADD EventCategory VARCHAR(20) NOT NULL DEFAULT 'GvG';
END
GO

IF NOT EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventDefinition' AND COLUMN_NAME = 'CategoryConfig'
)
BEGIN
    ALTER TABLE EventDefinition
    ADD CategoryConfig NVARCHAR(MAX) NULL;
END
GO

-- Set existing events to GvG (idempotent)
UPDATE EventDefinition
SET EventCategory = 'GvgKoth'
WHERE EventCategory = 'GvG';
GO
