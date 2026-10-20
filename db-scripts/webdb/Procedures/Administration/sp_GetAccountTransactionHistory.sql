USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetAccountTransactionHistory]
    @UserNum INT = NULL,
    @StartDate DATETIME = NULL,
    @EndDate DATETIME = NULL,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    -- Compute pagination offsets
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- STEP 1: Load filtered data into temp table
    SELECT *
    INTO #FilteredTransactions
    FROM RanWeb.dbo.WebTransaction WITH (NOLOCK)
    WHERE
        (@UserNum IS NULL OR UserNum = @UserNum) AND
        (@StartDate IS NULL OR TransactionTime >= @StartDate) AND
        (@EndDate IS NULL OR TransactionTime <= @EndDate);

    -- STEP 2: Summary grouped by TransactionType
    SELECT
        TransactionType,
        TransactionStatus,
        COUNT(*) AS TransactionCount,
        SUM(TransactionPrice) AS TotalAmount,
        SUM(TransactionAmount) AS TotalPoints
    FROM #FilteredTransactions
    GROUP BY 
        TransactionType,
        TransactionStatus;

    SELECT COUNT(*) AS TotalRecords FROM #FilteredTransactions;

    -- STEP 3: Paginated result set
    SELECT
        TransactionID,
        UserNum,
        UserName,
        TransactionType,
        TransactionTime,
        TransactionAmount,
        TransactionStatus,
        TransactionPrice,
        TransactionCost,
        RefferenceCode,
        AdminRemarks,
        UpdatedAt
    FROM #FilteredTransactions
    ORDER BY
        CASE WHEN TransactionStatus = 0 THEN 0 ELSE 1 END,
        TransactionTime ASC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
