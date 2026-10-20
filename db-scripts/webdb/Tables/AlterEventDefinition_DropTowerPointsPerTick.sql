USE RanWeb;
GO

-- Drop the default constraint on TowerPointsPerTick before dropping the column.
DECLARE @df NVARCHAR(128);
SELECT @df = dc.name
FROM sys.default_constraints dc
JOIN sys.columns c ON dc.parent_object_id = c.object_id AND dc.parent_column_id = c.column_id
WHERE dc.parent_object_id = OBJECT_ID('EventDefinition') AND c.name = 'TowerPointsPerTick';

IF @df IS NOT NULL
BEGIN
    EXEC('ALTER TABLE EventDefinition DROP CONSTRAINT [' + @df + ']');
    PRINT 'Dropped default constraint on TowerPointsPerTick';
END

IF EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'EventDefinition' AND COLUMN_NAME = 'TowerPointsPerTick'
)
BEGIN
    ALTER TABLE EventDefinition DROP COLUMN TowerPointsPerTick;
    PRINT 'Dropped column TowerPointsPerTick from EventDefinition';
END
ELSE
BEGIN
    PRINT 'Column TowerPointsPerTick already removed from EventDefinition';
END
GO
