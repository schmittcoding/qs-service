USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_LogEmailSent]
    @UserNum BIGINT,
    @EmailType NVARCHAR(50),
    @RecipientEmail NVARCHAR(100),
    @Status NVARCHAR(20) = 'sent',
    @ErrorMessage NVARCHAR(MAX) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    INSERT INTO [dbo].[EmailSentLog]
        ([UserNum], [EmailType], [RecipientEmail], [Status], [ErrorMessage])
    VALUES
        (@UserNum, @EmailType, @RecipientEmail, @Status, @ErrorMessage);
END
GO
