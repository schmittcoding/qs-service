USE [RanUser]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[ResetUserPincode]
    @UserNum BIGINT,
    @NewPinCode NVARCHAR(255)
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (
        SELECT 1 FROM RanUser.dbo.GSUserInfo WHERE UserNum = @UserNum
    )
    BEGIN
        SELECT 0 AS IsChanged, NULL AS UserNum, 'User not found' AS Message;
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

        SELECT 1 AS IsChanged, @UserNum AS UserNum, 'PIN code reset successfully.' AS Message;
    END TRY
    BEGIN CATCH
        ROLLBACK TRANSACTION;
        SELECT 0 AS IsChanged, NULL AS UserNum, ERROR_MESSAGE() AS Message;
    END CATCH
END
