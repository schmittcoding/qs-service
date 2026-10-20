USE [RanUser]
GO
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[GetProfileWithEmail]
    @UserEmail VARCHAR(50)  -- Input parameter for Email
AS
BEGIN
    DECLARE @UserNum INT;
    DECLARE @UserName VARCHAR(20);
    DECLARE @UserType INT;
    DECLARE @UserAvailable INT;

    -- Fetch profile data based on email
    SELECT 
        @UserNum = UserNum,
        @UserName = UserID,
        @UserEmail = UserEmail,
        @UserAvailable = UserAvailable
    FROM dbo.GSUserInfo
    WHERE UserEmail = @UserEmail;

    -- Check if user exists
    IF @UserName IS NULL
    BEGIN
        -- User not found: return all NULLs (and optionally UserNum as NULL)
        SELECT 
            NULL AS UserNum,
            NULL AS UserName, 
            NULL AS UserEmail, 
            NULL AS UserAvailable
        RETURN;
    END

    -- Return profile data if found
    SELECT 
        @UserNum AS UserNum,
        @UserName AS UserName,
        @UserEmail AS UserEmail,
        @UserAvailable AS UserAvailable
END
GO
