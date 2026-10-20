USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_RemoveEmailSuppression]
    @Email NVARCHAR(320),
    @RemovedBy NVARCHAR(100)
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @LowerEmail NVARCHAR(320) = LOWER(@Email);

    UPDATE [dbo].[EmailSuppression]
    SET RemovedAt = GETDATE(), RemovedBy = @RemovedBy
    WHERE Email = @LowerEmail AND RemovedAt IS NULL;
END
GO
