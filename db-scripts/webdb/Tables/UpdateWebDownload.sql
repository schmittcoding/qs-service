USE [RanWeb]
GO

ALTER TABLE RanWeb.dbo.WebDownload
    ADD WebDownloadVersion NVARCHAR(15)

ALTER TABLE RanWeb.dbo.WebDownload
    ADD WebDownloadType SMALLINT DEFAULT(1)
