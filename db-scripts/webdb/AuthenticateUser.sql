USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

IF OBJECT_ID('[dbo].[AuthenticateUser]', 'P') IS NOT NULL
    DROP PROCEDURE [dbo].[AuthenticateUser]
GO

CREATE PROCEDURE [dbo].[AuthenticateUser]
    @Username NVARCHAR(255),
    @Password NVARCHAR(255)
AS
BEGIN
    DECLARE @StoredPassword NVARCHAR(255);
    DECLARE @StoredUserNum BIGINT;

    -- Fetch user data
    SELECT 
        @StoredPassword = UserPass, 
        @StoredUserNum = UserNum
    FROM RanUser.dbo.GSUserInfo
    WHERE UserID = @Username;

    -- Check if the username exists
    IF @StoredPassword IS NULL
    BEGIN
        SELECT 0 AS IsAuthenticated, NULL AS UserNum, 'User not found' AS Message;
        RETURN;
    END

    -- Validate password
    IF @Password = @StoredPassword
    BEGIN
        SELECT 1 AS IsAuthenticated, @StoredUserNum AS UserNum, 'Login successful' AS Message;
    END
    ELSE
        SELECT 0 AS IsAuthenticated, NULL AS UserNum, 'Invalid password' AS Message;
END
GO
