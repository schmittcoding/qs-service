USE [RanWeb]
GO

CREATE TABLE [dbo].[EmailSuppression] (
    [SuppressionID]    UNIQUEIDENTIFIER NOT NULL DEFAULT NEWSEQUENTIALID(),
    [Email]            NVARCHAR(320)    NOT NULL,
    [Reason]           NVARCHAR(20)     NOT NULL,
    [DiagnosticInfo]   NVARCHAR(500)    NULL,
    [Source]           NVARCHAR(50)     NULL,
    [SuppressedAt]     DATETIME         NOT NULL DEFAULT GETDATE(),
    [RemovedAt]        DATETIME         NULL,
    [RemovedBy]        NVARCHAR(100)    NULL,
    CONSTRAINT PK_EmailSuppression PRIMARY KEY ([SuppressionID])
);
GO

CREATE UNIQUE NONCLUSTERED INDEX IX_EmailSuppression_Email
    ON [dbo].[EmailSuppression] ([Email])
    WHERE [RemovedAt] IS NULL;
GO

CREATE NONCLUSTERED INDEX IX_EmailSuppression_Reason_SuppressedAt
    ON [dbo].[EmailSuppression] ([Reason], [SuppressedAt]);
GO
