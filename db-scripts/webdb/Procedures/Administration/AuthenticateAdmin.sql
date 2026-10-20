USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE OR ALTER   PROCEDURE [dbo].[AuthenticateAdmin]
    @UserName NVARCHAR(20),
    @UserPass NVARCHAR(20)
AS
BEGIN
    DECLARE @PasswordHash VARBINARY(256);
    SET @PasswordHash = HASHBYTES('SHA2_256', @UserPass);

    -- Fetch admin data
    SELECT
        UserID,
        UserType
    FROM RanWeb.dbo.AdminInfo
    WHERE 
        UserName = @UserName AND
        UserAvailable = 1 AND
        UserPass = @PasswordHash;
END
GO
