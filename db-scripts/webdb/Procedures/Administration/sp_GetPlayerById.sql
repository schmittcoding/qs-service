USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetPlayerById]
    @UserNum INT = NULL
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        WU.UserID,
        WU.UserNum,
        WU.UserName,
        WU.UserType,
        UI.UserLoginState,
        WU.UserAvailable,
        UI.UserBlock,
        UI.UserBlockDate,
        UI.CreateDate,
        WU.UserEmail,
        WU.WebPoints,
        WU.UserVerified,
        UI.LastLoginDate
    FROM RanWeb.dbo.WebUser WU WITH (NOLOCK)
    INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = WU.UserNum
    WHERE WU.UserNum = @UserNum
END