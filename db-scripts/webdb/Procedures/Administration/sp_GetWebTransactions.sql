USE RanWeb
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetWebTransactions]
    @Search NVARCHAR(MAX) = NULL,
    @TransactionTypeList NVARCHAR(MAX) = NULL,
    @TransactionStatusList NVARCHAR(MAX) = NULL,
    @UserNum  INT = NULL,
    @Page INT = 1,
    @PageSize INT = 20
AS
BEGIN
    SET NOCOUNT ON;

    -- Compute pagination offsets
    DECLARE @Offset INT = @Page * @PageSize;

    DECLARE @TransactionTypes TABLE (Value INT);
    DECLARE @TransactionStatuses TABLE (Value INT);

    IF @TransactionTypeList IS NOT NULL
        INSERT INTO @TransactionTypes (Value)
        SELECT Value FROM dbo.fn_SplitInts(@TransactionTypeList, ',');

    IF @TransactionStatusList IS NOT NULL
        INSERT INTO @TransactionStatuses (Value)
        SELECT Value FROM dbo.fn_SplitInts(@TransactionStatusList, ',');

    -- STEP 1: Load filtered data into temp table
    SELECT *
    INTO #FilteredTransactions
    FROM RanWeb.dbo.WebTransaction WITH (NOLOCK)
    WHERE
        (
            (@Search IS NULL OR UserName LIKE '%' + @Search + '%') OR
            (@Search IS NULL OR RefferenceCode = @Search)
        )
        AND (NOT EXISTS (SELECT 1 FROM @TransactionTypes) OR TransactionType IN (SELECT Value FROM @TransactionTypes))
        AND (NOT EXISTS (SELECT 1 FROM @TransactionStatuses) OR TransactionStatus IN (SELECT Value FROM @TransactionStatuses))
        AND (@UserNum IS NULL OR UserNum = @UserNum);

    SELECT COUNT(*) AS TotalRecords FROM #FilteredTransactions;

    -- STEP 2: Paginated result set — pending (status = 0) always first
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
        ProofImage,
        AdminRemarks,
        UpdatedAt
    FROM #FilteredTransactions
    ORDER BY
        CASE WHEN TransactionStatus = 0 THEN 0 ELSE 1 END ASC,
        TransactionTime DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
