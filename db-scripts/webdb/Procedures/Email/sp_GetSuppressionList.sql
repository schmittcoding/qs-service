USE RanWeb;
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetSuppressionList]
    @Page INT = 1,
    @PageSize INT = 20,
    @Reason NVARCHAR(20) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    -- Result set 1: total count
    SELECT COUNT(*) AS TotalRecords
    FROM [dbo].[EmailSuppression]
    WHERE (@Reason IS NULL OR Reason = @Reason);

    -- Result set 2: paginated data
    SELECT
        SuppressionID,
        Email,
        Reason,
        DiagnosticInfo,
        Source,
        SuppressedAt,
        RemovedAt,
        RemovedBy
    FROM [dbo].[EmailSuppression]
    WHERE (@Reason IS NULL OR Reason = @Reason)
    ORDER BY SuppressedAt DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
