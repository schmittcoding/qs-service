USE RanWeb;
GO

IF OBJECT_ID('dbo.sp_TallyKothMatch', 'P') IS NOT NULL
    DROP PROCEDURE dbo.sp_TallyKothMatch;
GO

CREATE PROCEDURE dbo.sp_TallyKothMatch
    @EventID   UNIQUEIDENTIFIER,
    @MatchID   UNIQUEIDENTIFIER,
    @StartTime DATETIME,
    @EndTime   DATETIME
AS
BEGIN
    SET NOCOUNT ON;

    -- STEP 0: Load map config from EventDefinition
    DECLARE @EventMapMID INT, @EventMapSID INT;
    SELECT @EventMapMID = ISNULL(EventMapMID, 222),
           @EventMapSID = ISNULL(EventMapSID, 0)
    FROM EventDefinition WHERE EventID = @EventID;

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

    -- STEP 5: Write per-match character results
    INSERT INTO MatchCharResult (MatchID, ChaNum, ChaName, ChaClass, GuildNum, GuildName, MatchKills, MatchDeaths)
    SELECT @MatchID, cf.ChaNum, cf.ChaName, cf.ChaClass, cf.GuildNum,
           g.GuName, cf.MatchKills, cf.MatchDeaths
    FROM   #CharFinal cf
    LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = cf.GuildNum;

    -- STEP 6b: UPSERT KOTH scores (includes ChaSchool from registration)
    MERGE KOTHScore AS target
    USING (
        SELECT cf.ChaNum, cf.ChaName, cf.ChaClass, cf.GuildNum,
               g.GuName AS GuildName,
               r.ChaSchool,
               cf.MatchKills, cf.MatchDeaths
        FROM   #CharFinal cf
        JOIN   EventRegistration r ON r.ChaNum = cf.ChaNum AND r.EventID = @EventID
        LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = cf.GuildNum
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

    -- STEP 7: Apply admin deductions (KOTH player deductions)
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

    DROP TABLE #ValidKills, #CharFinal;
END
GO
