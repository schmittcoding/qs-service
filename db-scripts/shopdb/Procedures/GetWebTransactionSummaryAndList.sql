USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[GetWebTransactionSummaryAndList]
    @UserName NVARCHAR(20) = NULL,
    @TransactionType SMALLINT = NULL,
    @TransactionStatus SMALLINT = NULL,
    @StartDate DATETIME = NULL,
    @EndDate DATETIME = NULL,
    @Page INT = 1,
    @PageSize INT = 20
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
        (@UserName IS NULL OR UserName = @UserName) AND
        (@TransactionType IS NULL OR TransactionType = @TransactionType) AND
        (@TransactionStatus IS NULL OR TransactionStatus = @TransactionStatus) AND
        (@StartDate IS NULL OR TransactionTime >= @StartDate) AND
        (@EndDate IS NULL OR TransactionTime <= @EndDate);

    -- STEP 2: Summary grouped by TransactionType
    SELECT
        CASE
            WHEN @TransactionType IS NULL THEN NULL
            ELSE TransactionType
        END AS TransactionType,
        TransactionStatus,
        COUNT(*) AS TransactionCount,
        SUM(TransactionPrice) AS TotalAmount,
        SUM(TransactionAmount) AS TotalPoints
    FROM #FilteredTransactions
    GROUP BY
        CASE
            WHEN @TransactionType IS NULL THEN NULL
            ELSE TransactionType
        END,
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
    ORDER BY TransactionTime DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
