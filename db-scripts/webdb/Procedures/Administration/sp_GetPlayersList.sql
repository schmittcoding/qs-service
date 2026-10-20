USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetPlayersList]
    @SearchNum BIGINT = NULL,
    @Search NVARCHAR(100) = NULL,
    @UserType NVARCHAR(MAX) = NULL,
    @UserStatus NVARCHAR(MAX) = NULL,
    @Page INT = 1,
    @PageSize INT = 20
AS
BEGIN
    SET NOCOUNT ON;

    -- Validate and adjust pagination parameters
    IF (@Page < 1) SET @Page = 1;
    IF (@PageSize <= 0) SET @PageSize = 20;

    -- Compute pagination offset (1-based page to 0-based offset)
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- Parse comma-separated strings into temp tables for filtering
    DECLARE @UserTypes TABLE (Value INT);
    DECLARE @UserStatuses TABLE (Value INT);

    IF @UserType IS NOT NULL
        INSERT INTO @UserTypes (Value)
        SELECT Value FROM dbo.fn_SplitInts(@UserType, ',');

    IF @UserStatus IS NOT NULL
        INSERT INTO @UserStatuses (Value)
        SELECT Value FROM dbo.fn_SplitInts(@UserStatus, ',');

    -- STEP 1: Count total records
    SELECT COUNT(*) AS TotalRecords
    FROM RanWeb.dbo.WebUser WU WITH (NOLOCK)
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = WU.UserNum
    WHERE
        (
            -- If SearchNum is provided, match only on it
            (@SearchNum IS NOT NULL AND WU.UserNum = @SearchNum)
            -- If SearchNum is not provided, fall back to Search
            OR
            (@SearchNum IS NULL AND (
                @Search IS NULL OR WU.UserName LIKE '%' + @Search + '%' OR WU.UserEmail = @Search
            ))
        ) AND
        (NOT EXISTS (SELECT 1 FROM @UserTypes) OR WU.UserType IN (SELECT Value FROM @UserTypes)) AND
        (NOT EXISTS (SELECT 1 FROM @UserStatuses) OR UI.UserBlock IN (SELECT Value FROM @UserStatuses));

    -- STEP 2: Return paginated result set
    SELECT
        WU.UserID,
        WU.UserNum,
        WU.UserName,
        WU.UserType,
        UI.UserLoginState,
        WU.UserAvailable,
        UI.UserBlock,
        UI.UserBlockDate,
        WU.UserEmail,
        WU.UserVerified,
        WU.WebPoints,
        UI.CreateDate,
        UI.LastLoginDate
    FROM RanWeb.dbo.WebUser WU WITH (NOLOCK)
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = WU.UserNum
    WHERE
        (
            -- If SearchNum is provided, match only on it
            (@SearchNum IS NOT NULL AND WU.UserNum = @SearchNum)
            -- If SearchNum is not provided, fall back to Search
            OR
            (@SearchNum IS NULL AND (
                @Search IS NULL OR WU.UserName LIKE '%' + @Search + '%' OR WU.UserEmail = @Search
            ))
        ) AND
        (NOT EXISTS (SELECT 1 FROM @UserTypes) OR WU.UserType IN (SELECT Value FROM @UserTypes)) AND
        (NOT EXISTS (SELECT 1 FROM @UserStatuses) OR UI.UserBlock IN (SELECT Value FROM @UserStatuses))
    ORDER BY UI.CreateDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
