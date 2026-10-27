USE [RanWeb]
GO
/****** Object:  StoredProcedure [dbo].[GameRegister]    Script Date: 3/27/2025 8:25:26 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[GameRegister]
    @UserFullName    NVARCHAR(MAX),
    @UserID          VARCHAR(24),
    @UserPass        VARCHAR(20),
    @UserPin       VARCHAR(6),
    @UserEmail       VARCHAR(100),
    @RanDatabaseName VARCHAR(50)
AS
BEGIN
    SET NOCOUNT ON;
    SET XACT_ABORT ON;

    IF @RanDatabaseName IS NULL OR LTRIM(RTRIM(@RanDatabaseName)) = ''
    BEGIN
        RAISERROR('Database name cannot be null or empty.', 16, 1);
        RETURN;
    END

    DECLARE @ExistingUser VARCHAR(20)
    SELECT @ExistingUser = UserNum
    FROM RanUser.dbo.GSUserInfo
    WHERE UserID = @UserID

    IF @ExistingUser IS NOT NULL
    BEGIN
        RAISERROR('ERROR_DUPLICATE_USERNAME', 16, 1);
        RETURN;
    END

    BEGIN TRY
        BEGIN TRANSACTION;

        -- UPDLOCK + HOLDLOCK serializes concurrent registrations for the same email
        -- so parallel requests cannot both pass the check before either inserts.
        IF EXISTS (
            SELECT 1 FROM dbo.WebUser WITH (UPDLOCK, HOLDLOCK)
            WHERE UserEmail = @UserEmail
        )
            THROW 50001, 'ERROR_DUPLICATE_EMAIL', 1;

        DECLARE @sql NVARCHAR(MAX);
        DECLARE @NewUserNum INT;

        SET @sql = N'
INSERT INTO ' + QUOTENAME(@RanDatabaseName) + N'.dbo.GSUserInfo (UserID, UserPass, UserPin, UserEmail)
VALUES (@UserID, @UserPass, @UserPin, @UserEmail);
SELECT @NewUserNum = SCOPE_IDENTITY();';

        EXEC sp_executesql
            @sql,
            N'@UserID VARCHAR(24), @UserPass VARCHAR(20), @UserPin VARCHAR(6), @UserEmail VARCHAR(100), @NewUserNum INT OUTPUT',
            @UserID = @UserID,
            @UserPass = @UserPass,
            @UserPin = @UserPin,
            @UserEmail = @UserEmail,
            @NewUserNum = @NewUserNum OUTPUT;

        INSERT INTO WebUser (UserNum, UserType, UserName, UserEmail, WebPin, UserFullName)
        VALUES (@NewUserNum, 1, @UserID, @UserEmail, @UserPin, @UserFullName);

        COMMIT TRANSACTION;

        SELECT 1 AS Result, @NewUserNum AS UserNum;
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
