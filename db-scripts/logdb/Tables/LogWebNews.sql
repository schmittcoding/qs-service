USE [RanLogS1]
GO

CREATE TRIGGER trg_CheckNewsID ON [RanLogS1].[dbo].[WebNewsLog]
AFTER INSERT
AS
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM inserted i
        LEFT JOIN [RanWeb].[dbo].[WebNews] w ON i.NewsID = w.NewsID
        WHERE w.NewsID IS NULL
    )
    BEGIN
        RAISERROR ('Invalid NewsID: No matching record in WebNews.', 16, 1);
        ROLLBACK TRANSACTION;
    END
END;
GO

CREATE TABLE [dbo].[WebNewsLog] (
    [LogID] UNIQUEIDENTIFIER DEFAULT NEWSEQUENTIALID() PRIMARY KEY,
    [NewsID] UNIQUEIDENTIFIER NOT NULL,
    [ActionType] NVARCHAR(50) NOT NULL,
    [ChangedBy] NVARCHAR(255) NOT NULL,
    [ChangeDate] DATETIME DEFAULT GETDATE(),
    [OldNewsCategory] NVARCHAR(255),
    [OldNewsTitle] NVARCHAR(255),
    [OldNewsSlug] NVARCHAR(255),
    [OldNewsContent] NVARCHAR(MAX),
    [OldNewsAuthor] NVARCHAR(255),
    [OldNewsStatus] BIT,
    FOREIGN KEY ([NewsID]) REFERENCES [RanWeb].[dbo].[WebNews]([NewsID]) ON DELETE CASCADE
);
GO