USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[SaveVerificationToken]
    @UserNum BIGINT,
    @Token NVARCHAR(500)
AS
BEGIN
    SET NOCOUNT ON;

    UPDATE dbo.WebUser
    SET UserVerificationToken = @Token
    WHERE UserNum = @UserNum;

    SELECT
        UserEmail,
        UserName
    FROM dbo.WebUser
    WHERE UserNum = @UserNum;
END
GO
