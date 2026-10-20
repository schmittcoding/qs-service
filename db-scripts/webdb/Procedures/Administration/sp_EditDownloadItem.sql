USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

-- If the procedure exists, it will be altered; otherwise, it will be created.
CREATE OR ALTER PROCEDURE [dbo].[sp_EditDownloadItem]
    @Id UNIQUEIDENTIFIER,
    @Provider NVARCHAR(50),
    @Link NVARCHAR(MAX),
    @Version NVARCHAR(15),
    @Status BIT,
    @Type SMALLINT,
    @AdminID UNIQUEIDENTIFIER
AS
BEGIN
    IF NOT EXISTS (
        SELECT UserID 
        FROM AdminInfo
        WHERE UserID = @AdminID
    )
    BEGIN
        SELECT 'UNAUTHORIZED' AS Message;
        RETURN;
    END

    IF NOT EXISTS (
        SELECT WebDownloadID 
        FROM WebDownload 
        WHERE WebDownloadID = @Id
    )
    BEGIN
        SELECT 'NOT_FOUND' AS Message;
        RETURN;
    END;

    IF EXISTS (
        SELECT WebDownloadID 
        FROM WebDownload 
        WHERE WebDownloadProvider = @Provider AND 
            WebDownloadVersion = @Version AND
            WebDownloadType = @Type AND
            WebDownloadID <> @Id
    )
    BEGIN
        SELECT 'DUPLICATE_DATA' AS Message;
        RETURN;
    END;

    BEGIN TRANSACTION;
        UPDATE dbo.WebDownload
        SET WebDownloadStatus = @Status, 
            WebDownloadLink = @Link, 
            WebDownloadUpdateDate = GETDATE(), 
            WebDownloadProvider = @Provider,
            WebDownloadVersion = @Version,
            WebDownloadType = @Type
        WHERE WebDownloadID = @Id

        SELECT 'SUCCESS' AS Message;
    COMMIT;    
END;
GO