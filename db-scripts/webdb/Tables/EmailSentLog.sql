USE [RanWeb]
GO

CREATE TABLE [dbo].[EmailSentLog] (
    [LogID] UNIQUEIDENTIFIER NOT NULL DEFAULT NEWSEQUENTIALID(),
    [UserNum] BIGINT NOT NULL,
    [EmailType] NVARCHAR(50) NOT NULL,
    [RecipientEmail] NVARCHAR(100) NOT NULL,
    [SentAt] DATETIME NOT NULL DEFAULT GETDATE(),
    [Status] NVARCHAR(20) NOT NULL DEFAULT 'sent',
    [ErrorMessage] NVARCHAR(MAX) NULL,
    CONSTRAINT PK_EmailSentLog PRIMARY KEY ([LogID])
);
GO

CREATE NONCLUSTERED INDEX IX_EmailSentLog_UserNum ON [dbo].[EmailSentLog] ([UserNum]);
GO

CREATE NONCLUSTERED INDEX IX_EmailSentLog_EmailType_SentAt ON [dbo].[EmailSentLog] ([EmailType], [SentAt]);
GO
