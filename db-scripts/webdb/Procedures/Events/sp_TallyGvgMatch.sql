USE RanWeb;
GO

IF OBJECT_ID('dbo.sp_TallyGvgMatch', 'P') IS NOT NULL
    DROP PROCEDURE dbo.sp_TallyGvgMatch;
GO

CREATE PROCEDURE dbo.sp_TallyGvgMatch
    @EventID   UNIQUEIDENTIFIER,
    @MatchID   UNIQUEIDENTIFIER,
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

    SELECT @TowerPtsTH  = TowerPointsTH,
           @TowerPtsNuc  = TowerPointsNuc,
           @TowerPtsFaci = TowerPointsFaci,
           @TowerWinTH   = TowerWinnerTH,
           @TowerWinNuc  = TowerWinnerNuc,
           @TowerWinFaci = TowerWinnerFaci
    FROM MatchSchedule WHERE MatchID = @MatchID;

    -- STEP 2: Extract valid kills from LogAction (both attacker AND victim registered)
    -- De-duplicate: same ChaNum + TargetNum + ActionDate can appear multiple times due to
    -- a game-side logging bug. We pick MIN(ActionNum) per group to count each kill once.
    SELECT
        L.ChaNum          AS AttackerChaNum,
        L.TargetNum       AS VictimChaNum,
        RA.GuildNum        AS AttackerGuild,
        RA.ChaSchool       AS AttackerSchool,
        RA.ChaClass        AS AttackerClass,
        RV.GuildNum        AS VictimGuild,
        RV.ChaSchool       AS VictimSchool
    INTO #ValidKills
    FROM (
        SELECT MIN(ActionNum) AS ActionNum, ChaNum, TargetNum, ActionDate, baseMainMapID, baseSubMapID, Type
        FROM [RanLogS1].[dbo].[LogAction] WITH (NOLOCK)
        WHERE ActionDate >= @StartTime
          AND ActionDate <  @EndTime
          AND baseMainMapID = @EventMapMID
          AND baseSubMapID  = @EventMapSID
          AND Type = 2
        GROUP BY ChaNum, TargetNum, ActionDate, baseMainMapID, baseSubMapID, Type
    ) L
    INNER JOIN EventRegistration RA
        ON  RA.EventID = @EventID
        AND RA.ChaNum  = L.ChaNum
        AND RA.Status  = 1
    INNER JOIN EventRegistration RV
        ON  RV.EventID = @EventID
        AND RV.ChaNum  = L.TargetNum
        AND RV.Status  = 1;

    -- STEP 3: Aggregate per-character
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
               COUNT(*) AS MatchKills, 0 AS MatchDeaths
        FROM   #ValidKills
        GROUP BY AttackerChaNum, AttackerClass, AttackerGuild
        UNION ALL
        SELECT VictimChaNum AS ChaNum, CAST(NULL AS NVARCHAR(64)) AS ChaName,
               CAST(NULL AS INT) AS ChaClass, VictimGuild AS GuildNum,
               0 AS MatchKills, COUNT(*) AS MatchDeaths
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

    -- STEP 4: Aggregate per-guild
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
        GROUP BY AttackerGuild, AttackerSchool
        UNION ALL
        SELECT VictimGuild AS GuildNum, VictimSchool AS ChaSchool,
               0 AS MatchKills, COUNT(*) AS MatchDeaths
        FROM   #ValidKills
        GROUP BY VictimGuild, VictimSchool
    ) AS GuildAgg
    GROUP BY GuildNum;

    -- STEP 4a-filter: Remove guilds that don't meet MinGuildMembers threshold
    IF @MinGuildMembers > 0
    BEGIN
        DELETE gf
        FROM #GuildFinal gf
        WHERE (
            SELECT COUNT(*)
            FROM EventRegistration er
            WHERE er.EventID = @EventID
              AND er.GuildNum = gf.GuildNum
              AND er.Status = 1
        ) < @MinGuildMembers;
    END

    -- STEP 4b: Compute per-guild tower points based on tower winners
    ALTER TABLE #GuildFinal ADD MatchTower INT NOT NULL DEFAULT 0;

    UPDATE gf
    SET gf.MatchTower =
        CASE WHEN gf.ChaSchool = @TowerWinTH  THEN ISNULL(@TowerPtsTH, 0)  ELSE 0 END
      + CASE WHEN gf.ChaSchool = @TowerWinNuc  THEN ISNULL(@TowerPtsNuc, 0)  ELSE 0 END
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

    -- STEP 6a: UPSERT guild scores (now includes tower points)
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

    -- STEP 7: Apply admin deductions (guild + school)
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

    -- STEP 8: Refresh school aggregation
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
    SELECT @TotalRows = COUNT(*)
    FROM (
        SELECT MIN(ActionNum) AS ActionNum
        FROM [RanLogS1].[dbo].[LogAction] WITH (NOLOCK)
        WHERE ActionDate >= @StartTime AND ActionDate < @EndTime
          AND baseMainMapID = @EventMapMID AND baseSubMapID = @EventMapSID AND Type = 2
        GROUP BY ChaNum, TargetNum, ActionDate, baseMainMapID, baseSubMapID, Type
    ) deduped;

    DECLARE @ValidKillCount INT;
    SELECT @ValidKillCount = COUNT(*) FROM #ValidKills;

    SELECT @TotalRows AS TotalRows, @ValidKillCount AS ValidKills;

    DROP TABLE #ValidKills, #CharFinal, #GuildFinal;
END
GO
