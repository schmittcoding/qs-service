USE RanUser
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_BanUserAccount]
    @UserNum INT,
    @Status INT,
    @BlockDate DATETIME = NULL,
    @AdminID UNIQUEIDENTIFIER,
    @Notes NVARCHAR(50) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (
        SELECT UserID 
        FROM RanWeb.dbo.AdminInfo
        WHERE UserID = @AdminID
    )
    BEGIN
        SELECT 'UNAUTHORIZED' AS MESSAGE;
        RETURN;
    END

    IF NOT EXISTS (
        SELECT UserNum FROM GSUserInfo WHERE UserNum = @UserNum
    )
    BEGIN
        SELECT 'USER_NOT_FOUND' AS MESSAGE;
        RETURN;
    END

    UPDATE GSUserInfo
    SET UserBlock = @Status,
        UserBlockDate = @BlockDate
    WHERE UserNum = @UserNum

    INSERT INTO LogGmBlock (BlockUser, BlockDate, BlockState, AdminID, Memo)
    VALUES (@UserNum, @BlockDate, @Status, @AdminID, @Notes)

    SELECT 'SUCCESS' AS Message;
END