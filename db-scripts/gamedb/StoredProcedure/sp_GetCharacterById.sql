USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetCharacterById]
    @ChaNum     INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        CI.ChaNum,
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
        ChaVIP,
        ChaVIPExpiryTime,

        GI.GuNum,
        GI.GuName,

        UI.UserNum,
        UI.UserID
    FROM RanGameS1.dbo.ChaInfo CI WITH (NOLOCK)
    LEFT JOIN RanGameS1.dbo.GuildInfo GI
        ON GI.GuNum = CI.GuNum
    LEFT JOIN RanUser.dbo.GSUserInfo UI
        ON UI.UserNum = CI.UserNum
    WHERE CI.ChaNum = @ChaNum
END
