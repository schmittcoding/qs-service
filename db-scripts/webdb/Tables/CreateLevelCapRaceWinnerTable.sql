USE RanWeb;
GO

IF OBJECT_ID('dbo.LevelCapRaceWinner', 'U') IS NOT NULL
    DROP TABLE dbo.LevelCapRaceWinner;
GO

CREATE TABLE dbo.LevelCapRaceWinner (
    EventID       UNIQUEIDENTIFIER NOT NULL,
    ChaNum        INT NOT NULL,
    ChaName       NVARCHAR(64) NOT NULL,
    ChaClass      INT NOT NULL,
    ChaSchool     INT NULL,
    GuildNum      INT NULL,
    GuildName     NVARCHAR(64) NULL,
    ReachedLevel  INT NOT NULL,
    LevelUpDate   DATETIME NULL,
    ReachedAt     DATETIME NOT NULL DEFAULT GETDATE(),
    Rank          INT NOT NULL,

    PRIMARY KEY (EventID, ChaNum),
    CONSTRAINT FK_LCRWinner_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_LCRWinner_EventReachedAt ON dbo.LevelCapRaceWinner(EventID, ReachedAt);

GO
