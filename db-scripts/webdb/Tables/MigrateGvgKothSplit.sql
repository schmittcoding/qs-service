USE RanWeb;
GO

-- ============================================================================
-- Migration: Remove JoinGvG / JoinKOTH columns from EventRegistration
-- Prerequisite: AlterEventDefinition_AddCategory.sql (EventCategory column exists)
--
-- In the new category-driven model, each EventDefinition has exactly one
-- EventCategory. A registration for that event implicitly belongs to that
-- category, so the per-registration JoinGvG/JoinKOTH flags are redundant.
--
-- This script:
--   1. Verifies the EventCategory column exists on EventDefinition.
--   2. Drops JoinGvG and JoinKOTH columns from EventRegistration.
-- ============================================================================

-- Safety: abort if EventCategory column does not exist yet
IF NOT EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventDefinition' AND COLUMN_NAME = 'EventCategory'
)
BEGIN
    RAISERROR('EventCategory column does not exist on EventDefinition. Run AlterEventDefinition_AddCategory.sql first.', 16, 1);
    RETURN;
END

-- Drop filtered indexes that reference JoinGvG / JoinKOTH
IF EXISTS (SELECT 1 FROM sys.indexes WHERE name = 'IX_Reg_Event_Guild' AND object_id = OBJECT_ID('EventRegistration'))
BEGIN
    DROP INDEX IX_Reg_Event_Guild ON EventRegistration;
    PRINT 'Dropped index IX_Reg_Event_Guild';
END

IF EXISTS (SELECT 1 FROM sys.indexes WHERE name = 'IX_Reg_Event_Class' AND object_id = OBJECT_ID('EventRegistration'))
BEGIN
    DROP INDEX IX_Reg_Event_Class ON EventRegistration;
    PRINT 'Dropped index IX_Reg_Event_Class';
END

-- Drop check constraint CK_AtLeastOneCategory (references both columns)
IF EXISTS (SELECT 1 FROM sys.check_constraints WHERE name = 'CK_AtLeastOneCategory' AND parent_object_id = OBJECT_ID('EventRegistration'))
BEGIN
    ALTER TABLE RanWeb.dbo.EventRegistration DROP CONSTRAINT CK_AtLeastOneCategory;
    PRINT 'Dropped constraint CK_AtLeastOneCategory';
END

-- Drop auto-named DEFAULT constraint on JoinGvG
DECLARE @dfJoinGvG NVARCHAR(128);
SELECT @dfJoinGvG = dc.name
FROM sys.default_constraints dc
JOIN sys.columns c ON dc.parent_object_id = c.object_id AND dc.parent_column_id = c.column_id
WHERE dc.parent_object_id = OBJECT_ID('EventRegistration') AND c.name = 'JoinGvG';

IF @dfJoinGvG IS NOT NULL
BEGIN
    EXEC('ALTER TABLE EventRegistration DROP CONSTRAINT [' + @dfJoinGvG + ']');
    PRINT 'Dropped default constraint on JoinGvG';
END

-- Drop auto-named DEFAULT constraint on JoinKOTH
DECLARE @dfJoinKOTH NVARCHAR(128);
SELECT @dfJoinKOTH = dc.name
FROM sys.default_constraints dc
JOIN sys.columns c ON dc.parent_object_id = c.object_id AND dc.parent_column_id = c.column_id
WHERE dc.parent_object_id = OBJECT_ID('EventRegistration') AND c.name = 'JoinKOTH';

IF @dfJoinKOTH IS NOT NULL
BEGIN
    EXEC('ALTER TABLE EventRegistration DROP CONSTRAINT [' + @dfJoinKOTH + ']');
    PRINT 'Dropped default constraint on JoinKOTH';
END

-- Set JoinGvG column to nullable if it exists
IF EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventRegistration' AND COLUMN_NAME = 'JoinGvG'
)
BEGIN
    ALTER TABLE EventRegistration ALTER COLUMN JoinGvG BIT NULL;
    PRINT 'Set column JoinGvG to nullable from EventRegistration';
END

-- Set JoinKOTH column to nullable if it exists
IF EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventRegistration' AND COLUMN_NAME = 'JoinKOTH'
)
BEGIN
    ALTER TABLE EventRegistration ALTER COLUMN JoinKOTH BIT NULL;
    PRINT 'Set column JoinKOTH to nullable from EventRegistration';
END

-- -- Drop JoinGvG column if it exists
-- IF EXISTS (
--     SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
--     WHERE TABLE_NAME = 'EventRegistration' AND COLUMN_NAME = 'JoinGvG'
-- )
-- BEGIN
--     ALTER TABLE EventRegistration DROP COLUMN JoinGvG;
--     PRINT 'Dropped column JoinGvG from EventRegistration';
-- END
-- ELSE
-- BEGIN
--     PRINT 'Column JoinGvG already removed from EventRegistration';
-- END

-- -- Drop JoinKOTH column if it exists
-- IF EXISTS (
--     SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
--     WHERE TABLE_NAME = 'EventRegistration' AND COLUMN_NAME = 'JoinKOTH'
-- )
-- BEGIN
--     ALTER TABLE EventRegistration DROP COLUMN JoinKOTH;
--     PRINT 'Dropped column JoinKOTH from EventRegistration';
-- END
-- ELSE
-- BEGIN
--     PRINT 'Column JoinKOTH already removed from EventRegistration';
-- END

PRINT 'Migration complete: JoinGvG/JoinKOTH columns removed from EventRegistration';
