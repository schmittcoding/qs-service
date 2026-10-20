USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetCharacters]
    @SearchNum INT = NULL,
    @Search NVARCHAR(50) = NULL,
    @Class NVARCHAR(MAX) = NULL,
    @School NVARCHAR(MAX) = NULL,
    @Status INT = NULL,
    @ChaOnline NVARCHAR(MAX) = NULL,
    @SortBy NVARCHAR(50) = 'contribution_point',
    @SortOrder NVARCHAR(4) = 'desc',
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @SchoolTable TABLE (SchoolID INT)
    DECLARE @ClassTable TABLE (ClassID INT)
    DECLARE @ChaOnlineTable TABLE (ChaOnlineID INT)

    IF @School IS NOT NULL
    BEGIN
        INSERT INTO @SchoolTable (SchoolID)
        SELECT Value FROM dbo.fn_SplitInts(@School, ',');
    END

    IF @Class IS NOT NULL
    BEGIN
        INSERT INTO @ClassTable (ClassID)
        SELECT Value FROM dbo.fn_SplitInts(@Class, ',');
    END

    IF @ChaOnline IS NOT NULL
    BEGIN
        INSERT INTO @ChaOnlineTable (ChaOnlineID)
        SELECT Value FROM dbo.fn_SplitInts(@ChaOnline, ',');
    END

    -- Compute pagination offsets
    DECLARE @Offset INT = @Page * @PageSize;

    -- STEP 1: Load filtered data into temp table
    SELECT
        ChaNum,
        CI.ChaName,
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

        CI.UserNum,
        UI.UserID
    INTO #FilteredCharacters
    FROM RanGameS1.dbo.ChaInfo CI WITH (NOLOCK)
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = CI.UserNum
    WHERE
        (
            -- If SearchNum is provided, match only on it
            (@SearchNum IS NOT NULL AND (CI.UserNum = @SearchNum OR CI.ChaNum = @SearchNum))
            -- If SearchNum is not provided, fall back to Search
            OR
            (@SearchNum IS NULL AND (
                @Search IS NULL OR CI.ChaName LIKE '%' + @Search + '%' OR UI.UserID LIKE '%' + @Search + '%'
            ))
        ) AND
        (NOT EXISTS (SELECT 1 FROM @ClassTable) OR CI.ChaClass IN (SELECT ClassID FROM @ClassTable)) AND
        (NOT EXISTS (SELECT 1 FROM @SchoolTable) OR CI.ChaSchool IN (SELECT SchoolID FROM @SchoolTable)) AND
        (@Status IS NULL OR CI.ChaDeleted = @Status) AND
        (NOT EXISTS (SELECT 1 FROM @ChaOnlineTable) OR CI.ChaOnline IN (SELECT ChaOnlineID FROM @ChaOnlineTable));

    SELECT COUNT(*) AS TotalRecords FROM #FilteredCharacters;

    SELECT *
    FROM #FilteredCharacters
    ORDER BY
        CASE WHEN @SortOrder = 'asc' THEN
            CASE @SortBy
                WHEN 'contribution_point' THEN ContributionPoint
                WHEN 'level'              THEN ChaLevel
                WHEN 'money'              THEN ChaMoney
            END
        END ASC,

        CASE WHEN @SortOrder = 'desc' THEN
            CASE @SortBy
                WHEN 'contribution_point' THEN ContributionPoint
                WHEN 'level'              THEN ChaLevel
                WHEN 'money'              THEN ChaMoney
            END
        END DESC,
        ChaNum DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
