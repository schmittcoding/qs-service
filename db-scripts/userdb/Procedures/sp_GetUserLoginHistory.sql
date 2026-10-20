USE RanUser
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetUserLoginHistory]
    @UserNum INT,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    -- Validate and adjust pagination parameters
    IF (@Page < 1) SET @Page = 1;
    IF (@PageSize <= 0) SET @PageSize = 10;

    -- Compute pagination offset (1-based page to 0-based offset)
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- STEP 1: Count total records for this user
    SELECT COUNT(*) AS TotalRecords
    FROM RanUser.dbo.GSLogLogin WITH (NOLOCK)
    WHERE UserNum = @UserNum;

    -- STEP 2: Return paginated result set sorted by LogDate DESC
    SELECT
        LoginNum,
        UserID,
        LogInOut,
        LogDate,
        LogIpAddress,
        HWID
    FROM RanUser.dbo.GSLogLogin WITH (NOLOCK)
    WHERE UserNum = @UserNum
    ORDER BY LogDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
