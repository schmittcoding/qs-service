USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_AddEmailSuppression]
    @Email NVARCHAR(320),
    @Reason NVARCHAR(20),
    @DiagnosticInfo NVARCHAR(500) = NULL,
    @Source NVARCHAR(50) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @LowerEmail NVARCHAR(320) = LOWER(@Email);

    -- Check if email already has an active suppression
    IF EXISTS (SELECT 1 FROM [dbo].[EmailSuppression] WHERE Email = @LowerEmail AND RemovedAt IS NULL)
    BEGIN
        RETURN;
    END

    -- Insert new suppression
    INSERT INTO [dbo].[EmailSuppression] (Email, Reason, DiagnosticInfo, Source)
    VALUES (@LowerEmail, @Reason, @DiagnosticInfo, @Source);
END
GO
