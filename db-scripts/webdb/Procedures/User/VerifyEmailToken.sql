USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[VerifyEmailToken]
    @UserNum BIGINT,
    @Token NVARCHAR(500)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @UserVerified BIT;
    DECLARE @StoredToken NVARCHAR(500);

    SELECT
        @UserVerified = UserVerified,
        @StoredToken = UserVerificationToken
    FROM dbo.WebUser
    WHERE UserNum = @UserNum;

    IF @StoredToken IS NULL OR @StoredToken <> @Token
    BEGIN
        SELECT 0 AS IsVerified, 'Invalid token' AS Message;
        RETURN;
    END

    IF @UserVerified = 1
    BEGIN
        SELECT 0 AS IsVerified, 'Already verified' AS Message;
        RETURN;
    END

    UPDATE dbo.WebUser
    SET UserVerified = 1,
        UserVerificationToken = NULL
    WHERE UserNum = @UserNum;

    SELECT 1 AS IsVerified, 'Email verified successfully' AS Message;
END
GO
