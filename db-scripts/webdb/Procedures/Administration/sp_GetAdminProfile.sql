USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetAdminProfile]
    @UserId UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        UserID,
        UserName,
        UserType
    FROM RanWeb.dbo.AdminInfo WITH (NOLOCK)
    WHERE UserID = @UserId;
END
GO
