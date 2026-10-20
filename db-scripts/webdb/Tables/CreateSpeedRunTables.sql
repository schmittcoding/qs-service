USE RanWeb;
GO

-- CreateSpeedRunTables.sql
-- Creates the score and snapshot tables for SpeedRun events.
-- Run once on WebDB when SpeedRun tally is implemented (placeholder).

-- Completion times per participant per event
IF NOT EXISTS (
    SELECT 1 FROM sys.tables WHERE name = 'SpeedRunScore'
)
BEGIN
    CREATE TABLE SpeedRunScore (
        EventID                  UNIQUEIDENTIFIER NOT NULL,
        ChaNum                   INT              NOT NULL,
        ChaName                  NVARCHAR(64)     NOT NULL DEFAULT '',
        ChaClass                 INT              NOT NULL DEFAULT 0,
        ChaSchool                INT              NULL,
        GuildNum                 INT              NULL,
        GuildName                NVARCHAR(64)     NULL,
        CompletionTimeSeconds    INT              NULL,
        CompletedAt              DATETIME         NULL,
        UpdatedAt                DATETIME         NOT NULL DEFAULT GETDATE(),
        PRIMARY KEY (EventID, ChaNum)
    );
END

-- Point-in-time snapshot of SpeedRunScore
IF NOT EXISTS (
    SELECT 1 FROM sys.tables WHERE name = 'SnapshotSpeedRunScore'
)
BEGIN
    CREATE TABLE SnapshotSpeedRunScore (
        SnapshotID               UNIQUEIDENTIFIER NOT NULL,
        ChaNum                   INT              NOT NULL,
        ChaName                  NVARCHAR(64)     NOT NULL DEFAULT '',
        ChaClass                 INT              NOT NULL DEFAULT 0,
        ChaSchool                INT              NULL,
        GuildNum                 INT              NULL,
        GuildName                NVARCHAR(64)     NULL,
        CompletionTimeSeconds    INT              NULL,
        Rank                     INT              NOT NULL DEFAULT 0,
        PRIMARY KEY (SnapshotID, ChaNum),
        FOREIGN KEY (SnapshotID) REFERENCES EventSnapshot(SnapshotID) ON DELETE CASCADE
    );
END
