USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_IsEmailSuppressed]
    @Email NVARCHAR(320)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @LowerEmail NVARCHAR(320) = LOWER(@Email);

    SELECT
        CAST(CASE WHEN EXISTS (
            SELECT 1 FROM [dbo].[EmailSuppression]
            WHERE Email = @LowerEmail AND RemovedAt IS NULL
        ) THEN 1 ELSE 0 END AS BIT) AS IsSuppressed;
END
GO
