USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER   PROCEDURE [dbo].[AdminRegister]
    @UserName        NVARCHAR(24),
    @UserPass        NVARCHAR(20),
    @UserType        INT
AS
BEGIN
    SET NOCOUNT ON;
    
    DECLARE @ExistingUser VARCHAR(20)
    SELECT @ExistingUser = UserID
    FROM [RanWeb].dbo.AdminInfo
    WHERE UserName = @UserName;

    IF @ExistingUser IS NOT NULL
    BEGIN
        RAISERROR('Username is already taken', 16, 1);
        RETURN;
    END

    BEGIN TRY
        BEGIN TRANSACTION;

        DECLARE @PasswordHash VARBINARY(256);
        SET @PasswordHash = HASHBYTES('SHA2_256', @UserPass);

        INSERT INTO AdminInfo (UserType, UserName, UserPass)
        VALUES (@UserType, @UserName, @PasswordHash);
    
        COMMIT TRANSACTION;
        
        SELECT 1 AS Result;
    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0
            ROLLBACK TRANSACTION;

        IF ERROR_NUMBER() = 2627
            SELECT 0 AS Result;
        ELSE
            THROW;
    END CATCH
END
GO
