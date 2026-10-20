USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_ExecuteTally]
    @EventID  UNIQUEIDENTIFIER,
    @MatchID  UNIQUEIDENTIFIER,
    @StartTime DATETIME,
    @EndTime   DATETIME
AS
BEGIN
    SET NOCOUNT ON;

    -- STEP 0: Load event config from EventDefinition
    DECLARE @MinGuildMembers INT, @EventMapMID INT, @EventMapSID INT;
    SELECT @MinGuildMembers = ISNULL(MinGuildMembers, 0),
           @EventMapMID     = ISNULL(EventMapMID, 222),
           @EventMapSID     = ISNULL(EventMapSID, 0)
    FROM EventDefinition WHERE EventID = @EventID;

    -- STEP 1: Load tower config for this match
    DECLARE @TowerPtsTH  INT, @TowerPtsNuc  INT, @TowerPtsFaci  INT;
    DECLARE @TowerWinTH  INT, @TowerWinNuc  INT, @TowerWinFaci  INT;

    SELECT @TowerPtsTH   = TowerPointsTH,
           @TowerPtsNuc  = TowerPointsNuc,
           @TowerPtsFaci = TowerPointsFaci,
           @TowerWinTH   = TowerWinnerTH,
           @TowerWinNuc  = TowerWinnerNuc,
           @TowerWinFaci = TowerWinnerFaci
    FROM MatchSchedule WHERE MatchID = @MatchID;

    -- STEP 1a: Load gimmick windows for this match.
    -- If no gimmicks exist, #Gimmicks is empty and all flags below evaluate to 0 (no-op).
    SELECT GimmickID, GimmickType, GimmickStartTime, GimmickEndTime
    INTO #Gimmicks
    FROM EventMatchGimmick
    WHERE MatchID = @MatchID;

    -- STEP 2: Extract valid kills from LogAction (both attacker AND victim registered).
    -- Deduplicates by taking the MIN(ActionNum) per (ChaNum, TargetNum, ActionDate)
    -- to handle game-side duplicate log entries for the same kill event.
    -- DoublePointsHit / NullDeathsHit: 1 if the kill's ActionDate falls inside ANY active
    -- gimmick window of that type, 0 otherwise. Using EXISTS (not COUNT) ensures overlapping
    -- same-type windows never stack beyond 1 (no 4x doubling).
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
        RV.JoinKOTH        AS VictimInKOTH,
        CAST(CASE WHEN EXISTS (
            SELECT 1 FROM #Gimmicks g
            WHERE g.GimmickType = 'DoublePoints'
              AND L.ActionDate >= g.GimmickStartTime
              AND L.ActionDate <  g.GimmickEndTime
        ) THEN 1 ELSE 0 END AS INT) AS DoublePointsHit,
        CAST(CASE WHEN EXISTS (
            SELECT 1 FROM #Gimmicks g
            WHERE g.GimmickType = 'NullDeaths'
              AND L.ActionDate >= g.GimmickStartTime
              AND L.ActionDate <  g.GimmickEndTime
        ) THEN 1 ELSE 0 END AS INT) AS NullDeathsHit
    INTO #ValidKills
    FROM (
        -- Deduplicate: keep only one row per (ChaNum, TargetNum, ActionDate) by selecting MIN ActionNum
        SELECT MIN(ActionNum) AS ActionNum, ChaNum, TargetNum, ActionDate
        FROM [RanLogS1].[dbo].[LogAction] WITH (NOLOCK)
        WHERE ActionDate >= @StartTime
          AND ActionDate <  @EndTime
          AND baseMainMapID = @EventMapMID
          AND baseSubMapID  = @EventMapSID
          AND Type = 2
        GROUP BY ChaNum, TargetNum, ActionDate
    ) AS Deduped
    JOIN [RanLogS1].[dbo].[LogAction] L WITH (NOLOCK) ON L.ActionNum = Deduped.ActionNum
    INNER JOIN EventRegistration RA
        ON  RA.EventID = @EventID
        AND RA.ChaNum  = L.ChaNum
        AND RA.Status  = 1
    INNER JOIN EventRegistration RV
        ON  RV.EventID = @EventID
        AND RV.ChaNum  = L.TargetNum
        AND RV.Status  = 1;

    -- STEP 3: Aggregate per-character
    -- Kill credit  = 1 + DoublePointsHit (1 or 2 per event; never 4x from overlapping windows)
    -- Death credit = 1 - NullDeathsHit   (1 or 0 per event)
    SELECT
        ChaNum,
        MAX(ChaName) AS ChaName,
        MAX(ChaClass) AS ChaClass,
        MAX(GuildNum) AS GuildNum,
        SUM(MatchKills) AS MatchKills,
        SUM(MatchDeaths) AS MatchDeaths
    INTO #CharFinal
    FROM (
        SELECT AttackerChaNum AS ChaNum, CAST(NULL AS NVARCHAR(64)) AS ChaName,
               AttackerClass AS ChaClass, AttackerGuild AS GuildNum,
               SUM(1 + DoublePointsHit) AS MatchKills, 0 AS MatchDeaths
        FROM   #ValidKills
        GROUP BY AttackerChaNum, AttackerClass, AttackerGuild
        UNION ALL
        SELECT VictimChaNum AS ChaNum, CAST(NULL AS NVARCHAR(64)) AS ChaName,
               CAST(NULL AS INT) AS ChaClass, VictimGuild AS GuildNum,
               0 AS MatchKills, SUM(1 - NullDeathsHit) AS MatchDeaths
        FROM   #ValidKills
        GROUP BY VictimChaNum, VictimGuild
    ) AS CharAgg
    GROUP BY ChaNum;

    -- Fill display names from ChaInfo, use EventRegistration for mapped ChaClass
    UPDATE cf
    SET    cf.ChaName = c.ChaName,
           cf.ChaClass = COALESCE(cf.ChaClass, r.ChaClass),
           cf.GuildNum = COALESCE(cf.GuildNum, r.GuildNum)
    FROM   #CharFinal cf
    JOIN   [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = cf.ChaNum
    JOIN   EventRegistration r ON r.ChaNum = cf.ChaNum AND r.EventID = @EventID
    WHERE  cf.ChaName IS NULL;

    -- STEP 4: Aggregate per-guild (GvG only)
    -- Same gimmick-aware kill/death emission as STEP 3.
    SELECT
        GuildNum,
        MAX(ChaSchool) AS ChaSchool,
        SUM(MatchKills) AS MatchKills,
        SUM(MatchDeaths) AS MatchDeaths
    INTO #GuildFinal
    FROM (
        SELECT AttackerGuild AS GuildNum, AttackerSchool AS ChaSchool,
               SUM(1 + DoublePointsHit) AS MatchKills, 0 AS MatchDeaths
        FROM   #ValidKills
        WHERE  AttackerInGvG = 1 AND VictimInGvG = 1
        GROUP BY AttackerGuild, AttackerSchool
        UNION ALL
        SELECT VictimGuild AS GuildNum, VictimSchool AS ChaSchool,
               0 AS MatchKills, SUM(1 - NullDeathsHit) AS MatchDeaths
        FROM   #ValidKills
        WHERE  AttackerInGvG = 1 AND VictimInGvG = 1
        GROUP BY VictimGuild, VictimSchool
    ) AS GuildAgg
    GROUP BY GuildNum;

    -- STEP 4a-filter: Remove guilds that don't meet MinGuildMembers threshold.
    IF @MinGuildMembers > 0
    BEGIN
        DELETE gf
        FROM #GuildFinal gf
        WHERE (
            SELECT COUNT(*)
            FROM EventRegistration er
            WHERE er.EventID = @EventID
              AND er.GuildNum = gf.GuildNum
              AND er.JoinGvG = 1
              AND er.Status = 1
        ) < @MinGuildMembers;
    END

    -- STEP 4b: Compute per-guild tower points based on tower winners.
    ALTER TABLE #GuildFinal ADD MatchTower INT NOT NULL DEFAULT 0;

    UPDATE gf
    SET gf.MatchTower =
        CASE WHEN gf.ChaSchool = @TowerWinTH  THEN ISNULL(@TowerPtsTH,  0) ELSE 0 END
      + CASE WHEN gf.ChaSchool = @TowerWinNuc  THEN ISNULL(@TowerPtsNuc,  0) ELSE 0 END
      + CASE WHEN gf.ChaSchool = @TowerWinFaci THEN ISNULL(@TowerPtsFaci, 0) ELSE 0 END
    FROM #GuildFinal gf;

    -- STEP 5: Write per-match results
    INSERT INTO MatchCharResult (MatchID, ChaNum, ChaName, ChaClass, GuildNum, GuildName, MatchKills, MatchDeaths)
    SELECT @MatchID, cf.ChaNum, cf.ChaName, cf.ChaClass, cf.GuildNum,
           g.GuName, cf.MatchKills, cf.MatchDeaths
    FROM   #CharFinal cf
    LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = cf.GuildNum;

    INSERT INTO MatchGuildResult (MatchID, GuildNum, GuildName, ChaSchool, MatchKills, MatchDeaths, MatchTower)
    SELECT @MatchID, gf.GuildNum,
           COALESCE(g.GuName, 'Unknown'), gf.ChaSchool,
           gf.MatchKills, gf.MatchDeaths, gf.MatchTower
    FROM   #GuildFinal gf
    LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = gf.GuildNum;

    -- STEP 6a: UPSERT guild scores (includes tower points)
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
                (SELECT GuName FROM [RanGameS1].[dbo].[GuildInfo] WHERE GuNum = source.GuildNum),
                source.ChaSchool,
                source.MatchKills, source.MatchDeaths, source.MatchTower, 0);

    -- STEP 6b: UPSERT KOTH scores
    MERGE KOTHScore AS target
    USING (
        SELECT cf.ChaNum, cf.ChaName, cf.ChaClass, cf.GuildNum,
               g.GuName AS GuildName,
               r.ChaSchool,
               cf.MatchKills, cf.MatchDeaths
        FROM   #CharFinal cf
        JOIN   EventRegistration r ON r.ChaNum = cf.ChaNum AND r.EventID = @EventID
        LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = cf.GuildNum
        WHERE  r.JoinKOTH = 1
    ) AS source
    ON target.EventID = @EventID AND target.ChaNum = source.ChaNum
    WHEN MATCHED THEN
        UPDATE SET
            TotalKills  = target.TotalKills  + source.MatchKills,
            TotalDeaths = target.TotalDeaths + source.MatchDeaths
    WHEN NOT MATCHED THEN
        INSERT (EventID, ChaNum, ChaName, ChaClass, ChaSchool, GuildNum, GuildName, TotalKills, TotalDeaths, Deduction)
        VALUES (@EventID, source.ChaNum, source.ChaName, source.ChaClass,
                source.ChaSchool, source.GuildNum, source.GuildName,
                source.MatchKills, source.MatchDeaths, 0);

    -- STEP 7: Apply admin deductions
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

    -- STEP 8: Refresh school aggregation
    -- TotalTower is computed from MatchSchedule tower winners directly,
    -- NOT from SUM(GuildEventScore.TowerPoints), which would multiply
    -- tower points by the number of guilds in that school.
    MERGE SchoolEventScore AS target
    USING (
        SELECT g.EventID, g.ChaSchool,
               SUM(g.TotalKills) AS TotalKills,
               SUM(g.TotalDeaths) AS TotalDeaths,
               COALESCE(t.TotalTower, 0) AS TotalTower
        FROM   GuildEventScore g
        LEFT JOIN (
            SELECT ChaSchool, SUM(TowerPts) AS TotalTower
            FROM (
                SELECT TowerWinnerTH AS ChaSchool, ISNULL(TowerPointsTH, 0) AS TowerPts
                FROM MatchSchedule
                WHERE EventID = @EventID AND TallyStatus IN ('Completed', 'Processing')
                  AND TowerWinnerTH IS NOT NULL
                UNION ALL
                SELECT TowerWinnerNuc, ISNULL(TowerPointsNuc, 0)
                FROM MatchSchedule
                WHERE EventID = @EventID AND TallyStatus IN ('Completed', 'Processing')
                  AND TowerWinnerNuc IS NOT NULL
                UNION ALL
                SELECT TowerWinnerFaci, ISNULL(TowerPointsFaci, 0)
                FROM MatchSchedule
                WHERE EventID = @EventID AND TallyStatus IN ('Completed', 'Processing')
                  AND TowerWinnerFaci IS NOT NULL
            ) towers
            GROUP BY ChaSchool
        ) t ON t.ChaSchool = g.ChaSchool
        WHERE g.EventID = @EventID
        GROUP BY g.EventID, g.ChaSchool, t.TotalTower
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

    -- STEP 9: Return stats
    DECLARE @TotalRows INT;
    SELECT @TotalRows = COUNT(DISTINCT CONCAT(CAST(ChaNum AS VARCHAR), '_', CAST(TargetNum AS VARCHAR), '_', CAST(ActionDate AS VARCHAR)))
    FROM [RanLogS1].[dbo].[LogAction] WITH (NOLOCK)
    WHERE ActionDate >= @StartTime AND ActionDate < @EndTime
      AND baseMainMapID = @EventMapMID AND baseSubMapID = @EventMapSID AND Type = 2;

    DECLARE @ValidKillCount INT;
    SELECT @ValidKillCount = COUNT(*) FROM #ValidKills;

    SELECT @TotalRows AS TotalRows, @ValidKillCount AS ValidKills;

    DROP TABLE #Gimmicks, #ValidKills, #CharFinal, #GuildFinal;
END
GO
