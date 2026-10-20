USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetCharactersByUser]
    @UserNum INT,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    -- Compute pagination offsets
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- STEP 1: Load filtered data into temp table
    SELECT
        CI.ChaNum,
        ChaName,
        ChaClass,
        ChaSchool,
        ChaSex,
        ChaLevel,
        ChaMoney,
        ChaOnline,
        ChaCreateDate,
        ChaDeleted,
        ChaDeletedDate,
        ContributionPoint,
        GI.GuNum,
        GI.GuName,
        LeaderCI.ChaName AS GuildLeaderChaName,
        LeaderCI.ChaSchool AS GuildLeaderChaSchool
    INTO #FilteredCharacters
    FROM RanGameS1.dbo.ChaInfo CI WITH (NOLOCK)
    LEFT JOIN RanGameS1.dbo.GuildInfo GI
        ON GI.GuNum = CI.GuNum
    LEFT JOIN RanGameS1.dbo.ChaInfo LeaderCI WITH (NOLOCK)
        ON LeaderCI.ChaNum = GI.ChaNum
    WHERE UserNum = @UserNum

    SELECT COUNT(*) AS TotalRecords FROM #FilteredCharacters;

    -- STEP 3: Paginated result set
    SELECT *
    FROM #FilteredCharacters
    ORDER BY ChaCreateDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
