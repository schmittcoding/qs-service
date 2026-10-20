CREATE PROCEDURE AddAdministrator
    @UserName NVARCHAR(20),
    @UserPass NVARCHAR(255),
    @HWID NVARCHAR(255) = NULL
AS
BEGIN
    SET NOCOUNT ON;
    SET XACT_ABORT ON;

    -- Start a transaction
    BEGIN TRANSACTION;

    BEGIN TRY
        -- Check if the username already exists
        IF EXISTS (SELECT 1 FROM [dbo].[AdminInfo] WHERE UserName = @UserName)
        BEGIN
            RAISERROR ('Username already exists.', 16, 1);
            ROLLBACK TRANSACTION;
            RETURN;
        END

        -- Hash the password securely
        DECLARE @HashedPass VARBINARY(64);
        SET @HashedPass = HASHBYTES('SHA2_256', @UserPass);

        -- Insert the new admin record
        INSERT INTO [dbo].[AdminInfo] ([UserName], [UserPass], [HWID])
        VALUES (@UserName, @HashedPass, @HWID);

        -- Commit the transaction if successful
        COMMIT TRANSACTION;
    END TRY
    BEGIN CATCH
        -- Rollback transaction in case of errors
        ROLLBACK TRANSACTION;
        DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE();
        RAISERROR (@ErrorMessage, 16, 1);
    END CATCH
END;
