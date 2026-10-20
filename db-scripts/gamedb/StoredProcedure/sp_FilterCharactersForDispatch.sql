USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_FilterCharactersForDispatch]
    @SearchNum INT = NULL,
    @Search NVARCHAR(100) = NULL,
    @MinLevel INT = NULL,
    @MaxLevel INT = NULL,
    @LoginFrom DATETIME = NULL,
    @LoginTo DATETIME = NULL,
    @Page INT = 0,
    @PageSize INT = 50
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = @Page * @PageSize;
    DECLARE @HasLoginFilter BIT = CASE WHEN @LoginFrom IS NOT NULL OR @LoginTo IS NOT NULL THEN 1 ELSE 0 END;

    CREATE TABLE #LoginUsers (UserNum INT NOT NULL);

    IF @HasLoginFilter = 1
    BEGIN
        INSERT INTO #LoginUsers (UserNum)
        SELECT DISTINCT L.UserNum
        FROM RanUser.dbo.GSLogLogin L WITH (NOLOCK)
        WHERE (@LoginFrom IS NULL OR L.LogDate >= @LoginFrom)
          AND (@LoginTo IS NULL OR L.LogDate <= @LoginTo);

        CREATE CLUSTERED INDEX IX_LoginUsers ON #LoginUsers (UserNum);
    END

    SELECT
        CI.UserNum,
        UI.UserID,
        CI.ChaName,
        CI.ChaNum,
        CI.ChaLevel,
        CI.ChaClass,
        CI.ChaSchool
    INTO #FilteredChars
    FROM RanGameS1.dbo.ChaInfo CI WITH (NOLOCK)
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = CI.UserNum
        AND UI.UserAvailable = 1
        AND UI.UserBlock = 0
    WHERE CI.ChaDeleted = 0
      AND (
          (@SearchNum IS NOT NULL AND (CI.UserNum = @SearchNum OR CI.ChaNum = @SearchNum))
          OR
          (@SearchNum IS NULL AND (
              @Search IS NULL OR CI.ChaName LIKE '%' + @Search + '%' OR UI.UserID LIKE '%' + @Search + '%'
          ))
      )
      AND (@MinLevel IS NULL OR CI.ChaLevel >= @MinLevel)
      AND (@MaxLevel IS NULL OR CI.ChaLevel <= @MaxLevel)
      AND (
          @HasLoginFilter = 0
          OR EXISTS (SELECT 1 FROM #LoginUsers LU WHERE LU.UserNum = CI.UserNum)
      );

    SELECT COUNT(*) AS TotalRecords FROM #FilteredChars;

    SELECT *
    FROM #FilteredChars
    ORDER BY ChaNum DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;

    DROP TABLE #LoginUsers;
    DROP TABLE #FilteredChars;
END
GO
