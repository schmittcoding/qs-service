USE [RanGameS1]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetLevelRankingData]
AS
BEGIN
    SELECT
        UI.UserType,
        CI.ChaNum,
        CI.UserNum,
        CI.ChaName,
        CI.ChaClass,
        CI.ChaSchool,
        CI.ChaSex,
        CI.ChaLevel,
        CI.ChaExp
    FROM RanGameS1.dbo.ChaInfo CI
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = CI.UserNum
        AND UI.UserType = 1
    WHERE CI.ChaDeleted = 0;
END;
GO
