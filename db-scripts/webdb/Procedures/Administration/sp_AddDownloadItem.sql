USE [RanWeb]
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

-- If the procedure exists, it will be altered; otherwise, it will be created.
CREATE OR ALTER PROCEDURE [dbo].[sp_AddDownloadItem]
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
        SELECT 'UNAUTHORIZED' AS Message, NULL AS WebDownloadID;
        RETURN;
    END

    IF EXISTS (
        SELECT WebDownloadID 
        FROM WebDownload 
        WHERE WebDownloadProvider = @Provider AND 
            WebDownloadVersion = @Version AND
            WebDownloadType = @Type
    )
    BEGIN
        SELECT 'DUPLICATE_DATA' AS Message, NULL AS WebDownloadID
        RETURN;
    END;


    BEGIN TRANSACTION;
        INSERT INTO dbo.WebDownload
            (WebDownloadProvider, WebDownloadLink, WebDownloadStatus, WebDownloadVersion, WebDownloadType)
        OUTPUT 'SUCCESS' AS Message, inserted.WebDownloadID
        VALUES (@Provider, @Link, @Status, @Version, @Type);
    COMMIT;    
END;
GO