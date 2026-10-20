USE RanWeb;
GO

IF OBJECT_ID('dbo.sp_TallyLevelCapRace', 'P') IS NOT NULL
    DROP PROCEDURE dbo.sp_TallyLevelCapRace;
GO

CREATE PROCEDURE dbo.sp_TallyLevelCapRace
    @EventID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    -- Upsert level progress for all active registrants
    MERGE LevelCapRaceScore AS target
    USING (
        SELECT
            er.ChaNum,
            c.ChaName,
            er.ChaClass,
            er.ChaSchool,
            er.GuildNum,
            g.GuName AS GuildName,
            er.ChaLevel AS StartLevel,
            c.ChaLevel AS CurrentLevel,
            c.ChaLevel - er.ChaLevel AS LevelsGained,
            c.LevelUpDate
        FROM EventRegistration er
        JOIN [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = er.ChaNum
        LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
        WHERE er.EventID = @EventID AND er.Status = 1
    ) AS source
    ON target.EventID = @EventID AND target.ChaNum = source.ChaNum
    WHEN MATCHED THEN
        UPDATE SET
            CurrentLevel = source.CurrentLevel,
            LevelsGained = source.LevelsGained,
            LevelUpDate = source.LevelUpDate
    WHEN NOT MATCHED THEN
        INSERT (EventID, ChaNum, ChaName, ChaClass, ChaSchool, GuildNum, GuildName,
                StartLevel, CurrentLevel, LevelsGained, LevelUpDate)
        VALUES (@EventID, source.ChaNum, source.ChaName, source.ChaClass, source.ChaSchool,
                source.GuildNum, source.GuildName,
                source.StartLevel, source.CurrentLevel, source.LevelsGained, source.LevelUpDate);

    -- Extract target level from EventDefinition.CategoryConfig JSON, default to 99999 if missing
    DECLARE @TargetLevel INT;
    SELECT @TargetLevel = ISNULL(
        JSON_VALUE(ed.CategoryConfig, '$.target_level'),
        99999
    )
    FROM EventDefinition ed
    WHERE ed.EventID = @EventID;

    -- Build temp table of new winners: active registrants who reached target level during event window
    SELECT
        er.EventID,
        er.ChaNum,
        c.ChaName,
        er.ChaClass,
        er.ChaSchool,
        er.GuildNum,
        COALESCE(g.GuName, 'Unknown') AS GuildName,
        c.ChaLevel AS ReachedLevel,
        c.LevelUpDate,
        ROW_NUMBER() OVER (ORDER BY c.LevelUpDate ASC) + ISNULL(
            (SELECT MAX(Rank) FROM LevelCapRaceWinner WHERE EventID = @EventID),
            0
        ) AS Rank
    INTO #NewWinners
    FROM EventRegistration er
    INNER JOIN [RanGameS1].[dbo].[ChaInfo] c ON c.ChaNum = er.ChaNum
    LEFT JOIN [RanGameS1].[dbo].[GuildInfo] g ON g.GuNum = er.GuildNum
    INNER JOIN EventDefinition ed ON ed.EventID = @EventID
    WHERE er.EventID = @EventID
      AND er.Status = 1
      AND c.ChaLevel >= @TargetLevel
      AND c.LevelUpDate IS NOT NULL
      AND c.LevelUpDate BETWEEN ed.EventStart AND ed.EventEnd
      AND NOT EXISTS (
          SELECT 1 FROM LevelCapRaceWinner w
          WHERE w.EventID = @EventID AND w.ChaNum = er.ChaNum
      );

    -- Idempotent insert: only insert winners that don't already exist
    INSERT INTO LevelCapRaceWinner (EventID, ChaNum, ChaName, ChaClass, ChaSchool, GuildNum, GuildName, ReachedLevel, LevelUpDate, ReachedAt, Rank)
    SELECT EventID, ChaNum, ChaName, ChaClass, ChaSchool, GuildNum, GuildName, ReachedLevel, LevelUpDate, GETDATE(), Rank
    FROM #NewWinners
    WHERE NOT EXISTS (
        SELECT 1 FROM LevelCapRaceWinner w
        WHERE w.EventID = #NewWinners.EventID AND w.ChaNum = #NewWinners.ChaNum
    );

    -- Return participant count as rows_tallied (result-set 1, unchanged)
    DECLARE @ParticipantCount INT;
    SELECT @ParticipantCount = COUNT(*) FROM LevelCapRaceScore WHERE EventID = @EventID;
    SELECT @ParticipantCount AS TotalRows, @ParticipantCount AS ValidEntries;

    -- Return newly-crowned winners (result-set 2)
    SELECT ChaNum, ChaName, ChaClass, ChaSchool, ReachedLevel, LevelUpDate, GuildName, Rank
    FROM #NewWinners
    ORDER BY Rank ASC;

    DROP TABLE #NewWinners;
END
GO
