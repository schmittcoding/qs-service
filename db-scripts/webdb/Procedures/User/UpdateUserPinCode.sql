USE [RanUser]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[UpdateUserPincode]
    @UserNum BIGINT,
    @OldPinCode NVARCHAR(255),
    @NewPinCode NVARCHAR(255)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @StoredPinCode NVARCHAR(255);

    -- Fetch user pincode
    SELECT @StoredPinCode = UserPin
    FROM RanUser.dbo.GSUserInfo
    WHERE UserNum = @UserNum;

    -- If the user doesn't exist, return a failure message.
    IF @StoredPinCode IS NULL
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'User not found' AS Message;
        RETURN;
    END

    IF @StoredPinCode <> @OldPinCode
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'Invalid pincode' AS Message;
        RETURN;
    END

    BEGIN TRANSACTION;
    BEGIN TRY
        -- Update the PIN code to the new value.
        UPDATE RanUser.dbo.GSUserInfo
        SET UserPin = @NewPinCode
        WHERE UserNum = @UserNum;

        UPDATE RanWeb.dbo.WebUser
        SET WebPin = @NewPinCode,
            UpdatedAt = GETDATE()
        WHERE UserNum = @UserNum;

        COMMIT TRANSACTION;

        SELECT 1 AS IsChanged, @UserNum AS UserNum, 'PIN code updated successfully.' AS Message;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        SELECT 0 AS IsChanged, NULL AS UserNum, ERROR_MESSAGE() AS Message;
    END CATCH
END
