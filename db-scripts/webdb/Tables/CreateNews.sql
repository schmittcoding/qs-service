USE [RanWeb]
GO

CREATE TABLE WebNews (
    NewsID int IDENTITY(1,1) PRIMARY KEY,
    NewsCategory SMALLINT NOT NULL DEFAULT 1,
    NewsTitle nvarchar(255) NOT NULL,
    NewsSlug nvarchar(50) NOT NULL,
    NewsContent nvarchar(MAX) NOT NULL,
    NewsAuthor nvarchar(255) NOT NULL DEFAULT 'Administrator',
    NewsCreationDate datetime NOT NULL DEFAULT GETDATE(),
    NewsStatus SMALLINT NOT NULL DEFAULT 1
);

CREATE INDEX IX_NewsID ON [RanWeb].[dbo].[WebNews]([NewsID]);
CREATE INDEX IX_NewsCategory ON [RanWeb].[dbo].[WebNews]([NewsCategory]);
CREATE INDEX IX_NewsStatus ON [RanWeb].[dbo].[WebNews]([NewsStatus]);

ALTER TABLE [RanWeb].[dbo].[WebNews]
ADD CONSTRAINT UQ_NewsSlug UNIQUE ([NewsSlug]);