-- Added 28 Oct 2026, Schmitt
-- Index UserEmail for GameRegister's 1-account-per-email check
USE [RanWeb]
GO

CREATE NONCLUSTERED INDEX IX_WebUser_UserEmail
    ON dbo.WebUser (UserEmail);
GO
