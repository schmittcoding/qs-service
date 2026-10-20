USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetTransactionCountByStatus]
    @Status SMALLINT
AS
BEGIN
    SET NOCOUNT ON;

    SELECT COUNT(*) AS TransactionCount
    FROM RanWeb.dbo.WebTransaction WITH (NOLOCK)
    WHERE TransactionStatus = @Status;
END
GO
