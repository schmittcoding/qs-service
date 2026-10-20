# Ran Online — Tyranny War + KOTH Event System Design (Final)

---

## Table of Contents

1. [High-Level Architecture](#1-high-level-architecture)
2. [Registration System + Process Flow](#2-registration-system--process-flow)
3. [Database Schema](#3-database-schema)
4. [Data Flow (End-to-End Pipeline)](#4-data-flow-end-to-end-pipeline)
5. [Optimization Strategies](#5-optimization-strategies)
6. [Example Queries](#6-example-queries)
7. [API + Frontend Structure](#7-api--frontend-structure)
8. [Edge Cases + Admin Deductions](#8-edge-cases--admin-deductions)
9. [Tradeoffs and Recommendations](#9-tradeoffs-and-recommendations)
10. [Admin Event & Match Management](#10-admin-event--match-management)

---

## Design Principles

- **No live leaderboard updates during a match.** Frontend shows a "match ongoing" banner.
- **Post-match batch tally.** Aggregation runs once, 30 minutes after each match ends.
- **Dual-eligibility rule.** A kill only counts if BOTH the attacker AND the victim are registered.
- **Fixed match windows.** Each match is exactly 45 minutes, tracked by ActionDate range.
- **Admin point deductions.** Admins can impose penalties on players, guilds, or schools.
- **Column conventions.** All columns align with existing ChaInfo naming (`ChaSchool INT`, `ChaClass INT`, `ChaLevel INT`, `UserNum INT`). All system IDs (`EventID`, `MatchID`, `SnapshotID`, `RegistrationID`, `DeductionID`) are UUIDs. Every `GuildName` is always paired with its `GuildNum`.

---

## 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (Website)                             │
│  ┌──────────────┐  ┌──────────────────┐  ┌────────────────────────┐    │
│  │ Registration  │  │ Leaderboards     │  │ Snapshot History       │    │
│  │ Form + Status │  │ GvG / School /   │  │ Timeline Viewer        │    │
│  │               │  │ KOTH per-class   │  │                        │    │
│  └──────┬───────┘  └────────┬─────────┘  └───────────┬────────────┘    │
│         │           ▲ Shows "Match     ▲             │                  │
│         │           │  ongoing" if      │             │                  │
│         │           │  match is live    │             │                  │
└─────────┼───────────┼──────────────────┼─────────────┼──────────────────┘
          │           │                  │             │
          ▼           │                  │             ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           API LAYER (REST)                              │
│  POST /register         GET /leaderboard/{type}                        │
│  PUT  /register         GET /match/status  ← returns match state       │
│  GET  /register         GET /snapshots/{eventId}                       │
│  POST /admin/deduction  POST /admin/events  ← create event            │
│  POST /admin/match/{id}/trigger-tally                                  │
│  POST /admin/events/{id}/matches/generate  ← schedule matches         │
└────────┬────────────────────┬────────────────────────┬──────────────────┘
         │                    │                        │
         ▼                    ▼                        ▼
┌─────────────────┐  ┌───────────────────┐  ┌────────────────────────────┐
│  REGISTRATION    │  │  CACHE LAYER      │  │  SNAPSHOT SERVICE          │
│  SERVICE         │  │  (Redis)          │  │  (Cron / Scheduler)        │
│                  │  │                   │  │                            │
│  - Validate      │  │  - Guild ranks    │  │  - Triggered after tally   │
│  - Deduplicate   │  │  - School ranks   │  │  - Copies current agg     │
│  - Write to DB   │  │  - KOTH per-class │  │    tables into snapshot    │
│                  │  │  - Match status   │  │  - Versioned by ID + time  │
│                  │  │  - TTL: until     │  │                            │
│                  │  │    next tally     │  │                            │
└────────┬────────┘  └───────┬───────────┘  └────────────┬───────────────┘
         │                   │                           │
         ▼                   ▼                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          PRIMARY DATABASE                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ Registration │ │ GuildScores  │ │ KOTHScore│ │ Snapshots        │   │
│  │ Table        │ │ + SchoolAgg  │ │ per-class│ │ (frozen copies)  │   │
│  └──────────────┘ └──────────────┘ └──────────┘ └──────────────────┘   │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────────────┐  │
│  │ MatchSchedule│ │ MatchResult  │ │ AdminDeduction                 │  │
│  │ (per match)  │ │ (per-match   │ │ (penalties on guild/char/school)│  │
│  │              │ │  raw tally)  │ │                                │  │
│  └──────────────┘ └──────────────┘ └────────────────────────────────┘  │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  ▲
                                  │
┌─────────────────────────────────┴───────────────────────────────────────┐
│                POST-MATCH TALLY JOB (Scheduled / One-Shot)             │
│                                                                         │
│  Triggered: 30 minutes after each match's EndTime                       │
│                                                                         │
│  1. Determine match time range (e.g. 20:00:00 - 20:45:00)              │
│  2. Query LogAction WHERE ActionDate BETWEEN @Start AND @End            │
│     AND baseMainMapID = 222 AND baseSubMapID = 0 AND Type = 2          │
│  3. INNER JOIN both ChaNum AND TargetNum against EventRegistration      │
│     → only count kills where BOTH parties are registered                │
│  4. Aggregate kills/deaths per character, per guild                     │
│  5. INSERT into MatchResult (per-match breakdown)                       │
│  6. UPSERT cumulative totals into GuildEventScore, KOTHScore            │
│  7. Apply AdminDeduction adjustments                                    │
│  8. Refresh SchoolEventScore                                            │
│  9. Invalidate Redis cache → leaderboards now show updated totals       │
│ 10. Mark match as TallyStatus = 'Completed'                            │
│                                                                         │
│  Runs ONCE per match, not continuously.                                 │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     GAME SERVER (Existing)                              │
│                                                                         │
│  LogAction table ← continuously written by game engine                  │
│  ActionNum auto-increments                                              │
│  We ONLY READ from this table. Never write to it.                       │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Summary

| Component            | Role                                                                  |
| -------------------- | --------------------------------------------------------------------- |
| Frontend             | Registration UI, leaderboards, "match ongoing" banner                 |
| API Layer            | RESTful endpoints for registration, leaderboards, match status, admin |
| Registration Service | Validates, deduplicates, writes to DB                                 |
| Tally Job            | Post-match batch processing — runs once per match, 30 min after end   |
| Cache Layer (Redis)  | Pre-computed leaderboard JSON — lives until next tally                |
| Snapshot Service     | Freezes standings after each tally                                    |
| Primary Database     | All event state — schemas listed in Section 3                         |

---

## 2. Registration System + Process Flow

### 2.1 Category Encoding

A single row per character per event. Two BIT columns encode membership:

| JoinGvG | JoinKOTH | Meaning                         |
| ------- | -------- | ------------------------------- |
| 1       | 0        | GvG only                        |
| 0       | 1        | KOTH only                       |
| 1       | 1        | Both                            |
| 0       | 0        | **Blocked by CHECK constraint** |

### 2.2 Registration Process Flow

```
User clicks "Register for Event"
        │
        ▼
┌──────────────────────────────┐
│ 1. Authenticate              │
│    Verify session/JWT → get  │
│    UserNum                   │
└──────────────┬───────────────┘
               ▼
┌──────────────────────────────┐
│ 2. Select character          │
│    API fetches chars from    │
│    ChaInfo for this UserNum  │
│    Returns: ChaNum, ChaName, │
│    ChaClass, ChaLevel,       │
│    GuildNum, ChaSchool       │
└──────────────┬───────────────┘
               ▼
┌──────────────────────────────┐
│ 3. Choose category           │
│    GvG / KOTH / Both         │
│    (GvG disabled if no guild)│
└──────────────┬───────────────┘
               ▼
┌──────────────────────────────────────────────────┐
│ 4. Server-side validation                        │
│                                                  │
│  V1. Event exists, registration window open      │
│  V2. Character belongs to this UserNum           │
│  V3. Character meets level requirement           │
│      (ChaLevel >= EventDefinition.MinLevel)      │
│  V4. If GvG, character must have a guild         │
│      (GuildNum IS NOT NULL and > 0)              │
│  V5. No duplicate registration                   │
│      (UNIQUE constraint on EventID + ChaNum)     │
│  V6. Account limit (max N chars per UserNum)     │
│  V7. ChaClass in allowed set                     │
│  V8. Rate limit (IP + UserNum)                   │
│  V9. If GvG, guild school consistency:           │
│      All guild members registered for GvG must   │
│      belong to the same ChaSchool                │
└──────────────┬───────────────────────────────────┘
               │
        ┌──────┴──────┐
        │  Passed?    │
        └──┬───────┬──┘
       Yes │       │ No → return error JSON
           ▼
┌──────────────────────────────────────┐
│ 5. INSERT (transactional)            │
│    BEGIN TRANSACTION                 │
│      INSERT INTO EventRegistration   │
│    COMMIT                            │
│    Catch UK violation as fallback    │
└──────────────┬───────────────────────┘
               ▼
┌──────────────────────────────┐
│ 6. Return success JSON       │
│    { registrationId, char,   │
│      categories, guild }     │
└──────────────────────────────┘
```

### 2.3 Handling Updates (Before Event Start)

```sql
UPDATE EventRegistration
SET    JoinGvG  = @NewGvG,
       JoinKOTH = @NewKOTH,
       UpdatedAt = GETDATE()
WHERE  EventID = @EventID
  AND  ChaNum  = @ChaNum
  AND  Status  = 1
  AND  EXISTS (
    SELECT 1 FROM EventDefinition
    WHERE EventID = @EventID
      AND GETDATE() BETWEEN RegistrationOpen AND RegistrationClose
  );
```

### 2.4 Failure Cases

| Failure             | Code           | Message                                                |
| ------------------- | -------------- | ------------------------------------------------------ |
| Already registered  | `DUPLICATE`    | "This character is already registered for this event." |
| Registration closed | `CLOSED`       | "Registration is no longer open."                      |
| Level too low       | `LEVEL_REQ`    | "Character must be at least level {min}."              |
| No guild for GvG    | `NO_GUILD`     | "You must be in a guild to join GvG."                  |
| Invalid class       | `BAD_CLASS`    | "Class is not eligible for this event."                |
| Account limit       | `ACCT_LIMIT`   | "You can register at most {n} characters."             |
| Rate limited        | `RATE_LIMIT`   | "Too many requests. Try again shortly."                |
| Not your character  | `UNAUTHORIZED` | "This character does not belong to your account."      |

### 2.5 API Endpoints for Registration

| Method   | Path                                        | Purpose                                 |
| -------- | ------------------------------------------- | --------------------------------------- |
| `GET`    | `/api/events/current`                       | Active event info + registration window |
| `GET`    | `/api/events/{eventId}/characters`          | UserNum's eligible characters           |
| `POST`   | `/api/events/{eventId}/register`            | Register a character                    |
| `PUT`    | `/api/events/{eventId}/register/{chaNum}`   | Update categories (before event start)  |
| `DELETE` | `/api/events/{eventId}/register/{chaNum}`   | Withdraw (Status=2)                     |
| `GET`    | `/api/events/{eventId}/registration-status` | Check registration for UserNum          |

---

## 3. Database Schema

### 3.0 Reference / Lookup Tables

Since `ChaSchool` and `ChaClass` are integers pulled from `ChaInfo`, the API layer must resolve display names. Use reference tables or application-layer constants:

```sql
CREATE TABLE RefClass (
    ChaClass    INT PRIMARY KEY,
    ClassName   NVARCHAR(20) NOT NULL       -- 'Shaman','Brawler','Assassin','Science',
                                            -- 'Swordsman','Magician','Archer','Extreme'
);

CREATE TABLE RefSchool (
    ChaSchool   INT PRIMARY KEY,
    SchoolName  NVARCHAR(20) NOT NULL,      -- 'Sacred Gate','Mystic Peak','Phoenix'
    SchoolAbbr  VARCHAR(10) NOT NULL        -- 'SG','MP','PNX'
);
```

### 3.1 EventDefinition

```sql
CREATE TABLE EventDefinition (
    EventID            UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventName          NVARCHAR(100) NOT NULL,
    Season             INT NOT NULL,
    EventMapMID        INT NOT NULL DEFAULT 222,
    EventMapSID        INT NOT NULL DEFAULT 0,
    MatchDurationMin   INT NOT NULL DEFAULT 45,      -- fixed 45-minute matches
    TallyDelayMin      INT NOT NULL DEFAULT 30,      -- tally runs 30 min after match ends
    RegistrationOpen   DATETIME NOT NULL,
    RegistrationClose  DATETIME NOT NULL,
    EventStart         DATETIME NOT NULL,
    EventEnd           DATETIME NOT NULL,
    MinLevel           INT NOT NULL DEFAULT 200,
    IsActive           BIT NOT NULL DEFAULT 1,
    TowerPointsPerTick INT NOT NULL DEFAULT 1,
    CreatedAt          DATETIME NOT NULL DEFAULT GETDATE()
);
```

### 3.2 EventRegistration

```sql
CREATE TABLE EventRegistration (
    RegistrationID   UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID          UNIQUEIDENTIFIER NOT NULL,       -- FK → EventDefinition
    ChaNum           INT NOT NULL,                    -- FK → ChaInfo
    UserNum          INT NOT NULL,                    -- account owning the character
    GuildNum         INT NULL,                        -- guild at time of registration
    ChaSchool        INT NULL,                        -- school INT from ChaInfo
    ChaClass         INT NOT NULL,                    -- class INT from ChaInfo
    ChaLevel         INT NOT NULL,                    -- level at registration time
    JoinGvG          BIT NOT NULL DEFAULT 0,
    JoinKOTH         BIT NOT NULL DEFAULT 0,
    Status           TINYINT NOT NULL DEFAULT 1,      -- 1=Active, 2=Withdrawn, 3=Disqualified
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
```

### 3.3 MatchSchedule

Each match is a discrete 45-minute window. This table drives everything: the frontend banner, the tally job trigger, and the ActionDate filter.

```sql
CREATE TABLE MatchSchedule (
    MatchID          UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID          UNIQUEIDENTIFIER NOT NULL,
    MatchLabel       NVARCHAR(50) NULL,              -- e.g. 'Week 1 - Wednesday'
    MatchDate        DATE NOT NULL,
    StartTime        DATETIME NOT NULL,              -- e.g. 2026-03-20 20:00:00
    EndTime          DATETIME NOT NULL,              -- e.g. 2026-03-20 20:45:00
    TallyScheduled   DATETIME NOT NULL,              -- e.g. 2026-03-20 21:15:00
    TallyStatus      VARCHAR(20) NOT NULL DEFAULT 'Pending',
                     -- 'Pending'    = match hasn't happened yet
                     -- 'Waiting'    = match ended, waiting for tally delay
                     -- 'Processing' = tally job running
                     -- 'Completed'  = tally done, scores updated
                     -- 'Failed'     = tally encountered an error
    TallyStartedAt   DATETIME NULL,
    TallyCompletedAt DATETIME NULL,
    RowsTallied      INT NULL,
    KillsCounted     INT NULL,                       -- valid kills (both parties registered)
    KillsRejected    INT NULL,                       -- kills where one party unregistered
    -- Per-match tower point values (rotational, set by admin at creation/edit)
    TowerPointsTH    INT NOT NULL DEFAULT 0,         -- Town Hall tower points for this match
    TowerPointsNuc   INT NOT NULL DEFAULT 0,         -- Nuclear tower points for this match
    TowerPointsFaci  INT NOT NULL DEFAULT 0,         -- Facility tower points for this match
    -- Which school (ChaSchool INT) won each tower — set by admin after match ends
    TowerWinnerTH    INT NULL,                       -- ChaSchool of school that won TH (NULL = no winner)
    TowerWinnerNuc   INT NULL,                       -- ChaSchool of school that won Nuc
    TowerWinnerFaci  INT NULL,                       -- ChaSchool of school that won Faci

    CONSTRAINT FK_Match_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID),
    CONSTRAINT UQ_Match_Time  UNIQUE (EventID, StartTime),
    CONSTRAINT CK_Match_Times CHECK (EndTime > StartTime AND TallyScheduled > EndTime)
);

CREATE INDEX IX_Match_Tally ON MatchSchedule(TallyStatus, TallyScheduled);
```

**Example data:**

```
MatchID (UUID)  | EventID (UUID)  | MatchLabel             | StartTime           | EndTime             | TallyScheduled
a1b2...         | f0e1...         | Week 1 - Wednesday     | 2026-03-18 20:00:00 | 2026-03-18 20:45:00 | 2026-03-18 21:15:00
c3d4...         | f0e1...         | Week 1 - Friday        | 2026-03-20 20:00:00 | 2026-03-20 20:45:00 | 2026-03-20 21:15:00
e5f6...         | f0e1...         | Week 1 - Saturday      | 2026-03-21 20:00:00 | 2026-03-21 20:45:00 | 2026-03-21 21:15:00
g7h8...         | f0e1...         | Week 1 - Sunday        | 2026-03-22 20:00:00 | 2026-03-22 20:45:00 | 2026-03-22 21:15:00
```

### 3.4 Per-Match Result Tables

Stores the breakdown from each individual match before rolling into cumulative scores. Provides full auditability — you can see exactly what each match contributed.

```sql
-- Guild-level results per match
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

-- Character-level results per match (for KOTH + audit)
CREATE TABLE MatchCharResult (
    MatchID       UNIQUEIDENTIFIER NOT NULL,
    ChaNum        INT NOT NULL,
    CharName      NVARCHAR(50) NOT NULL,
    ChaClass      INT NOT NULL,
    GuildNum      INT NULL,
    GuildName     NVARCHAR(50) NULL,
    MatchKills    INT NOT NULL DEFAULT 0,
    MatchDeaths   INT NOT NULL DEFAULT 0,

    PRIMARY KEY (MatchID, ChaNum),
    CONSTRAINT FK_MCR_Match FOREIGN KEY (MatchID) REFERENCES MatchSchedule(MatchID)
);
```

### 3.5 Cumulative Event Scores

```sql
-- Guild cumulative scores across all matches
CREATE TABLE GuildEventScore (
    EventID       UNIQUEIDENTIFIER NOT NULL,
    GuildNum      INT NOT NULL,
    GuildName     NVARCHAR(50) NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL DEFAULT 0,
    TotalDeaths   INT NOT NULL DEFAULT 0,
    TowerPoints   INT NOT NULL DEFAULT 0,
    Deduction     INT NOT NULL DEFAULT 0,       -- admin penalty points
    ComputedScore AS (TowerPoints + TotalKills - TotalDeaths - Deduction) PERSISTED,

    PRIMARY KEY (EventID, GuildNum),
    CONSTRAINT FK_GScore_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_GScore_Rank   ON GuildEventScore(EventID, ComputedScore DESC);
CREATE INDEX IX_GScore_School ON GuildEventScore(EventID, ChaSchool, ComputedScore DESC);

-- School aggregation (rebuilt after each tally)
CREATE TABLE SchoolEventScore (
    EventID       UNIQUEIDENTIFIER NOT NULL,
    ChaSchool     INT NOT NULL,
    TotalKills    INT NOT NULL DEFAULT 0,
    TotalDeaths   INT NOT NULL DEFAULT 0,
    TotalTower    INT NOT NULL DEFAULT 0,
    Deduction     INT NOT NULL DEFAULT 0,       -- school-level penalty
    ComputedScore AS (TotalTower + TotalKills - TotalDeaths - Deduction) PERSISTED,

    PRIMARY KEY (EventID, ChaSchool)
);

-- KOTH per-character cumulative scores
CREATE TABLE KOTHScore (
    EventID     UNIQUEIDENTIFIER NOT NULL,
    ChaNum      INT NOT NULL,
    CharName    NVARCHAR(50) NOT NULL,
    ChaClass    INT NOT NULL,
    ChaSchool   INT NULL,                       -- school from registration (avoids JOIN)
    GuildNum    INT NULL,
    GuildName   NVARCHAR(50) NULL,
    TotalKills  INT NOT NULL DEFAULT 0,
    TotalDeaths INT NOT NULL DEFAULT 0,
    Deduction   INT NOT NULL DEFAULT 0,         -- player-level penalty

    PRIMARY KEY (EventID, ChaNum),
    CONSTRAINT FK_KOTH_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID)
);

CREATE INDEX IX_KOTH_ClassRank ON KOTHScore(EventID, ChaClass, TotalKills DESC);
```

### 3.6 Admin Deduction Log

Full audit trail of every penalty imposed:

```sql
CREATE TABLE AdminDeduction (
    DeductionID     UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID         UNIQUEIDENTIFIER NOT NULL,
    TargetType      VARCHAR(10) NOT NULL,        -- 'GUILD', 'PLAYER', 'SCHOOL'
    TargetID        INT NULL,                    -- GuildNum or ChaNum (NULL if school)
    TargetChaSchool INT NULL,                    -- for school-level deductions
    Points          INT NOT NULL,                -- positive number = penalty
    Reason          NVARCHAR(500) NOT NULL,      -- mandatory justification
    AdminUserID     INT NOT NULL,                -- who imposed the penalty
    AppliedAt       DATETIME NOT NULL DEFAULT GETDATE(),
    MatchID         UNIQUEIDENTIFIER NULL,       -- optional: tied to a specific match
    IsReverted      BIT NOT NULL DEFAULT 0,
    RevertedAt      DATETIME NULL,
    RevertedBy      INT NULL,

    CONSTRAINT FK_Deduct_Event FOREIGN KEY (EventID) REFERENCES EventDefinition(EventID),
    CONSTRAINT FK_Deduct_Match FOREIGN KEY (MatchID) REFERENCES MatchSchedule(MatchID),
    CONSTRAINT CK_Deduct_Type CHECK (TargetType IN ('GUILD','PLAYER','SCHOOL')),
    CONSTRAINT CK_Deduct_Points CHECK (Points > 0)
);

CREATE INDEX IX_Deduct_Event ON AdminDeduction(EventID, TargetType, TargetID);
```

### 3.7 Snapshot Tables

```sql
CREATE TABLE EventSnapshot (
    SnapshotID   UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID() PRIMARY KEY,
    EventID      UNIQUEIDENTIFIER NOT NULL,
    MatchID      UNIQUEIDENTIFIER NULL,          -- snapshot taken after this match's tally
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
    CharName     NVARCHAR(50) NOT NULL,
    ChaClass     INT NOT NULL,
    ChaSchool    INT NULL,
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
```

### 3.8 Index on LogAction (Not Our Table — Read Only)

```sql
-- Optimized for the post-match tally query: ActionDate range + map filter
CREATE INDEX IX_LogAction_MatchTally
ON LogAction (ActionDate, baseMainMapID, baseSubMapID, Type)
INCLUDE (ActionNum, ChaNum, TargetNum)
WHERE baseMainMapID = 222 AND baseSubMapID = 0;
```

This filtered index is small (only event-map rows) and perfectly covers the tally query's WHERE clause and SELECT list.

### 3.9 UUID Implementation Notes

These IDs represent event-management entities created through the website, not high-throughput game data. Volume is low, so UUID overhead is negligible.

```sql
-- For tables with high insert volume (EventRegistration), use sequential UUIDs
-- to reduce page splits on the clustered index:
RegistrationID UNIQUEIDENTIFIER NOT NULL DEFAULT NEWSEQUENTIALID()

-- For low-volume tables (EventDefinition, MatchSchedule), NEWID() is fine:
EventID UNIQUEIDENTIFIER NOT NULL DEFAULT NEWID()
```

Application-side generation (when you need the ID before INSERT):

```javascript
// Node.js
const { randomUUID } = require("crypto");
const eventId = randomUUID();
```

```csharp
// C#
var eventId = Guid.NewGuid();
```

```php
// PHP
$eventId = Ramsey\Uuid\Uuid::uuid4()->toString();
```

### 3.10 Schema Summary — All 16 Tables

| Table               | PK                     | Purpose                             |
| ------------------- | ---------------------- | ----------------------------------- |
| RefClass            | ChaClass (INT)         | Class display name lookup           |
| RefSchool           | ChaSchool (INT)        | School display name lookup          |
| EventDefinition     | EventID (UUID)         | Event configuration                 |
| EventRegistration   | RegistrationID (UUID)  | Character registration per event    |
| MatchSchedule       | MatchID (UUID)         | Discrete 45-min match windows       |
| MatchGuildResult    | MatchID + GuildNum     | Per-match guild breakdown           |
| MatchCharResult     | MatchID + ChaNum       | Per-match character breakdown       |
| GuildEventScore     | EventID + GuildNum     | Cumulative guild scores             |
| SchoolEventScore    | EventID + ChaSchool    | Aggregated school scores            |
| KOTHScore           | EventID + ChaNum       | Cumulative per-player scores        |
| AdminDeduction      | DeductionID (UUID)     | Penalty audit trail                 |
| EventSnapshot       | SnapshotID (UUID)      | Snapshot metadata                   |
| SnapshotGuildScore  | SnapshotID + GuildNum  | Frozen guild standings              |
| SnapshotKOTHScore   | SnapshotID + ChaNum    | Frozen KOTH standings               |
| SnapshotSchoolScore | SnapshotID + ChaSchool | Frozen school standings             |
| LogAction           | ActionNum (BIGINT)     | **Existing game table — read only** |

---

## 4. Data Flow (End-to-End Pipeline)

### Phase A: Setup (Admin) — See Section 10 for full details

```
1. Admin creates EventDefinition via POST /api/admin/events
   → Event starts in DRAFT state (IsActive = 0)
   → Validates: date sequence, season uniqueness, required fields

2. Admin populates MatchSchedule via one of:
   a. POST /api/admin/events/{eventId}/matches           (single or bulk)
   b. POST /api/admin/events/{eventId}/matches/generate  (auto-generate from pattern)
   → Server computes EndTime = StartTime + 45min, TallyScheduled = EndTime + 30min
   → Validates: within event range, no overlap, future start time

3. Admin activates the event via PATCH /api/admin/events/{eventId}/activate
   → Requires at least 1 match scheduled
   → Event is now visible on frontend

4. Registration opens at RegistrationOpen datetime
   Players register via website (see Section 2)

5. Registration closes at RegistrationClose datetime
```

### Phase B: Match Lifecycle (Per Match)

```
 ┌──────────────────────────────────────────────────────────────────────┐
 │  TIMELINE FOR ONE MATCH (e.g. Wednesday March 18)                   │
 │                                                                      │
 │  20:00:00 ──── Match starts ────────────────────────── 20:45:00     │
 │     │          Game server writes LogAction rows           │         │
 │     │          for all combat on MID=222, SID=0            │         │
 │     │                                                      │         │
 │     │   Frontend shows:                                    │         │
 │     │   ┌────────────────────────────────────────────┐     │         │
 │     │   │  ⚔️ Match is ongoing!                      │     │         │
 │     │   │  The total tally will be updated after     │     │         │
 │     │   │  the match. Check back after 9:15 PM.     │     │         │
 │     │   └────────────────────────────────────────────┘     │         │
 │     │                                                      │         │
 │  20:45:00 ──── Match ends ─────────────────────────────────┘         │
 │     │                                                                │
 │     │  Admin sets tower results via:                                 │
 │     │  PUT /api/admin/matches/{matchId}/tower-results               │
 │     │  { towerWinnerTH: 0, towerWinnerNuc: 1, towerWinnerFaci: 2 } │
 │     │  → TallyStatus changes: 'Pending' → 'Waiting'                │
 │     │                                                                │
 │     │  30-minute grace period (logs finish writing,                  │
 │     │  late network packets arrive, etc.)                            │
 │     │                                                                │
 │  21:15:00 ──── Tally job fires ─────────────────────────────         │
 │     │  TallyStatus: 'Waiting' → 'Processing'                        │
 │     │                                                                │
 │     │  1. Query LogAction WHERE ActionDate BETWEEN                   │
 │     │     '2026-03-18 20:00:00' AND '2026-03-18 20:45:00'          │
 │     │     AND baseMainMapID = 222 AND baseSubMapID = 0              │
 │     │     AND Type = 2  (killer perspective only)                    │
 │     │                                                                │
 │     │  2. INNER JOIN BOTH ChaNum and TargetNum against              │
 │     │     EventRegistration WHERE Status = 1 (active)               │
 │     │     → Kill only counts if BOTH are registered                 │
 │     │                                                                │
 │     │  3. Aggregate per-character and per-guild                      │
 │     │  4. INSERT into MatchGuildResult + MatchCharResult             │
 │     │  5. ADD match results to cumulative GuildEventScore            │
 │     │  6. ADD match results to cumulative KOTHScore                  │
 │     │  7. Apply AdminDeduction totals                                │
 │     │  8. Refresh SchoolEventScore via MERGE                         │
 │     │  9. Invalidate Redis cache                                     │
 │     │                                                                │
 │  ~21:16:00 ── Tally complete ───────────────────────────────         │
 │     TallyStatus: 'Processing' → 'Completed'                         │
 │     Leaderboards now show updated cumulative scores                  │
 │                                                                      │
 │  Immediately after: take snapshot (frozen copy of standings)         │
 └──────────────────────────────────────────────────────────────────────┘
```

### Phase C: Post-Match Tally Job (Full SQL)

```sql
-- ================================================================
-- STEP 1: Identify the match to tally
-- ================================================================
DECLARE @MatchID UNIQUEIDENTIFIER, @EventID UNIQUEIDENTIFIER,
        @Start DATETIME, @End DATETIME;

SELECT TOP 1
    @MatchID = MatchID,
    @EventID = EventID,
    @Start   = StartTime,
    @End     = EndTime
FROM   MatchSchedule
WHERE  TallyStatus = 'Waiting'
  AND  TallyScheduled <= GETDATE()
ORDER BY TallyScheduled;

UPDATE MatchSchedule
SET    TallyStatus = 'Processing', TallyStartedAt = GETDATE()
WHERE  MatchID = @MatchID;

-- ================================================================
-- STEP 2: Extract valid kills from LogAction
--         BOTH attacker AND victim must be registered & active
-- ================================================================
SELECT
    L.ChaNum          AS AttackerChaNum,
    L.TargetNum       AS VictimChaNum,
    RA.GuildNum        AS AttackerGuild,
    RA.ChaSchool       AS AttackerSchool,
    RA.ChaClass        AS AttackerClass,
    RA.JoinGvG         AS AttackerInGvG,
    RA.JoinKOTH        AS AttackerInKOTH,
    RV.GuildNum        AS VictimGuild,
    RV.ChaSchool       AS VictimSchool,
    RV.JoinGvG         AS VictimInGvG,
    RV.JoinKOTH        AS VictimInKOTH
INTO #ValidKills
FROM LogAction L WITH (NOLOCK)
INNER JOIN EventRegistration RA
    ON  RA.EventID = @EventID
    AND RA.ChaNum  = L.ChaNum
    AND RA.Status  = 1
INNER JOIN EventRegistration RV
    ON  RV.EventID = @EventID
    AND RV.ChaNum  = L.TargetNum
    AND RV.Status  = 1
WHERE L.ActionDate >= @Start
  AND L.ActionDate <  @End          -- half-open interval [Start, End)
  AND L.baseMainMapID = 222
  AND L.baseSubMapID  = 0
  AND L.Type = 2;                   -- killer perspective only

DECLARE @ValidKills INT;
SELECT @ValidKills = COUNT(*) FROM #ValidKills;

-- ================================================================
-- STEP 3: Aggregate per-character
-- ================================================================
SELECT
    ChaNum,
    MAX(CharName) AS CharName,
    MAX(ChaClass) AS ChaClass,
    MAX(GuildNum) AS GuildNum,
    SUM(MatchKills) AS MatchKills,
    SUM(MatchDeaths) AS MatchDeaths
INTO #CharFinal
FROM (
    SELECT AttackerChaNum AS ChaNum, NULL AS CharName,
           AttackerClass AS ChaClass, AttackerGuild AS GuildNum,
           COUNT(*) AS MatchKills, 0 AS MatchDeaths
    FROM   #ValidKills
    GROUP BY AttackerChaNum, AttackerClass, AttackerGuild

    UNION ALL

    SELECT VictimChaNum AS ChaNum, NULL AS CharName,
           NULL AS ChaClass, VictimGuild AS GuildNum,
           0 AS MatchKills, COUNT(*) AS MatchDeaths
    FROM   #ValidKills
    GROUP BY VictimChaNum, VictimGuild
) AS CharAgg
GROUP BY ChaNum;

-- Fill display names from ChaInfo
UPDATE cf
SET    cf.CharName = c.ChaName,
       cf.ChaClass = COALESCE(cf.ChaClass, c.ChaClass),
       cf.GuildNum = COALESCE(cf.GuildNum, r.GuildNum)
FROM   #CharFinal cf
JOIN   ChaInfo c ON c.ChaNum = cf.ChaNum
JOIN   EventRegistration r ON r.ChaNum = cf.ChaNum AND r.EventID = @EventID
WHERE  cf.CharName IS NULL;

-- ================================================================
-- STEP 4: Aggregate per-guild (GvG — both parties must be GvG)
-- ================================================================
SELECT
    GuildNum,
    MAX(ChaSchool) AS ChaSchool,
    SUM(MatchKills) AS MatchKills,
    SUM(MatchDeaths) AS MatchDeaths
INTO #GuildFinal
FROM (
    SELECT AttackerGuild AS GuildNum, AttackerSchool AS ChaSchool,
           COUNT(*) AS MatchKills, 0 AS MatchDeaths
    FROM   #ValidKills
    WHERE  AttackerInGvG = 1 AND VictimInGvG = 1
    GROUP BY AttackerGuild, AttackerSchool

    UNION ALL

    SELECT VictimGuild AS GuildNum, VictimSchool AS ChaSchool,
           0 AS MatchKills, COUNT(*) AS MatchDeaths
    FROM   #ValidKills
    WHERE  AttackerInGvG = 1 AND VictimInGvG = 1
    GROUP BY VictimGuild, VictimSchool
) AS GuildAgg
GROUP BY GuildNum;

-- ================================================================
-- STEP 4b: Compute per-guild tower points from tower winners
--          A guild earns tower points for each tower won by its school.
-- ================================================================
DECLARE @TowerPtsTH  INT, @TowerPtsNuc  INT, @TowerPtsFaci  INT;
DECLARE @TowerWinTH  INT, @TowerWinNuc  INT, @TowerWinFaci  INT;

SELECT @TowerPtsTH  = TowerPointsTH,
       @TowerPtsNuc  = TowerPointsNuc,
       @TowerPtsFaci = TowerPointsFaci,
       @TowerWinTH   = TowerWinnerTH,
       @TowerWinNuc  = TowerWinnerNuc,
       @TowerWinFaci = TowerWinnerFaci
FROM MatchSchedule WHERE MatchID = @MatchID;

ALTER TABLE #GuildFinal ADD MatchTower INT NOT NULL DEFAULT 0;

UPDATE gf
SET gf.MatchTower =
    CASE WHEN gf.ChaSchool = @TowerWinTH  THEN ISNULL(@TowerPtsTH, 0)  ELSE 0 END
  + CASE WHEN gf.ChaSchool = @TowerWinNuc  THEN ISNULL(@TowerPtsNuc, 0)  ELSE 0 END
  + CASE WHEN gf.ChaSchool = @TowerWinFaci THEN ISNULL(@TowerPtsFaci, 0) ELSE 0 END
FROM #GuildFinal gf;

-- ================================================================
-- STEP 5: Write per-match results (audit trail)
-- ================================================================
INSERT INTO MatchCharResult (MatchID, ChaNum, CharName, ChaClass, GuildNum, GuildName, MatchKills, MatchDeaths)
SELECT @MatchID, cf.ChaNum, cf.CharName, cf.ChaClass, cf.GuildNum,
       g.GuildName,
       cf.MatchKills, cf.MatchDeaths
FROM   #CharFinal cf
LEFT JOIN GuildInfo g ON g.GuildNum = cf.GuildNum;

INSERT INTO MatchGuildResult (MatchID, GuildNum, GuildName, ChaSchool, MatchKills, MatchDeaths, MatchTower)
SELECT @MatchID, gf.GuildNum,
       COALESCE(g.GuildName, 'Unknown'),
       gf.ChaSchool,
       gf.MatchKills, gf.MatchDeaths, gf.MatchTower
FROM   #GuildFinal gf
LEFT JOIN GuildInfo g ON g.GuildNum = gf.GuildNum;

-- ================================================================
-- STEP 6: Update cumulative scores (now includes tower points)
-- ================================================================

-- 6a. Guild scores (UPSERT)
MERGE GuildEventScore AS target
USING #GuildFinal AS source
ON target.EventID = @EventID AND target.GuildNum = source.GuildNum
WHEN MATCHED THEN
    UPDATE SET
        TotalKills  = target.TotalKills  + source.MatchKills,
        TotalDeaths = target.TotalDeaths + source.MatchDeaths,
        TowerPoints = target.TowerPoints + source.MatchTower
WHEN NOT MATCHED THEN
    INSERT (EventID, GuildNum, GuildName, ChaSchool, TotalKills, TotalDeaths, TowerPoints, Deduction)
    VALUES (@EventID, source.GuildNum,
            (SELECT GuildName FROM GuildInfo WHERE GuildNum = source.GuildNum),
            source.ChaSchool,
            source.MatchKills, source.MatchDeaths, source.MatchTower, 0);

-- 6b. KOTH scores (only KOTH-registered players, includes ChaSchool from registration)
MERGE KOTHScore AS target
USING (
    SELECT cf.ChaNum, cf.CharName, cf.ChaClass, cf.GuildNum,
           g.GuildName, r.ChaSchool,
           cf.MatchKills, cf.MatchDeaths
    FROM   #CharFinal cf
    JOIN   EventRegistration r ON r.ChaNum = cf.ChaNum AND r.EventID = @EventID
    LEFT JOIN GuildInfo g ON g.GuildNum = cf.GuildNum
    WHERE  r.JoinKOTH = 1
) AS source
ON target.EventID = @EventID AND target.ChaNum = source.ChaNum
WHEN MATCHED THEN
    UPDATE SET
        TotalKills  = target.TotalKills  + source.MatchKills,
        TotalDeaths = target.TotalDeaths + source.MatchDeaths
WHEN NOT MATCHED THEN
    INSERT (EventID, ChaNum, CharName, ChaClass, ChaSchool, GuildNum, GuildName, TotalKills, TotalDeaths, Deduction)
    VALUES (@EventID, source.ChaNum, source.CharName, source.ChaClass,
            source.ChaSchool, source.GuildNum, source.GuildName,
            source.MatchKills, source.MatchDeaths, 0);

-- ================================================================
-- STEP 7: Apply admin deductions (recalculate from full log)
-- ================================================================
UPDATE g
SET    g.Deduction = COALESCE(d.TotalDeduction, 0)
FROM   GuildEventScore g
LEFT JOIN (
    SELECT TargetID, SUM(Points) AS TotalDeduction
    FROM   AdminDeduction
    WHERE  EventID = @EventID AND TargetType = 'GUILD' AND IsReverted = 0
    GROUP BY TargetID
) d ON d.TargetID = g.GuildNum
WHERE  g.EventID = @EventID;

UPDATE k
SET    k.Deduction = COALESCE(d.TotalDeduction, 0)
FROM   KOTHScore k
LEFT JOIN (
    SELECT TargetID, SUM(Points) AS TotalDeduction
    FROM   AdminDeduction
    WHERE  EventID = @EventID AND TargetType = 'PLAYER' AND IsReverted = 0
    GROUP BY TargetID
) d ON d.TargetID = k.ChaNum
WHERE  k.EventID = @EventID;

-- ================================================================
-- STEP 8: Refresh school aggregation
-- ================================================================
MERGE SchoolEventScore AS target
USING (
    SELECT EventID, ChaSchool,
           SUM(TotalKills) AS TotalKills,
           SUM(TotalDeaths) AS TotalDeaths,
           SUM(TowerPoints) AS TotalTower
    FROM   GuildEventScore
    WHERE  EventID = @EventID
    GROUP BY EventID, ChaSchool
) AS source
ON target.EventID = source.EventID AND target.ChaSchool = source.ChaSchool
WHEN MATCHED THEN UPDATE SET
    TotalKills  = source.TotalKills,
    TotalDeaths = source.TotalDeaths,
    TotalTower  = source.TotalTower
WHEN NOT MATCHED THEN
    INSERT (EventID, ChaSchool, TotalKills, TotalDeaths, TotalTower, Deduction)
    VALUES (source.EventID, source.ChaSchool,
            source.TotalKills, source.TotalDeaths, source.TotalTower, 0);

-- Apply school-level deductions
UPDATE s
SET    s.Deduction = COALESCE(d.TotalDeduction, 0)
FROM   SchoolEventScore s
LEFT JOIN (
    SELECT TargetChaSchool, SUM(Points) AS TotalDeduction
    FROM   AdminDeduction
    WHERE  EventID = @EventID AND TargetType = 'SCHOOL' AND IsReverted = 0
    GROUP BY TargetChaSchool
) d ON d.TargetChaSchool = s.ChaSchool
WHERE  s.EventID = @EventID;

-- ================================================================
-- STEP 9: Finalize
-- ================================================================
DECLARE @TotalRows INT;
SELECT @TotalRows = COUNT(*) FROM LogAction WITH (NOLOCK)
WHERE ActionDate >= @Start AND ActionDate < @End
  AND baseMainMapID = 222 AND baseSubMapID = 0 AND Type = 2;

UPDATE MatchSchedule
SET    TallyStatus      = 'Completed',
       TallyCompletedAt = GETDATE(),
       RowsTallied      = @TotalRows,
       KillsCounted     = @ValidKills,
       KillsRejected    = @TotalRows - @ValidKills
WHERE  MatchID = @MatchID;

-- Invalidate Redis cache (application code calls these):
-- DEL event:{eventId}:guild_ranks
-- DEL event:{eventId}:school_ranks
-- DEL event:{eventId}:koth:*

DROP TABLE #ValidKills, #CharFinal, #GuildFinal;
```

### Phase D: Snapshot Creation (Immediately After Tally)

```sql
DECLARE @SnapshotID UNIQUEIDENTIFIER = NEWID();

INSERT INTO EventSnapshot (SnapshotID, EventID, MatchID, SnapshotTime, Label)
VALUES (@SnapshotID, @EventID, @MatchID, GETDATE(), @MatchLabel);

INSERT INTO SnapshotGuildScore
SELECT @SnapshotID, GuildNum, GuildName, ChaSchool,
       TotalKills, TotalDeaths, TowerPoints, Deduction, ComputedScore,
       RANK() OVER (ORDER BY ComputedScore DESC)
FROM   GuildEventScore
WHERE  EventID = @EventID;

INSERT INTO SnapshotKOTHScore
SELECT @SnapshotID, ChaNum, CharName, ChaClass, GuildNum, GuildName,
       TotalKills, TotalDeaths, Deduction,
       RANK() OVER (PARTITION BY ChaClass ORDER BY TotalKills DESC)
FROM   KOTHScore
WHERE  EventID = @EventID
  AND  TotalKills > 0;

INSERT INTO SnapshotSchoolScore
SELECT @SnapshotID, ChaSchool,
       TotalKills, TotalDeaths, TotalTower, Deduction, ComputedScore,
       RANK() OVER (ORDER BY ComputedScore DESC)
FROM   SchoolEventScore
WHERE  EventID = @EventID;
```

### Phase E: Frontend Display

```
1. User opens leaderboard page

2. API checks match status FIRST:
   SELECT TOP 1 * FROM MatchSchedule
   WHERE EventID = @EventID
     AND GETDATE() BETWEEN StartTime AND DATEADD(MINUTE, 30, EndTime)
     AND TallyStatus IN ('Pending','Waiting','Processing')

   IF match is live or tally is pending:
     → Return { matchOngoing: true, nextUpdateAt: TallyScheduled }
     → Frontend renders "match ongoing" banner
     → Leaderboard still shows last completed tally (stale but accurate)

   IF no active match:
     → Return cached leaderboard JSON from Redis

3. Leaderboard endpoints serve from Redis (see Section 7)

4. Snapshot history serves from immutable snapshot tables
```

---

## 5. Optimization Strategies

### 5.1 Post-Match Batch (The Core Advantage)

| Concern               | Real-time (rejected)                | Post-match batch (chosen)                    |
| --------------------- | ----------------------------------- | -------------------------------------------- |
| DB load during match  | Continuous reads every 15-30s       | **Zero** — no reads during match             |
| Correctness guarantee | Eventual consistency                | **Full consistency** — single atomic tally   |
| Recovery from failure | Complex cursor reconciliation       | **Re-run the tally** — idempotent            |
| Audit trail           | Must reconstruct from cursor + logs | **MatchResult tables** — per-match breakdown |
| Frontend complexity   | Real-time polling + partial updates | **Simple** — show banner during match        |

### 5.2 Query Optimization for the Tally

The tally job's heaviest query hits LogAction once per match. With the filtered index:

```sql
-- Covered by IX_LogAction_MatchTally
SELECT ChaNum, TargetNum
FROM   LogAction WITH (NOLOCK)
WHERE  ActionDate >= '2026-03-18 20:00:00'
  AND  ActionDate <  '2026-03-18 20:45:00'
  AND  baseMainMapID = 222
  AND  baseSubMapID  = 0
  AND  Type = 2
```

Expected volume: a 45-minute match with 2,000 players generates ~50K–200K kill events. Single bounded range scan — fast even without the filtered index, trivial with it.

### 5.3 Redis Cache Strategy

Since scores only change after a tally completes, the cache lives much longer than a real-time model:

```
┌─────────────────────────────────────────────────┐
│ Key Pattern                  │ TTL     │ Note   │
├──────────────────────────────┼─────────┼────────┤
│ event:{id}:guild_ranks       │ 24h     │ Flushed│
│ event:{id}:school_ranks      │ 24h     │ after  │
│ event:{id}:koth:{chaClass}   │ 24h     │ each   │
│ event:{id}:match_status      │ 60s     │ tally  │
│ event:{id}:snapshot:{sid}    │ ∞       │        │
└──────────────────────────────┴─────────┴────────┘
```

After a tally completes, the job explicitly DELetes cache keys. They rebuild lazily on the next request. Between tallies, cache hit rate is 100%.

### 5.4 Idempotent Re-Tally

If the tally fails or an admin wants to re-run it:

```sql
-- Subtract this match's contribution before re-processing
UPDATE g
SET    g.TotalKills  = g.TotalKills  - m.MatchKills,
       g.TotalDeaths = g.TotalDeaths - m.MatchDeaths
FROM   GuildEventScore g
JOIN   MatchGuildResult m ON m.GuildNum = g.GuildNum AND m.MatchID = @MatchID
WHERE  g.EventID = @EventID;

UPDATE k
SET    k.TotalKills  = k.TotalKills  - m.MatchKills,
       k.TotalDeaths = k.TotalDeaths - m.MatchDeaths
FROM   KOTHScore k
JOIN   MatchCharResult m ON m.ChaNum = k.ChaNum AND m.MatchID = @MatchID
WHERE  k.EventID = @EventID;

DELETE FROM MatchGuildResult WHERE MatchID = @MatchID;
DELETE FROM MatchCharResult  WHERE MatchID = @MatchID;

UPDATE MatchSchedule SET TallyStatus = 'Waiting' WHERE MatchID = @MatchID;

-- Now re-run the tally job normally
```

### 5.5 In-Memory Registration Lookup

The tally job can load the full registration table into memory at the start of each run:

```python
# Pseudocode
reg_cache = {}
rows = db.query("""
    SELECT ChaNum, GuildNum, ChaSchool, ChaClass, JoinGvG, JoinKOTH
    FROM EventRegistration
    WHERE EventID = %s AND Status = 1
""", event_id)
for row in rows:
    reg_cache[row.ChaNum] = row
```

The registration table is small (thousands of rows) and fits in memory. This avoids repeated JOINs if the tally logic is implemented in application code rather than pure SQL.

### 5.6 Optimization Summary

| Strategy                                    | What It Solves                         |
| ------------------------------------------- | -------------------------------------- |
| Post-match batch (not real-time)            | Zero DB load during combat             |
| ActionDate range filter + filtered index    | Fast bounded scan of LogAction         |
| Dual INNER JOIN (both parties registered)   | Prevents farming unregistered alts     |
| Only process Type = 2 (killer perspective)  | Halves rows, prevents double-counting  |
| Redis cache with 24h TTL (flushed on tally) | Frontend never hits DB between tallies |
| Denormalized columns (GuildName, ChaSchool) | Avoids JOINs on display queries        |
| Computed/persisted `ComputedScore` column   | Score calculation is zero-cost at read |
| Per-match result tables                     | Enables safe re-tally and auditing     |
| Snapshot pre-ranked data                    | Historical queries are instant lookups |

---

## 6. Example Queries

### 6.1 Match Status Check (Frontend Polls This)

```sql
SELECT
    MatchID,
    MatchLabel,
    StartTime,
    EndTime,
    TallyScheduled,
    TallyStatus,
    CASE
        WHEN GETDATE() BETWEEN StartTime AND EndTime THEN 'LIVE'
        WHEN GETDATE() BETWEEN EndTime AND TallyScheduled
             AND TallyStatus IN ('Waiting','Processing') THEN 'TALLYING_SOON'
        WHEN TallyStatus = 'Processing' THEN 'TALLYING_NOW'
        ELSE 'IDLE'
    END AS DisplayStatus
FROM MatchSchedule
WHERE EventID = @EventID
  AND (
    GETDATE() BETWEEN StartTime AND EndTime
    OR (TallyStatus IN ('Waiting','Processing')
        AND TallyScheduled <= DATEADD(HOUR, 1, GETDATE()))
  )
ORDER BY StartTime DESC;
```

### 6.2 Guild Leaderboard

```sql
SELECT g.GuildNum, g.GuildName, g.ChaSchool,
       rs.SchoolName, rs.SchoolAbbr,
       g.TotalKills, g.TotalDeaths, g.TowerPoints,
       g.Deduction, g.ComputedScore,
       RANK() OVER (ORDER BY g.ComputedScore DESC) AS Rank
FROM   GuildEventScore g
LEFT JOIN RefSchool rs ON rs.ChaSchool = g.ChaSchool
WHERE  g.EventID = @EventID
ORDER BY g.ComputedScore DESC
OFFSET 0 ROWS FETCH NEXT 20 ROWS ONLY;
```

### 6.3 School Leaderboard

```sql
SELECT s.ChaSchool,
       rs.SchoolName, rs.SchoolAbbr,
       s.TotalKills, s.TotalDeaths, s.TotalTower,
       s.Deduction, s.ComputedScore,
       RANK() OVER (ORDER BY s.ComputedScore DESC) AS Rank
FROM   SchoolEventScore s
LEFT JOIN RefSchool rs ON rs.ChaSchool = s.ChaSchool
WHERE  s.EventID = @EventID
ORDER BY s.ComputedScore DESC;
```

### 6.4 KOTH Per-Class Leaderboard

```sql
SELECT k.ChaNum, k.CharName, k.ChaClass,
       rc.ClassName, k.ChaSchool,
       k.GuildNum, k.GuildName,
       k.TotalKills, k.TotalDeaths, k.Deduction,
       (k.TotalKills - k.Deduction) AS AdjustedKills,
       RANK() OVER (ORDER BY (k.TotalKills - k.Deduction) DESC) AS Rank
FROM   KOTHScore k
LEFT JOIN RefClass rc ON rc.ChaClass = k.ChaClass
WHERE  k.EventID = @EventID
  AND  k.ChaClass = @ChaClass
  AND  k.TotalKills > 0
ORDER BY AdjustedKills DESC
OFFSET 0 ROWS FETCH NEXT 20 ROWS ONLY;
```

### 6.5 Per-Match Breakdown (Audit View)

```sql
SELECT ms.MatchLabel, ms.StartTime,
       mgr.GuildNum, mgr.GuildName,
       mgr.MatchKills, mgr.MatchDeaths, mgr.MatchTower,
       (mgr.MatchTower + mgr.MatchKills - mgr.MatchDeaths) AS MatchScore
FROM   MatchGuildResult mgr
JOIN   MatchSchedule ms ON ms.MatchID = mgr.MatchID
WHERE  mgr.GuildNum = @GuildNum
  AND  ms.EventID = @EventID
ORDER BY ms.StartTime;
```

### 6.6 Snapshot Comparison (Rank Movement)

```sql
SELECT curr.GuildNum, curr.GuildName,
       prev.Rank AS PrevRank,
       curr.Rank AS CurrRank,
       prev.Rank - curr.Rank AS RankChange,
       curr.ComputedScore - prev.ComputedScore AS ScoreDelta
FROM   SnapshotGuildScore curr
  JOIN SnapshotGuildScore prev ON prev.GuildNum = curr.GuildNum
                               AND prev.SnapshotID = @PrevSnapshotID
WHERE  curr.SnapshotID = @CurrSnapshotID
ORDER BY curr.Rank;
```

### 6.7 Admin Deduction History

```sql
SELECT d.DeductionID, d.TargetType, d.TargetID,
       d.TargetChaSchool, d.Points, d.Reason,
       d.AppliedAt, d.IsReverted, d.MatchID,
       a.AdminName
FROM   AdminDeduction d
JOIN   AdminUsers a ON a.AdminUserID = d.AdminUserID
WHERE  d.EventID = @EventID
ORDER BY d.AppliedAt DESC;
```

### 6.8 Registration Validation

```sql
SELECT
    CASE
        WHEN EXISTS (
            SELECT 1 FROM EventRegistration
            WHERE EventID = @EventID AND ChaNum = @ChaNum
        ) THEN 'DUPLICATE'
        WHEN NOT EXISTS (
            SELECT 1 FROM EventDefinition
            WHERE EventID = @EventID
              AND GETDATE() BETWEEN RegistrationOpen AND RegistrationClose
        ) THEN 'CLOSED'
        WHEN (
            SELECT ChaLevel FROM ChaInfo WHERE ChaNum = @ChaNum
        ) < (
            SELECT MinLevel FROM EventDefinition WHERE EventID = @EventID
        ) THEN 'LEVEL_REQ'
        WHEN @JoinGvG = 1 AND (
            SELECT GuildNum FROM ChaInfo WHERE ChaNum = @ChaNum
        ) IS NULL THEN 'NO_GUILD'
        ELSE 'OK'
    END AS ValidationResult;
```

### 6.9 Kill Trading Detection (Post-Tally Audit)

```sql
SELECT
    v1.AttackerChaNum AS Player1,
    v1.VictimChaNum   AS Player2,
    COUNT(*)          AS MutualKills
FROM #ValidKills v1
WHERE EXISTS (
    SELECT 1 FROM #ValidKills v2
    WHERE v2.AttackerChaNum = v1.VictimChaNum
      AND v2.VictimChaNum   = v1.AttackerChaNum
)
GROUP BY v1.AttackerChaNum, v1.VictimChaNum
HAVING COUNT(*) > 15;
```

---

## 7. API + Frontend Structure

### 7.1 All API Endpoints

| Method   | Path                                                         | Purpose                                 | Cache |
| -------- | ------------------------------------------------------------ | --------------------------------------- | ----- |
| `GET`    | `/api/events/current`                                        | Active event info + registration window | 300s  |
| `GET`    | `/api/events/{eventId}/characters`                           | UserNum's eligible characters           | —     |
| `POST`   | `/api/events/{eventId}/register`                             | Register a character                    | —     |
| `PUT`    | `/api/events/{eventId}/register/{chaNum}`                    | Update categories                       | —     |
| `DELETE` | `/api/events/{eventId}/register/{chaNum}`                    | Withdraw (Status=2)                     | —     |
| `GET`    | `/api/events/{eventId}/registration-status`                  | Check registration for UserNum          | —     |
| `GET`    | `/api/events/{eventId}/match/status`                         | Current match state for banner          | 60s   |
| `GET`    | `/api/events/{eventId}/matches`                              | List all matches + tally status         | 300s  |
| `GET`    | `/api/events/{eventId}/matches/{matchId}/results`            | Per-match breakdown                     | 24h   |
| `GET`    | `/api/events/{eventId}/leaderboard/guilds?page=&limit=`      | Guild rankings                          | 24h\* |
| `GET`    | `/api/events/{eventId}/leaderboard/schools`                  | School rankings                         | 24h\* |
| `GET`    | `/api/events/{eventId}/leaderboard/koth?class=&page=&limit=` | KOTH per-class                          | 24h\* |
| `GET`    | `/api/events/{eventId}/snapshots`                            | Snapshot list                           | 3600s |
| `GET`    | `/api/events/{eventId}/snapshots/{sid}/guilds`               | Frozen guild standings                  | ∞     |
| `GET`    | `/api/events/{eventId}/snapshots/{sid}/koth?class=`          | Frozen KOTH standings                   | ∞     |
| `GET`    | `/api/events/{eventId}/snapshots/{sid}/schools`              | Frozen school standings                 | ∞     |
| `POST`   | `/api/admin/events/{eventId}/deduction`                      | Impose a penalty                        | —     |
| `PUT`    | `/api/admin/deductions/{deductionId}/revert`                 | Undo a penalty                          | —     |
| `POST`   | `/api/admin/matches/{matchId}/retally`                       | Force re-tally                          | —     |
| `POST`   | `/api/admin/events`                                          | Create event (EventDefinition)          | —     |
| `PUT`    | `/api/admin/events/{eventId}`                                | Update event details                    | —     |
| `PATCH`  | `/api/admin/events/{eventId}/activate`                       | Activate event                          | —     |
| `PATCH`  | `/api/admin/events/{eventId}/deactivate`                     | Deactivate event                        | —     |
| `GET`    | `/api/admin/events`                                          | List all events (admin view)            | —     |
| `GET`    | `/api/admin/events/{eventId}`                                | Full event detail + match list          | —     |
| `GET`    | `/api/admin/events/{eventId}/dashboard`                      | Event overview + stats                  | —     |
| `POST`   | `/api/admin/events/{eventId}/matches`                        | Create match(es)                        | —     |
| `POST`   | `/api/admin/events/{eventId}/matches/generate`               | Auto-generate schedule from pattern     | —     |
| `PUT`    | `/api/admin/matches/{matchId}`                               | Update match (Pending only)             | —     |
| `DELETE` | `/api/admin/matches/{matchId}`                               | Delete match (Pending only)             | —     |

_\*Flushed after each tally completes_

### 7.2 API Response Shapes

**Registration Payload:**

```json
// POST /api/events/{eventId}/register
{
  "chaNum": 71996,
  "joinGvG": true,
  "joinKOTH": true
}
```

**Registration Response:**

```json
{
  "success": true,
  "registration": {
    "registrationId": "f9e8d7c6-...",
    "chaNum": 71996,
    "characterName": "xDarkSlayerx",
    "chaClass": 3,
    "className": "Assassin",
    "chaLevel": 285,
    "guildNum": 5012,
    "guildName": "BloodPact",
    "chaSchool": 1,
    "schoolName": "Sacred Gate",
    "categories": ["GvG", "KOTH"],
    "registeredAt": "2026-03-15T14:22:00Z"
  }
}
```

**Match Status:**

```json
{
  "eventId": "a1b2c3d4-...",
  "currentMatch": {
    "matchId": "e5f6a7b8-...",
    "label": "Week 1 - Friday",
    "startTime": "2026-03-20T20:00:00Z",
    "endTime": "2026-03-20T20:45:00Z",
    "tallyScheduled": "2026-03-20T21:15:00Z",
    "displayStatus": "LIVE",
    "message": "The match is ongoing. Total tally will be updated after the match. Check back after 9:15 PM."
  },
  "leaderboardStale": true,
  "lastTallyCompletedAt": "2026-03-18T21:16:42Z"
}
```

**Guild Rankings:**

```json
{
  "eventId": "a1b2c3d4-...",
  "updatedAt": "2026-03-18T21:16:42Z",
  "matchesTallied": 1,
  "totalMatches": 8,
  "pagination": { "page": 1, "pageSize": 20, "totalItems": 87 },
  "rankings": [
    {
      "rank": 1,
      "guildNum": 5012,
      "guildName": "BloodPact",
      "chaSchool": 1,
      "schoolName": "Sacred Gate",
      "schoolAbbr": "SG",
      "kills": 1842,
      "deaths": 923,
      "towerPoints": 450,
      "deduction": 0,
      "score": 1369,
      "matchBreakdown": [
        {
          "matchId": "e5f6a7b8-...",
          "label": "Wed",
          "kills": 1842,
          "deaths": 923,
          "tower": 450
        }
      ]
    }
  ]
}
```

**School Rankings:**

```json
{
  "eventId": "a1b2c3d4-...",
  "updatedAt": "2026-03-18T21:16:42Z",
  "rankings": [
    {
      "rank": 1,
      "chaSchool": 1,
      "schoolName": "Sacred Gate",
      "schoolAbbr": "SG",
      "kills": 8420,
      "deaths": 5213,
      "towerPoints": 2100,
      "deduction": 0,
      "score": 5307,
      "guildCount": 12
    }
  ]
}
```

**KOTH Per-Class:**

```json
{
  "eventId": "a1b2c3d4-...",
  "chaClass": 3,
  "className": "Assassin",
  "updatedAt": "2026-03-18T21:16:42Z",
  "pagination": { "page": 1, "pageSize": 20, "totalItems": 142 },
  "rankings": [
    {
      "rank": 1,
      "chaNum": 71997,
      "characterName": "ShadowBlade",
      "guildNum": 5012,
      "guildName": "BloodPact",
      "kills": 312,
      "deaths": 89,
      "deduction": 0,
      "adjustedKills": 312,
      "kdr": 3.51
    }
  ]
}
```

**Snapshot List:**

```json
{
  "eventId": "a1b2c3d4-...",
  "snapshots": [
    {
      "snapshotId": "aa11bb22-...",
      "matchId": "e5f6a7b8-...",
      "time": "2026-03-18T21:16:42Z",
      "label": "Week 1 - Wednesday"
    },
    {
      "snapshotId": "cc33dd44-...",
      "matchId": "f6g7h8i9-...",
      "time": "2026-03-20T21:17:05Z",
      "label": "Week 1 - Friday"
    }
  ]
}
```

**Admin Deduction Request:**

```json
// POST /api/admin/events/{eventId}/deduction
{
  "targetType": "GUILD",
  "targetId": 5012,
  "targetChaSchool": null,
  "points": 50,
  "reason": "Guild BloodPact forfeited Sunday match by leaving event early.",
  "matchId": "e5f6a7b8-..."
}
```

**Admin Deduction (School):**

```json
{
  "targetType": "SCHOOL",
  "targetId": null,
  "targetChaSchool": 2,
  "points": 500,
  "reason": "Mystic Peak school withdrew from the event after Week 1.",
  "matchId": null
}
```

**Admin Deduction Response:**

```json
{
  "success": true,
  "deductionId": "ee55ff66-...",
  "applied": {
    "targetType": "GUILD",
    "targetName": "BloodPact",
    "points": 50,
    "previousScore": 1369,
    "newScore": 1319
  }
}
```

### 7.3 Frontend State Machine

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│  Frontend polls GET /match/status every 60 seconds       │
│                                                          │
│  displayStatus === "IDLE"                                │
│    → Show leaderboards normally (from cache)             │
│    → No banner                                           │
│                                                          │
│  displayStatus === "LIVE"                                │
│    → Show prominent banner:                              │
│      "⚔ The match is ongoing! Total tally will be       │
│       updated after the match."                          │
│    → Leaderboards still visible but marked "as of        │
│      last match" with grayed styling                     │
│    → Disable refresh button                              │
│                                                          │
│  displayStatus === "TALLYING_SOON"                       │
│    → Show banner:                                        │
│      "Match ended. Tally processing at 9:15 PM..."      │
│    → Leaderboards still showing previous data            │
│                                                          │
│  displayStatus === "TALLYING_NOW"                        │
│    → Show banner with spinner:                           │
│      "Crunching the numbers..."                          │
│    → Poll more frequently (every 10s)                    │
│                                                          │
│  Transition to "IDLE" after tally completes:             │
│    → Remove banner                                       │
│    → Fetch fresh leaderboard data                        │
│    → Show "Updated!" flash notification                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### 7.4 Frontend Tips

- Use **tabs** for KOTH classes (8 tabs). Only fetch the active tab's data; lazy-load others.
- Show **rank change arrows** (↑3, ↓1) by comparing current rank to last snapshot rank.
- Use **polling** (every 60s) during event hours; stop polling outside event time.
- Cache the last response in memory so the page renders instantly while fresh data loads.

---

## 8. Edge Cases + Admin Deductions

### 8.1 A Guild / School Decides to End the Event Early

This is the primary use case for admin deductions.

**Scenario A: A guild forfeits a match** (leaves the map mid-fight)

```
POST /admin/events/{eventId}/deduction
{ "targetType": "GUILD", "targetId": 5012, "points": 100,
  "reason": "BloodPact forfeited match 4 — left event map at 20:22.",
  "matchId": "g7h8..." }

Effect: GuildEventScore.Deduction += 100
        ComputedScore = TowerPoints + Kills - Deaths - 100
```

**Scenario B: An entire school withdraws**

```
POST /admin/events/{eventId}/deduction
{ "targetType": "SCHOOL", "targetChaSchool": 2, "points": 500,
  "reason": "Mystic Peak school withdrew after Week 1." }

Effect: SchoolEventScore.Deduction += 500
        Individual MP guilds keep their scores,
        but the school aggregate is penalized.
```

**Scenario C: A player caught exploiting**

```
POST /admin/events/{eventId}/deduction
{ "targetType": "PLAYER", "targetId": 71996, "points": 50,
  "reason": "Kill trading — 47 mutual kills with ChaNum 71997 in Match 2.",
  "matchId": "c3d4..." }
```

### 8.2 Admin Deduction Application Flow

```
Admin submits deduction via admin panel
         │
         ▼
┌──────────────────────────────┐
│ Validate:                    │
│ - Admin has permission       │
│ - Event exists               │
│ - Target exists              │
│ - Points > 0                 │
│ - Reason is not empty        │
└──────────────┬───────────────┘
               ▼
┌──────────────────────────────┐
│ INSERT into AdminDeduction   │
│ (full audit trail)           │
└──────────────┬───────────────┘
               ▼
┌──────────────────────────────────────────────────────┐
│ Apply to score table immediately:                    │
│                                                      │
│ Recalculate Deduction = SUM(Points)                  │
│ FROM AdminDeduction WHERE IsReverted = 0             │
│ for the specific target                              │
│                                                      │
│ IF GUILD  → UPDATE GuildEventScore.Deduction         │
│ IF PLAYER → UPDATE KOTHScore.Deduction               │
│ IF SCHOOL → UPDATE SchoolEventScore.Deduction        │
└──────────────┬───────────────────────────────────────┘
               ▼
┌──────────────────────────────┐
│ Invalidate Redis cache       │
│ DEL event:{id}:guild_ranks   │
│ DEL event:{id}:school_ranks  │
│ DEL event:{id}:koth:*        │
└──────────────┬───────────────┘
               ▼
       Return success + new score
```

### 8.3 Reverting a Deduction

```sql
UPDATE AdminDeduction
SET    IsReverted = 1, RevertedAt = GETDATE(), RevertedBy = @AdminID
WHERE  DeductionID = @DeductionID;

-- Recalculate: SUM of non-reverted deductions for the target
-- (same logic as Step 7 in the tally job)
```

### 8.4 Players Switching Guilds Mid-Event

Guild is locked at registration time. The `GuildNum` in `EventRegistration` is what the tally job uses, not the player's current guild in `ChaInfo`. If a player leaves their guild, their kills still count toward the guild they registered with.

### 8.5 Duplicate Logs / Inconsistent Kill Pairs

We only process `Type = 2` (killer perspective). This eliminates double-counting entirely. If a Type=2 row exists, the kill is counted once — even if the matching Type=1 row is missing or delayed.

### 8.6 Registered but Inactive Players

Players with 0 kills are hidden from KOTH leaderboards (`WHERE TotalKills > 0`). They don't affect guild scores since 0+0-0=0.

### 8.7 ActionDate Boundary Edge Case

```sql
-- Half-open interval: [Start, End) prevents double-counting at boundaries
WHERE ActionDate >= @Start AND ActionDate < @End
```

A kill at exactly 20:45:00.000 belongs to the next window (if any), not this match.

### 8.8 Late Joins

Allow registration up to a cutoff (e.g., 24h after event start), but kills before registration are NOT counted retroactively. The tally job only credits registered players.

### 8.9 Tally Job Failure

If the job crashes mid-process:

- `TallyStatus` remains `'Processing'`
- Admin triggers a re-tally via `POST /admin/matches/{matchId}/retally`
- The re-tally subtracts the partial match results (if any), deletes them, resets status to `'Waiting'`, and re-runs
- The per-match result tables make this safe and idempotent

### 8.10 Tower Results & Tally Trigger (Pending → Waiting Transition)

The `TallyStatus` does **not** automatically change from `'Pending'` to `'Waiting'` when a match ends. Instead, the admin must **manually set tower results** after each match, which triggers the transition.

> **Note:** The previous Match Readiness Scheduler (automated Pending → Waiting at fixed times) is **obsolete** and disabled. Since there is no way to automatically detect which school dominated each tower from the game database, the admin must report tower winners manually. This action replaces the automated scheduler as the tally trigger.

**Workflow:**

1. Match ends (EndTime passes)
2. Admin determines which school won each tower (TH, Nuc, Faci)
3. Admin calls `PUT /api/admin/matches/{matchId}/tower-results`
4. Server saves tower winners and transitions `TallyStatus` from `'Pending'` → `'Waiting'`
5. Tally scheduler picks it up on its next 30s poll cycle

**Endpoint:**

```
PUT /api/admin/matches/{matchId}/tower-results
```

```json
{
  "tower_winner_th": 0,       // ChaSchool: 0=SG, 1=MP, 2=PH, null=no winner
  "tower_winner_nuc": 1,
  "tower_winner_faci": 2
}
```

- Only allowed when `TallyStatus = 'Pending'`
- Tower winner values must be valid ChaSchool (0, 1, 2) or null
- Saves tower winners and transitions to `'Waiting'`
- Resets tally metadata (`TallyStartedAt`, `TallyCompletedAt`, etc.) to NULL

**Tower Points Computation (during tally):**

Each guild earns tower points for each tower won by its school:
```sql
MatchTower =
    CASE WHEN ChaSchool = @TowerWinTH  THEN ISNULL(@TowerPtsTH, 0)  ELSE 0 END
  + CASE WHEN ChaSchool = @TowerWinNuc  THEN ISNULL(@TowerPtsNuc, 0)  ELSE 0 END
  + CASE WHEN ChaSchool = @TowerWinFaci THEN ISNULL(@TowerPtsFaci, 0) ELSE 0 END
```

Tower point values (TH, Nuc, Faci) are **rotational per match** — set by admin at match creation/edit time. Default is 0 for all towers if not set.

**Example:** Match 1: TH=25, Nuc=50, Faci=0. Match 2: TH=50, Nuc=0, Faci=25.

**Fallback — Manual Set-Waiting:**

If the admin needs to re-trigger tally without changing tower results (e.g., after a failure):

```
PATCH /api/admin/matches/{matchId}/set-waiting
```

- Only allowed when `TallyStatus` is `'Pending'` or `'Failed'`
- Resets tally metadata (`TallyStartedAt`, `TallyCompletedAt`, `RowsTallied`, etc.) to NULL
- Once set to `'Waiting'`, the tally scheduler will pick it up on its next 30s poll cycle

**Full TallyStatus State Machine:**
```
                  Admin sets tower results
                  (or admin manual set-waiting)
  Pending ──────────────────────────────────────► Waiting
                                                    │
                                         Tally Scheduler picks up
                                         (TallyScheduled <= NOW)
                                                    │
                                                    ▼
                                               Processing
                                                 /     \
                                           success     failure
                                              /           \
                                             ▼             ▼
                                        Completed       Failed
                                                          │
                                             admin re-tally or
                                             admin set-waiting
                                                          │
                                                          ▼
                                                       Waiting
```

---

## 9. Tradeoffs and Recommendations

### Architecture Decision Record

| Decision                              | Chosen                 | Alternative            | Why                                                    |
| ------------------------------------- | ---------------------- | ---------------------- | ------------------------------------------------------ |
| Post-match batch tally                | ✅ Once per match      | Real-time (15-30s)     | Zero DB load during match; simpler; auditable          |
| 30-min delay before tally             | ✅ Conservative        | Immediate post-match   | Ensures all log rows are flushed; safe margin          |
| Dual-eligibility (both parties)       | ✅ INNER JOIN both     | Check attacker only    | Prevents farming unregistered alts                     |
| Per-match result tables               | ✅ Full breakdown      | Cumulative only        | Enables re-tally, per-match auditing, match forfeit    |
| Admin deduction as separate column    | ✅ Deduction column    | Modify kills/deaths    | Transparent; original scores preserved; reversible     |
| Admin deduction audit log             | ✅ Full log table      | Just update score      | Accountability; revert capability; dispute resolution  |
| Fixed 45-min match windows            | ✅ MatchSchedule table | Config-only            | Precise ActionDate filtering; no ambiguity             |
| Frontend "match ongoing" banner       | ✅ State machine       | Live partial scores    | Simpler; no incorrect partial data shown               |
| Single registration row               | ✅ BIT flags           | Junction table         | Simpler, faster, no joins needed                       |
| Only process Type=2 logs              | ✅ Killer-only         | Both types             | Halves processing, avoids double-count                 |
| Denormalized names in score tables    | ✅ Copy on write       | JOIN at display        | Eliminates joins on hot read paths                     |
| GuildName always paired with GuildNum | ✅ Both columns        | Name only              | Can always trace back to source data                   |
| UUIDs for system IDs                  | ✅ UNIQUEIDENTIFIER    | INT IDENTITY           | Safe for distributed/web systems; no collision risk    |
| ChaSchool/ChaClass as INT             | ✅ Matches ChaInfo     | VARCHAR strings        | Consistent with game DB; resolved in API layer         |
| Snapshot = full copy                  | ✅ Full copy           | Delta/diff             | Snapshots are small; deltas add complexity for no gain |
| Guild locked at registration          | ✅ Lock                | Allow mid-event change | Prevents manipulation, simplifies aggregation          |

### Critical Recommendations

1. **Pre-populate MatchSchedule for the entire event season.** Don't create match rows on the fly. The frontend always knows when the next match is, and the tally cron can find pending matches reliably.

2. **Test the tally with production-scale LogAction.** Run the ActionDate range query against your real table to measure execution time. With the filtered index, 200K rows for a 45-min window should complete in under 5 seconds.

3. **Admin deductions should be rare.** Build the admin panel with friction — require a text reason, show a confirmation dialog, and log who did what.

4. **The re-tally capability is your safety net.** If something goes wrong, re-run any match's tally cleanly because per-match results are isolated.

5. **Snapshot after every tally.** Since scores only change after tallies, take a snapshot immediately after each tally completes. One snapshot per match gives a complete progression history.

6. **Monitor the 30-minute delay.** If your game server flushes LogAction rows with significant delay, consider extending the delay.

7. **School-level deductions for mass withdrawals.** One school-level deduction covers all guilds in a school — visible as a single line item in the audit log.

8. **Use NEWSEQUENTIALID() for EventRegistration.** This is the highest-volume UUID table; sequential UUIDs prevent clustered index page splits.

9. **Start the tally job infrastructure before the first match.** Let it poll idle so you confirm it's healthy. Don't first-deploy when players are fighting.

10. **Backup before each snapshot.** Snapshots are the authoritative historical record. A corrupted snapshot can't be regenerated after scores are reset for the next event.

---

## 10. Admin Event & Match Management

This section covers how admins create and manage events and match schedules — the setup that must happen before registration opens and matches begin.

### 10.1 Admin API Endpoints

| Method   | Path                                           | Purpose                                                             |
| -------- | ---------------------------------------------- | ------------------------------------------------------------------- |
| `POST`   | `/api/admin/events`                            | Create a new event (EventDefinition)                                |
| `PUT`    | `/api/admin/events/{eventId}`                  | Update event details (before event starts)                          |
| `PATCH`  | `/api/admin/events/{eventId}/activate`         | Set `IsActive = 1`                                                  |
| `PATCH`  | `/api/admin/events/{eventId}/deactivate`       | Set `IsActive = 0`                                                  |
| `GET`    | `/api/admin/events`                            | List all events (with status summary)                               |
| `GET`    | `/api/admin/events/{eventId}`                  | Get full event detail + match list                                  |
| `POST`   | `/api/admin/events/{eventId}/matches`          | Create matches (single or bulk)                                     |
| `POST`   | `/api/admin/events/{eventId}/matches/generate` | Auto-generate match schedule from a pattern                         |
| `PUT`    | `/api/admin/matches/{matchId}`                 | Update a match (only if `TallyStatus = 'Pending'`)                  |
| `DELETE` | `/api/admin/matches/{matchId}`                 | Delete a match (only if `TallyStatus = 'Pending'`)                  |
| `GET`    | `/api/admin/events/{eventId}/matches`          | List all matches with tally status                                  |
| `PUT`    | `/api/admin/matches/{matchId}/tower-results`   | Set tower winners after match ends → triggers tally (Pending only)  |
| `PATCH`  | `/api/admin/matches/{matchId}/set-waiting`     | Manually set a match to 'Waiting' (only if Pending or Failed)       |
| `GET`    | `/api/admin/events/{eventId}/dashboard`        | Event overview: registration count, matches tallied, scores summary |

### 10.2 Create Event — Process Flow

```
Admin opens "Create Event" form
        │
        ▼
┌──────────────────────────────────────────────────┐
│ Payload:                                         │
│ {                                                │
│   "eventName": "Tyranny War Season 12",          │
│   "season": 12,                                  │
│   "eventMapMID": 222,                            │
│   "eventMapSID": 0,                              │
│   "matchDurationMin": 45,                        │
│   "tallyDelayMin": 30,                           │
│   "registrationOpen": "2026-03-10T00:00:00",     │
│   "registrationClose": "2026-03-17T23:59:59",    │
│   "eventStart": "2026-03-18T00:00:00",           │
│   "eventEnd": "2026-04-01T23:59:59",             │
│   "minLevel": 200,                               │
│   "towerPointsPerTick": 1                        │
│ }                                                │
└──────────────┬───────────────────────────────────┘
               ▼
┌──────────────────────────────────────────────────┐
│ Server-side validation:                          │
│                                                  │
│ E1. Admin has permission (admin role in JWT)     │
│                                                  │
│ E2. EventName is not empty                       │
│                                                  │
│ E3. Season is unique (no other EventDefinition   │
│     with same Season that is still active)       │
│                                                  │
│ E4. Date sequence is valid:                      │
│     RegistrationOpen < RegistrationClose         │
│     RegistrationClose <= EventStart              │
│     EventStart < EventEnd                        │
│                                                  │
│ E5. MatchDurationMin > 0 (default 45)            │
│                                                  │
│ E6. TallyDelayMin > 0 (default 30)              │
│                                                  │
│ E7. MinLevel > 0                                 │
│                                                  │
│ E8. EventMapMID and EventMapSID are valid        │
│     (222 / 0 for standard TW map)                │
└──────────────┬───────────────────────────────────┘
               │
        ┌──────┴──────┐
        │  Passed?    │
        └──┬───────┬──┘
       Yes │       │ No → return validation error
           ▼
┌──────────────────────────────────────┐
│ Generate UUID for EventID            │
│ INSERT INTO EventDefinition          │
│ IsActive defaults to 0 (draft)       │
│ Admin activates explicitly when ready│
└──────────────┬───────────────────────┘
               ▼
       Return success + EventID
```

### 10.3 Create Event — Validation Query

```sql
-- Check season uniqueness among active events
SELECT CASE
    WHEN EXISTS (
        SELECT 1 FROM EventDefinition
        WHERE Season = @Season AND IsActive = 1
    ) THEN 'SEASON_CONFLICT'
    ELSE 'OK'
END AS ValidationResult;
```

### 10.4 Create Event — Request & Response

**Request:**

```json
// POST /api/admin/events
{
  "eventName": "Tyranny War Season 12",
  "season": 12,
  "eventMapMID": 222,
  "eventMapSID": 0,
  "matchDurationMin": 45,
  "tallyDelayMin": 30,
  "registrationOpen": "2026-03-10T00:00:00",
  "registrationClose": "2026-03-17T23:59:59",
  "eventStart": "2026-03-18T00:00:00",
  "eventEnd": "2026-04-01T23:59:59",
  "minLevel": 200,
  "towerPointsPerTick": 1
}
```

**Response:**

```json
{
  "success": true,
  "event": {
    "eventId": "a1b2c3d4-...",
    "eventName": "Tyranny War Season 12",
    "season": 12,
    "isActive": false,
    "registrationOpen": "2026-03-10T00:00:00",
    "registrationClose": "2026-03-17T23:59:59",
    "eventStart": "2026-03-18T00:00:00",
    "eventEnd": "2026-04-01T23:59:59",
    "matchDurationMin": 45,
    "tallyDelayMin": 30,
    "minLevel": 200,
    "matchCount": 0,
    "registrationCount": 0,
    "createdAt": "2026-03-05T10:30:00Z"
  }
}
```

### 10.5 Update Event — Rules

Updates are only allowed under certain conditions:

| Field                | Editable When                                             |
| -------------------- | --------------------------------------------------------- |
| `EventName`          | Always                                                    |
| `RegistrationOpen`   | Before registration has opened (no registrations exist)   |
| `RegistrationClose`  | Before registration has closed                            |
| `EventStart`         | Before event has started AND no matches have been tallied |
| `EventEnd`           | Before event has ended                                    |
| `MinLevel`           | Before registration has opened (no registrations exist)   |
| `MatchDurationMin`   | Before any matches exist                                  |
| `TallyDelayMin`      | Always (affects future tallies only)                      |
| `TowerPointsPerTick` | Always                                                    |
| `IsActive`           | Always (via dedicated activate/deactivate endpoints)      |

```sql
-- Guard: check if event can be modified
SELECT
    (SELECT COUNT(*) FROM EventRegistration WHERE EventID = @EventID) AS RegCount,
    (SELECT COUNT(*) FROM MatchSchedule WHERE EventID = @EventID) AS MatchCount,
    (SELECT COUNT(*) FROM MatchSchedule WHERE EventID = @EventID AND TallyStatus = 'Completed') AS TalliedCount,
    e.RegistrationOpen,
    e.EventStart,
    e.EventEnd,
    e.IsActive
FROM EventDefinition e
WHERE e.EventID = @EventID;
```

### 10.6 Activate / Deactivate Event

**Activate** (`PATCH /api/admin/events/{eventId}/activate`):

- Sets `IsActive = 1`
- Validates: event must have at least 1 match scheduled
- This makes the event visible on the frontend and opens registration at the scheduled time

**Deactivate** (`PATCH /api/admin/events/{eventId}/deactivate`):

- Sets `IsActive = 0`
- Validates: no matches currently in `'Processing'` status
- Frontend stops showing the event
- Running tally jobs will skip inactive events

```sql
-- Activate: ensure at least one match exists
UPDATE EventDefinition
SET    IsActive = 1
WHERE  EventID = @EventID
  AND  EXISTS (SELECT 1 FROM MatchSchedule WHERE EventID = @EventID);

-- Deactivate: ensure no tally is running
UPDATE EventDefinition
SET    IsActive = 0
WHERE  EventID = @EventID
  AND  NOT EXISTS (
    SELECT 1 FROM MatchSchedule
    WHERE EventID = @EventID AND TallyStatus = 'Processing'
  );
```

### 10.7 Create Matches — Single

```json
// POST /api/admin/events/{eventId}/matches
{
  "matchLabel": "Week 1 - Wednesday",
  "matchDate": "2026-03-18",
  "startTime": "2026-03-18T20:00:00",
  "tower_points_th": 25,
  "tower_points_nuc": 50,
  "tower_points_faci": 0
}
```

Tower point fields are optional and default to 0. They are **rotational per match** — the admin sets different values each match to vary the tower importance.

Server computes `EndTime` and `TallyScheduled` from `EventDefinition.MatchDurationMin` and `TallyDelayMin`:

```sql
DECLARE @EndTime DATETIME = DATEADD(MINUTE, @MatchDurationMin, @StartTime);
DECLARE @TallyScheduled DATETIME = DATEADD(MINUTE, @TallyDelayMin, @EndTime);

INSERT INTO MatchSchedule (MatchID, EventID, MatchLabel, MatchDate, StartTime, EndTime, TallyScheduled, TallyStatus,
                           TowerPointsTH, TowerPointsNuc, TowerPointsFaci)
VALUES (NEWID(), @EventID, @MatchLabel, @MatchDate, @StartTime, @EndTime, @TallyScheduled, 'Pending',
        @TowerPointsTH, @TowerPointsNuc, @TowerPointsFaci);
```

### 10.8 Create Matches — Bulk (Manual List)

```json
// POST /api/admin/events/{eventId}/matches
{
  "matches": [
    {
      "matchLabel": "Week 1 - Wednesday",
      "matchDate": "2026-03-18",
      "startTime": "2026-03-18T20:00:00"
    },
    {
      "matchLabel": "Week 1 - Friday",
      "matchDate": "2026-03-20",
      "startTime": "2026-03-20T20:00:00"
    },
    {
      "matchLabel": "Week 1 - Saturday",
      "matchDate": "2026-03-21",
      "startTime": "2026-03-21T20:00:00"
    },
    {
      "matchLabel": "Week 1 - Sunday",
      "matchDate": "2026-03-22",
      "startTime": "2026-03-22T20:00:00"
    }
  ]
}
```

**Response:**

```json
{
  "success": true,
  "created": 4,
  "matches": [
    {
      "matchId": "c3d4e5f6-...",
      "matchLabel": "Week 1 - Wednesday",
      "startTime": "2026-03-18T20:00:00",
      "endTime": "2026-03-18T20:45:00",
      "tallyScheduled": "2026-03-18T21:15:00",
      "tallyStatus": "Pending"
    }
  ]
}
```

### 10.9 Auto-Generate Match Schedule (Pattern-Based)

For convenience, admins can generate a full schedule from a repeating pattern:

```json
// POST /api/admin/events/{eventId}/matches/generate
{
  "startDate": "2026-03-18",
  "endDate": "2026-03-29",
  "daysOfWeek": ["Wednesday", "Friday", "Saturday", "Sunday"],
  "startHour": 20,
  "startMinute": 0,
  "labelPrefix": "Week"
}
```

Server logic:

1. Iterate each date from `startDate` to `endDate`
2. If the day of week matches `daysOfWeek`, create a match
3. Auto-label: `"Week 1 - Wednesday"`, `"Week 1 - Friday"`, ..., `"Week 2 - Wednesday"`, etc.
4. Compute `EndTime = StartTime + MatchDurationMin` and `TallyScheduled = EndTime + TallyDelayMin` from the event config
5. Week number increments when the day of week wraps past the first entry in `daysOfWeek`

**Validation:**

- All generated match times must fall within `EventStart` to `EventEnd`
- No overlap with existing matches for this event (check `UQ_Match_Time`)
- Maximum 50 matches per generation call (safety limit)

**Response:**

```json
{
  "success": true,
  "created": 8,
  "matches": [
    {
      "matchId": "...",
      "matchLabel": "Week 1 - Wednesday",
      "startTime": "2026-03-18T20:00:00",
      "endTime": "2026-03-18T20:45:00",
      "tallyScheduled": "2026-03-18T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 1 - Friday",
      "startTime": "2026-03-20T20:00:00",
      "endTime": "2026-03-20T20:45:00",
      "tallyScheduled": "2026-03-20T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 1 - Saturday",
      "startTime": "2026-03-21T20:00:00",
      "endTime": "2026-03-21T20:45:00",
      "tallyScheduled": "2026-03-21T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 1 - Sunday",
      "startTime": "2026-03-22T20:00:00",
      "endTime": "2026-03-22T20:45:00",
      "tallyScheduled": "2026-03-22T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 2 - Wednesday",
      "startTime": "2026-03-25T20:00:00",
      "endTime": "2026-03-25T20:45:00",
      "tallyScheduled": "2026-03-25T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 2 - Friday",
      "startTime": "2026-03-27T20:00:00",
      "endTime": "2026-03-27T20:45:00",
      "tallyScheduled": "2026-03-27T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 2 - Saturday",
      "startTime": "2026-03-28T20:00:00",
      "endTime": "2026-03-28T20:45:00",
      "tallyScheduled": "2026-03-28T21:15:00"
    },
    {
      "matchId": "...",
      "matchLabel": "Week 2 - Sunday",
      "startTime": "2026-03-29T20:00:00",
      "endTime": "2026-03-29T20:45:00",
      "tallyScheduled": "2026-03-29T21:15:00"
    }
  ]
}
```

### 10.10 Match Schedule Validation

Every match creation (single, bulk, or auto-generated) must pass these checks:

| #   | Rule                                                                    | Error Code                               |
| --- | ----------------------------------------------------------------------- | ---------------------------------------- |
| M1  | Event exists and admin has permission                                   | `UNAUTHORIZED`                           |
| M2  | `StartTime` falls within `EventStart` to `EventEnd`                     | `OUT_OF_EVENT_RANGE`                     |
| M3  | No overlap with existing matches (same event, overlapping time windows) | `MATCH_OVERLAP`                          |
| M4  | `StartTime` is in the future                                            | `START_IN_PAST`                          |
| M5  | Event must not be deactivated (or warn if `IsActive = 0`)               | `EVENT_INACTIVE` (warning, not blocking) |

**Overlap check query:**

```sql
-- Check if a proposed match overlaps with any existing match for this event
SELECT COUNT(*) AS OverlapCount
FROM   MatchSchedule
WHERE  EventID = @EventID
  AND  (
    (@ProposedStart >= StartTime AND @ProposedStart < EndTime)    -- new start falls inside existing
    OR (@ProposedEnd > StartTime AND @ProposedEnd <= EndTime)     -- new end falls inside existing
    OR (@ProposedStart <= StartTime AND @ProposedEnd >= EndTime)  -- new fully contains existing
  );
```

### 10.11 Update Match

Only allowed when `TallyStatus = 'Pending'` (match hasn't happened yet).

```json
// PUT /api/admin/matches/{matchId}
{
  "matchLabel": "Week 1 - Wednesday (Rescheduled)",
  "startTime": "2026-03-18T21:00:00",
  "tower_points_th": 50,
  "tower_points_nuc": 0,
  "tower_points_faci": 25
}
```

All fields are optional. Tower point fields default to the existing values if not provided.

```sql
-- Guard + update
UPDATE MatchSchedule
SET    MatchLabel      = @MatchLabel,
       MatchDate       = CAST(@StartTime AS DATE),
       StartTime       = @StartTime,
       EndTime         = DATEADD(MINUTE, @MatchDurationMin, @StartTime),
       TallyScheduled  = DATEADD(MINUTE, @TallyDelayMin,
                          DATEADD(MINUTE, @MatchDurationMin, @StartTime)),
       TowerPointsTH   = @TowerPointsTH,
       TowerPointsNuc  = @TowerPointsNuc,
       TowerPointsFaci = @TowerPointsFaci
WHERE  MatchID = @MatchID
  AND  TallyStatus = 'Pending';

-- If @@ROWCOUNT = 0, match either doesn't exist or is no longer Pending
```

### 10.12 Delete Match

Only allowed when `TallyStatus = 'Pending'`.

```sql
DELETE FROM MatchSchedule
WHERE  MatchID = @MatchID
  AND  TallyStatus = 'Pending';

-- If @@ROWCOUNT = 0, match doesn't exist or is no longer Pending
```

### 10.13 Admin Event Dashboard

A convenience endpoint that gives the admin a full overview:

```json
// GET /api/admin/events/{eventId}/dashboard
{
  "event": {
    "eventId": "a1b2c3d4-...",
    "eventName": "Tyranny War Season 12",
    "season": 12,
    "isActive": true,
    "registrationOpen": "2026-03-10T00:00:00",
    "registrationClose": "2026-03-17T23:59:59",
    "eventStart": "2026-03-18T00:00:00",
    "eventEnd": "2026-04-01T23:59:59"
  },
  "registrationStats": {
    "totalRegistered": 1847,
    "gvgOnly": 423,
    "kothOnly": 612,
    "both": 812,
    "withdrawn": 15,
    "disqualified": 2,
    "guildCount": 87,
    "schoolBreakdown": [
      { "chaSchool": 1, "schoolName": "Sacred Gate", "count": 634 },
      { "chaSchool": 2, "schoolName": "Mystic Peak", "count": 589 },
      { "chaSchool": 3, "schoolName": "Phoenix", "count": 624 }
    ]
  },
  "matchStats": {
    "totalMatches": 8,
    "completed": 3,
    "pending": 5,
    "nextMatch": {
      "matchId": "e5f6a7b8-...",
      "label": "Week 1 - Sunday",
      "startTime": "2026-03-22T20:00:00"
    }
  },
  "deductionStats": {
    "totalDeductions": 3,
    "totalPointsDeducted": 200,
    "reverted": 1
  }
}
```

**Query for registration stats:**

```sql
SELECT
    COUNT(*) AS TotalRegistered,
    SUM(CASE WHEN JoinGvG = 1 AND JoinKOTH = 0 THEN 1 ELSE 0 END) AS GvgOnly,
    SUM(CASE WHEN JoinGvG = 0 AND JoinKOTH = 1 THEN 1 ELSE 0 END) AS KothOnly,
    SUM(CASE WHEN JoinGvG = 1 AND JoinKOTH = 1 THEN 1 ELSE 0 END) AS Both,
    SUM(CASE WHEN Status = 2 THEN 1 ELSE 0 END) AS Withdrawn,
    SUM(CASE WHEN Status = 3 THEN 1 ELSE 0 END) AS Disqualified,
    COUNT(DISTINCT GuildNum) AS GuildCount
FROM   EventRegistration
WHERE  EventID = @EventID;

-- School breakdown
SELECT ChaSchool, COUNT(*) AS Count
FROM   EventRegistration
WHERE  EventID = @EventID AND Status = 1
GROUP BY ChaSchool;
```

### 10.14 Event Lifecycle State Machine

```
┌───────────────────────────────────────────────────────────┐
│                                                           │
│  DRAFT (IsActive = 0, no registrations)                   │
│    Admin can:                                             │
│    - Edit all fields freely                               │
│    - Create/delete matches                                │
│    - Activate when ready                                  │
│                                                           │
│    ──── Admin activates ────▶                             │
│                                                           │
│  ACTIVE (IsActive = 1)                                    │
│    │                                                      │
│    ├── REGISTRATION OPEN (NOW between Open and Close)     │
│    │     Players can register                             │
│    │     Admin can still edit matches (if Pending)        │
│    │     Admin cannot change MinLevel or RegistrationOpen │
│    │                                                      │
│    ├── REGISTRATION CLOSED (NOW > RegistrationClose)      │
│    │     No new registrations                             │
│    │     Players can withdraw (Status=2)                  │
│    │                                                      │
│    ├── EVENT RUNNING (NOW between EventStart and EventEnd)│
│    │     Matches fire according to MatchSchedule          │
│    │     Tally jobs run after each match                  │
│    │     Admin can impose deductions                      │
│    │     Admin can trigger re-tally                       │
│    │     Admin can still add future matches               │
│    │                                                      │
│    └── EVENT ENDED (NOW > EventEnd)                       │
│          No more matches                                  │
│          Final standings are in snapshots                  │
│          Admin can still impose/revert deductions          │
│          Admin can deactivate to archive                  │
│                                                           │
│    ──── Admin deactivates ────▶                           │
│                                                           │
│  ARCHIVED (IsActive = 0, has historical data)             │
│    Read-only: leaderboards + snapshots still accessible   │
│    No new matches, registrations, or tallies              │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### 10.15 Failure Cases

| Failure                         | Code                 | Message                                                                       |
| ------------------------------- | -------------------- | ----------------------------------------------------------------------------- |
| Season already exists (active)  | `SEASON_CONFLICT`    | "An active event for Season {n} already exists."                              |
| Invalid date sequence           | `INVALID_DATES`      | "Registration must open before it closes, and close before the event starts." |
| Match time outside event range  | `OUT_OF_EVENT_RANGE` | "Match start time must be within the event date range."                       |
| Match overlaps existing match   | `MATCH_OVERLAP`      | "This match overlaps with '{existingLabel}' ({existingStart})."               |
| Match start in the past         | `START_IN_PAST`      | "Match start time must be in the future."                                     |
| Cannot edit non-pending match   | `MATCH_LOCKED`       | "Cannot modify a match that has already been tallied or is in progress."      |
| Cannot delete non-pending match | `MATCH_LOCKED`       | "Cannot delete a match that has already been tallied."                        |
| Cannot activate without matches | `NO_MATCHES`         | "Add at least one match before activating the event."                         |
| Cannot deactivate during tally  | `TALLY_IN_PROGRESS`  | "A tally is currently running. Wait for it to complete."                      |
| Event not found                 | `NOT_FOUND`          | "Event not found."                                                            |
