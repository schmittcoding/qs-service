USE RanWeb;
GO

-- Add LevelUpDate column to LevelCapRaceScore table
IF NOT EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'LevelCapRaceScore' AND COLUMN_NAME = 'LevelUpDate'
)
BEGIN
    ALTER TABLE LevelCapRaceScore
    ADD LevelUpDate DATETIME NULL;

    PRINT 'Added LevelUpDate column to LevelCapRaceScore';
END
ELSE
BEGIN
    PRINT 'LevelUpDate column already exists in LevelCapRaceScore';
END
GO

-- Add LevelUpDate column to SnapshotLevelCapRaceScore table for consistency
IF NOT EXISTS (
    SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_NAME = 'SnapshotLevelCapRaceScore' AND COLUMN_NAME = 'LevelUpDate'
)
BEGIN
    ALTER TABLE SnapshotLevelCapRaceScore
    ADD LevelUpDate DATETIME NULL;

    PRINT 'Added LevelUpDate column to SnapshotLevelCapRaceScore';
END
ELSE
BEGIN
    PRINT 'LevelUpDate column already exists in SnapshotLevelCapRaceScore';
END
GO
