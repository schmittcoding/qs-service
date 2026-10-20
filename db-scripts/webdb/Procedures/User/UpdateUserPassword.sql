USE [RanUser]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[UpdateUserPassword]
    @UserNum BIGINT,
    @Pincode NVARCHAR(255),
    @OldPassword NVARCHAR(255),
    @NewPassword NVARCHAR(255)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @StoredPinCode NVARCHAR(255);
    DECLARE @StoredPassword NVARCHAR(255);

    -- Fetch user data
    SELECT
        @StoredPinCode = UserPin,
        @StoredPassword = UserPass
    FROM GSUserInfo
    WHERE UserNum = @UserNum;

    -- If the user doesn't exist, return a failure message.
    IF @StoredPinCode IS NULL
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'User not found' AS Message;
        RETURN;
    END

    IF @StoredPinCode <> @Pincode
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'Invalid pincode' AS Message;
        RETURN;
    END

    IF @StoredPassword <> @OldPassword
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'Invalid old password' AS Message;
        RETURN;
    END

    BEGIN TRANSACTION;
    BEGIN TRY
        UPDATE GSUserInfo
        SET UserPass = @NewPassword
        WHERE UserNum = @UserNum;

        UPDATE RanWeb.dbo.WebUser
        SET UpdatedAt = GETDATE()
        WHERE UserNum = @UserNum;

        COMMIT TRANSACTION;

        SELECT 1 AS IsChanged, @UserNum AS UserNum, 'Password updated successfully.' AS Message;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        SELECT 0 AS IsChanged, NULL AS UserNum, ERROR_MESSAGE() AS Message;
    END CATCH
END
