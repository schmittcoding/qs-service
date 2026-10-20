USE RanWeb
GO

-- Widen UserVerificationToken column to accommodate JWT tokens
-- Previously: NVARCHAR(100)
-- Now: NVARCHAR(500)

IF EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'WebUser' AND COLUMN_NAME = 'UserVerificationToken'
)
BEGIN
    ALTER TABLE dbo.WebUser
    ALTER COLUMN UserVerificationToken NVARCHAR(500) NULL;
END
GO
