USE RanWeb;
-- ============================================================
-- Event System Tables — Tyranny War + KOTH
-- Target database: WebDB
-- ============================================================

-- 1. Reference / Lookup Tables
CREATE TABLE RefClass (
    ChaClass    INT PRIMARY KEY,
    ClassName   NVARCHAR(20) NOT NULL
);

CREATE TABLE RefSchool (
    ChaSchool   INT PRIMARY KEY,
    SchoolName  NVARCHAR(20) NOT NULL,
    SchoolAbbr  VARCHAR(10) NOT NULL
);

-- 2. Event Definition
CREATE TABLE EventDefinition (
    EventID            UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventName          NVARCHAR(100) NOT NULL,
    Season             INT NOT NULL,
    EventMapMID        INT NOT NULL DEFAULT 222,
    EventMapSID        INT NOT NULL DEFAULT 0,
    MatchDurationMin   INT NOT NULL DEFAULT 45,
    TallyDelayMin      INT NOT NULL DEFAULT 30,
    RegistrationOpen   DATETIME NOT NULL,
    RegistrationClose  DATETIME NOT NULL,
    EventStart         DATETIME NOT NULL,
    EventEnd           DATETIME NOT NULL,
    MinLevel           INT NOT NULL DEFAULT 200,
    IsActive           BIT NOT NULL DEFAULT 1,
    TowerPointsPerTick INT NOT NULL DEFAULT 1,
    CreatedAt          DATETIME NOT NULL DEFAULT GETDATE()
);

-- 3. Event Registration
CREATE TABLE EventRegistration (
    RegistrationID   UNIQUEIDENTIFIER NOT NULL DEFAULT NEWSEQUENTIALID() PRIMARY KEY,
    EventID          UNIQUEIDENTIFIER NOT NULL,
    ChaNum           INT NOT NULL,
    UserNum          INT NOT NULL,
    GuildNum         INT NULL,
    ChaSchool        INT NULL,
    ChaClass         INT NOT NULL,
    ChaLevel         INT NOT NULL,
    JoinGvG          BIT NOT NULL DEFAULT 0,
    JoinKOTH         BIT NOT NULL DEFAULT 0,
    Status           TINYINT NOT NULL DEFAULT 1,
    RegisteredAt     DATETIME NOT NULL DEFAULT GETDATE(),
    UpdatedAt        DATETIME NULL,
    IPAddress        VARCHAR(45) NULL,

    CONSTRAINT UQ_Event_Character UNIQUE (EventID, ChaNum),
    CONSTRAINT CK_AtLeastOneCategory CHECK (JoinGvG = 1 OR JoinKOTH = 1),
    CONSTRAINT FK_Reg_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_Reg_Event_Guild   ON EventRegistration(EventID, GuildNum) WHERE JoinGvG = 1;
CREATE INDEX IX_Reg_Event_Class   ON EventRegistration(EventID, ChaClass) WHERE JoinKOTH = 1;
CREATE INDEX IX_Reg_UserNum       ON EventRegistration(EventID, UserNum);

-- 4. Match Schedule
CREATE TABLE MatchSchedule (
    MatchID          UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID          UNIQUEIDENTIFIER NOT NULL,
    MatchLabel       NVARCHAR(50) NULL,
    MatchDate        DATE NOT NULL,
    StartTime        DATETIME NOT NULL,
    EndTime          DATETIME NOT NULL,
    TallyScheduled   DATETIME NOT NULL,
    TallyStatus      VARCHAR(20) NOT NULL DEFAULT 'Pending',
    TallyStartedAt   DATETIME NULL,
    TallyCompletedAt DATETIME NULL,
    RowsTallied      INT NULL,
    KillsCounted     INT NULL,
    KillsRejected    INT NULL,

    CONSTRAINT FK_Match_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID),
    CONSTRAINT UQ_Match_Time  UNIQUE (EventID, StartTime),
    CONSTRAINT CK_Match_Times CHECK (EndTime > StartTime AND TallyScheduled > EndTime)
);

CREATE INDEX IX_Match_Tally ON MatchSchedule(TallyStatus, TallyScheduled);

-- 5. Per-Match Result Tables
CREATE TABLE MatchGuildResult (
    MatchID       UNIQUEIDENTIFIER NOT NULL,
    GuildNum      INT NOT NULL,
    GuildName     NVARCHAR(50) NOT NULL,
    ChaSchool     INT NOT NULL,
    MatchKills    INT NOT NULL DEFAULT 0,
    MatchDeaths   INT NOT NULL DEFAULT 0,
    MatchTower    INT NOT NULL DEFAULT 0,

    PRIMARY KEY (MatchID, GuildNum),
    CONSTRAINT FK_MGR_Match FOREIGN KEY (MatchID) REFERENCES MatchSchedule(MatchID)
);

CREATE TABLE MatchCharResult (
    MatchID       UNIQUEIDENTIFIER NOT NULL,
    ChaNum        INT NOT NULL,
    ChaName      NVARCHAR(50) NOT NULL,
    ChaClass      INT NOT NULL,
    GuildNum      INT NULL,
    GuildName     NVARCHAR(50) NULL,
    MatchKills    INT NOT NULL DEFAULT 0,
    MatchDeaths   INT NOT NULL DEFAULT 0,

    PRIMARY KEY (MatchID, ChaNum),
    CONSTRAINT FK_MCR_Match FOREIGN KEY (MatchID) REFERENCES MatchSchedule(MatchID)
);

-- 6. Cumulative Event Scores
CREATE TABLE GuildEventScore (
    EventID       UNIQUEIDENTIFIER NOT NULL,
    GuildNum      INT NOT NULL,
    GuildName     NVARCHAR(50) NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL DEFAULT 0,
    TotalDeaths   INT NOT NULL DEFAULT 0,
    TowerPoints   INT NOT NULL DEFAULT 0,
    Deduction     INT NOT NULL DEFAULT 0,
    ComputedScore AS (TowerPoints + TotalKills - TotalDeaths - Deduction) PERSISTED,

    PRIMARY KEY (EventID, GuildNum),
    CONSTRAINT FK_GScore_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_GScore_Rank   ON GuildEventScore(EventID, ComputedScore DESC);
CREATE INDEX IX_GScore_School ON GuildEventScore(EventID, ChaSchool, ComputedScore DESC);

CREATE TABLE SchoolEventScore (
    EventID       UNIQUEIDENTIFIER NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL DEFAULT 0,
    TotalDeaths   INT NOT NULL DEFAULT 0,
    TotalTower    INT NOT NULL DEFAULT 0,
    Deduction     INT NOT NULL DEFAULT 0,
    ComputedScore AS (TotalTower + TotalKills - TotalDeaths - Deduction) PERSISTED,

    PRIMARY KEY (EventID, ChaSchool)
);

CREATE TABLE KOTHScore (
    EventID     UNIQUEIDENTIFIER NOT NULL,
    ChaNum      INT NOT NULL,
    ChaName    NVARCHAR(50) NOT NULL,
    ChaClass    INT NOT NULL,
    ChaSchool   INT NOT NULL,
    GuildNum    INT NULL,
    GuildName   NVARCHAR(50) NULL,
    TotalKills  INT NOT NULL DEFAULT 0,
    TotalDeaths INT NOT NULL DEFAULT 0,
    Deduction   INT NOT NULL DEFAULT 0,

    PRIMARY KEY (EventID, ChaNum),
    CONSTRAINT FK_KOTH_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_KOTH_ClassRank ON KOTHScore(EventID, ChaClass, TotalKills DESC);

-- 7. Admin Deduction Log
CREATE TABLE AdminDeduction (
    DeductionID     UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID         UNIQUEIDENTIFIER NOT NULL,
    TargetType      VARCHAR(10) NOT NULL,
    TargetID        INT NULL,
    TargetChaSchool INT NULL,
    Points          INT NOT NULL,
    Reason          NVARCHAR(500) NOT NULL,
    AdminUserID     UNIQUEIDENTIFIER NOT NULL,
    AppliedAt       DATETIME NOT NULL DEFAULT GETDATE(),
    MatchID         UNIQUEIDENTIFIER NULL,
    IsReverted      BIT NOT NULL DEFAULT 0,
    RevertedAt      DATETIME NULL,
    RevertedBy      UNIQUEIDENTIFIER NULL,

    CONSTRAINT FK_Deduct_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID),
    CONSTRAINT FK_Deduct_Match FOREIGN KEY (MatchID) REFERENCES MatchSchedule(MatchID),
    CONSTRAINT CK_Deduct_Type CHECK (TargetType IN ('GUILD','PLAYER','SCHOOL')),
    CONSTRAINT CK_Deduct_Points CHECK (Points > 0)
);

CREATE INDEX IX_Deduct_Event ON AdminDeduction(EventID, TargetType, TargetID);

-- 8. Snapshot Tables
CREATE TABLE EventSnapshot (
    SnapshotID   UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID      UNIQUEIDENTIFIER NOT NULL,
    MatchID      UNIQUEIDENTIFIER NULL,
    SnapshotTime DATETIME NOT NULL,
    Label        NVARCHAR(50) NULL,
    CreatedAt    DATETIME NOT NULL DEFAULT GETDATE(),

    CONSTRAINT FK_Snap_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID),
    CONSTRAINT UQ_Snap_Time  UNIQUE (EventID, SnapshotTime)
);

CREATE TABLE SnapshotGuildScore (
    SnapshotID    UNIQUEIDENTIFIER NOT NULL,
    GuildNum      INT NOT NULL,
    GuildName     NVARCHAR(50) NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL,
    TotalDeaths   INT NOT NULL,
    TowerPoints   INT NOT NULL,
    Deduction     INT NOT NULL DEFAULT 0,
    ComputedScore INT NOT NULL,
    Rank          INT NOT NULL,

    PRIMARY KEY (SnapshotID, GuildNum),
    CONSTRAINT FK_SGS_Snap FOREIGN KEY (SnapshotID) REFERENCES EventSnapshot(SnapshotID)
);

CREATE TABLE SnapshotKOTHScore (
    SnapshotID   UNIQUEIDENTIFIER NOT NULL,
    ChaNum       INT NOT NULL,
    ChaName     NVARCHAR(50) NOT NULL,
    ChaClass     INT NOT NULL,
    ChaSchool    INT NOT NULL,
    GuildNum     INT NULL,
    GuildName    NVARCHAR(50) NULL,
    TotalKills   INT NOT NULL,
    TotalDeaths  INT NOT NULL,
    Deduction    INT NOT NULL DEFAULT 0,
    Rank         INT NOT NULL,

    PRIMARY KEY (SnapshotID, ChaNum),
    CONSTRAINT FK_SKS_Snap FOREIGN KEY (SnapshotID) REFERENCES EventSnapshot(SnapshotID)
);

CREATE TABLE SnapshotSchoolScore (
    SnapshotID    UNIQUEIDENTIFIER NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL,
    TotalDeaths   INT NOT NULL,
    TotalTower    INT NOT NULL,
    Deduction     INT NOT NULL DEFAULT 0,
    ComputedScore INT NOT NULL,
    Rank          INT NOT NULL,

    PRIMARY KEY (SnapshotID, ChaSchool),
    CONSTRAINT FK_SSS_Snap FOREIGN KEY (SnapshotID) REFERENCES EventSnapshot(SnapshotID)
);

-- 9. Seed Reference Data
INSERT INTO RefSchool (ChaSchool, SchoolName, SchoolAbbr) VALUES
(0, 'Sacred Gate', 'SG'),
(1, 'Mystic Peak', 'MP'),
(2, 'Phoenix', 'PH');

INSERT INTO RefClass (ChaClass, ClassName) VALUES
(0, 'Brawler'),
(1, 'Swordsman'),
(2, 'Archer'),
(3, 'Shaman'),
(4, 'Extreme'),
(5, 'Assassin'),
(6, 'Magician'),
(7, 'Science');

-- ============================================================
-- 10. Add per-match tower point configuration and tower winner columns
-- Tower points are rotational per match (e.g., Match 1: TH=25, Nuc=50, Faci=0).
-- Tower winners are set by admin after each match ends, which triggers tally.
-- ============================================================
ALTER TABLE MatchSchedule       ADD TowerPointsTH   INT NOT NULL DEFAULT 0;
ALTER TABLE MatchSchedule       ADD TowerPointsNuc  INT NOT NULL DEFAULT 0;
ALTER TABLE MatchSchedule       ADD TowerPointsFaci INT NOT NULL DEFAULT 0;

-- 11. Minimum guild members required for tally eligibility (default 0 = no minimum)
ALTER TABLE EventDefinition     ADD MinGuildMembers INT NOT NULL DEFAULT 0;

-- 12. Track total members participated per guild in snapshots
ALTER TABLE SnapshotGuildScore  ADD MemberCount INT NOT NULL DEFAULT 0;

-- Which school (ChaSchool INT) dominated each tower after the match.
-- NULL = no school controlled that tower. Set by admin post-match.
ALTER TABLE MatchSchedule       ADD TowerWinnerTH   INT NULL;
ALTER TABLE MatchSchedule       ADD TowerWinnerNuc  INT NULL;
ALTER TABLE MatchSchedule       ADD TowerWinnerFaci INT NULL;
