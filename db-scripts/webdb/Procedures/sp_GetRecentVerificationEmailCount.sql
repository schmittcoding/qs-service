USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetRecentVerificationEmailCount]
    @UserNum BIGINT,
    @MinuteWindow INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT COUNT(*) AS EmailCount
    FROM [dbo].[EmailSentLog]
    WHERE UserNum = @UserNum
        AND EmailType IN ('verification', 'resend_verification')
        AND SentAt >= DATEADD(MINUTE, -@MinuteWindow, GETDATE());
END
GO
