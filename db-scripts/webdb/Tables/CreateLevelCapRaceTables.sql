USE RanWeb;
GO

-- CreateLevelCapRaceTables.sql
-- Creates the score and snapshot tables for LevelCapRace events.
-- Run once on WebDB after Phase 4 deployment.

-- Cumulative level progress per participant per event
IF NOT EXISTS (
    SELECT 1 FROM sys.tables WHERE name = 'LevelCapRaceScore'
)
BEGIN
    CREATE TABLE LevelCapRaceScore (
        EventID       UNIQUEIDENTIFIER NOT NULL,
        ChaNum        INT              NOT NULL,
        ChaName       NVARCHAR(64)     NOT NULL DEFAULT '',
        ChaClass      INT              NOT NULL DEFAULT 0,
        ChaSchool     INT              NULL,
        GuildNum      INT              NULL,
        GuildName     NVARCHAR(64)     NULL,
        StartLevel    INT              NOT NULL DEFAULT 0,
        CurrentLevel  INT              NOT NULL DEFAULT 0,
        LevelsGained  INT              NOT NULL DEFAULT 0,
        UpdatedAt     DATETIME         NOT NULL DEFAULT GETDATE(),
        PRIMARY KEY (EventID, ChaNum)
    );
END

-- Point-in-time snapshot of LevelCapRaceScore
IF NOT EXISTS (
    SELECT 1 FROM sys.tables WHERE name = 'SnapshotLevelCapRaceScore'
)
BEGIN
    CREATE TABLE SnapshotLevelCapRaceScore (
        SnapshotID    UNIQUEIDENTIFIER NOT NULL,
        ChaNum        INT              NOT NULL,
        ChaName       NVARCHAR(64)     NOT NULL DEFAULT '',
        ChaClass      INT              NOT NULL DEFAULT 0,
        ChaSchool     INT              NULL,
        GuildNum      INT              NULL,
        GuildName     NVARCHAR(64)     NULL,
        StartLevel    INT              NOT NULL DEFAULT 0,
        CurrentLevel  INT              NOT NULL DEFAULT 0,
        LevelsGained  INT              NOT NULL DEFAULT 0,
        Rank          INT              NOT NULL DEFAULT 0,
        PRIMARY KEY (SnapshotID, ChaNum),
        FOREIGN KEY (SnapshotID) REFERENCES EventSnapshot(SnapshotID) ON DELETE CASCADE
    );
END
