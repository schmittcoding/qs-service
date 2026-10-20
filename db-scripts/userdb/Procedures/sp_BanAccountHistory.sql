USE RanUser
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_BanAccountHistory]
    @UserNum INT,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    IF NOT EXISTS (
        SELECT UserNum FROM GSUserInfo WHERE UserNum = @UserNum
    )
    BEGIN
        SELECT 'USER_NOT_FOUND' AS MESSAGE;
        RETURN;
    END

    -- Compute pagination offsets
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    SELECT
        BL.GmBlockNum,
        BL.AdminID, 
        A.UserName AS AdminName,
        BL.BlockDate,
        BL.BlockState,
        BL.BlockUser,
        BL.LogDate,
        BL.Memo
    INTO #FilteredHistory
    FROM dbo.LogGmBlock BL
    LEFT JOIN RanWeb.dbo.AdminInfo A
        ON A.UserID = BL.AdminID
    WHERE BL.BlockUser = @UserNum
    
    SELECT COUNT(*) AS TotalRecords FROM #FilteredHistory

    SELECT
        GmBlockNum AS BlockNum,
        AdminID, 
        AdminName,
        BlockDate,
        BlockState,
        BlockUser,
        LogDate,
        Memo
    FROM #FilteredHistory
    ORDER BY GmBlockNum DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END