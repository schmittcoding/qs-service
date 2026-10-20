USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetTotalRechargeAmount]
    @Period NVARCHAR(10)
AS
BEGIN
    SET NOCOUNT ON;

    SELECT CAST(ISNULL(SUM(TransactionAmount), 0) AS DECIMAL(18, 2)) AS TotalAmount
    FROM RanWeb.dbo.WebTransaction WITH (NOLOCK)
    WHERE TransactionStatus = 1
      AND (
        (@Period = 'today' AND CAST(TransactionTime AS DATE) = CAST(GETDATE() AS DATE))
        OR (@Period = 'month' AND YEAR(TransactionTime) = YEAR(GETDATE())
                              AND MONTH(TransactionTime) = MONTH(GETDATE()))
        OR (@Period = 'year' AND YEAR(TransactionTime) = YEAR(GETDATE()))
      );
END
GO
