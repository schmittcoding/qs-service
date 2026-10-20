USE [RanUser]
GO
/****** Object:  StoredProcedure [dbo].[UpdateUserEmail]    Script Date: 4/2/2025 8:28:58 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[UpdateUserEmail]
    @UserNum BIGINT,
    @Pincode NVARCHAR(255),
    @NewEmail NVARCHAR(255)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @StoredPinCode NVARCHAR(255);

    -- Fetch user pincode
    SELECT @StoredPinCode = UserPin
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

    -- Check if the new email is already in use by another user.
    IF EXISTS (SELECT 1 FROM GSUserInfo WHERE UserEmail = @NewEmail AND UserNum <> @UserNum)
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'Email is already in use' AS Message;
        RETURN;
    END

    BEGIN TRANSACTION;
    BEGIN TRY
        -- Update email in the game database.
        UPDATE GSUserInfo
        SET UserEmail = @NewEmail
        WHERE UserNum = @UserNum;

        -- Update email, verification status, and timestamp in the web database.
        UPDATE RanWeb.dbo.WebUser
        SET UserEmail = @NewEmail,
            UserVerified = 0,
            UserVerificationToken = NULL,
            UpdatedAt = GETDATE()
        WHERE UserNum = @UserNum;

        COMMIT TRANSACTION;

        SELECT 1 AS IsChanged, @UserNum AS UserNum, 'Email updated successfully.' AS Message;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        SELECT 0 AS IsChanged, NULL AS UserNum, ERROR_MESSAGE() AS Message;
    END CATCH
END
