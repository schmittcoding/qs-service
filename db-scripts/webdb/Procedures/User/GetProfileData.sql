USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[GetProfileData]
    @UserNum INT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        UserName,
        WU.UserEmail,
        WU.UserType,
        WU.UserAvailable,
        WU.UserVerified,
        WebPoints,
        MileagePoints,
        UI.CreateDate AS CreatedAt,
        UpdatedAt
    FROM dbo.WebUser WU
        INNER JOIN RanUser.dbo.GSUserInfo UI ON UI.UserNum = WU.UserNum
    WHERE WU. UserNum = @UserNum;
END
