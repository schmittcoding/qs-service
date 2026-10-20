USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetSuppressedEmails]
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        Email
    FROM [dbo].[EmailSuppression]
    WHERE RemovedAt IS NULL;
END
GO
