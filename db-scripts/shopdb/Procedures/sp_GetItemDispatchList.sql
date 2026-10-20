USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetItemDispatchList]
    @Search NVARCHAR(100) = NULL,
    @Page INT = 0,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    ;WITH DispatchSummary AS
    (
        SELECT
            IDL.DispatchID,
            IDL.DispatchBy AS AdminID,
            AI.UserName AS AdminName,
            IDL.DispatchNotes,
            IDL.CreatedAt,
            ItemCount = COUNT(DISTINCT IL.DispatchItemID),
            UserCount = COUNT(DISTINCT CL.DispatchChaID)
        FROM dbo.ItemDispatchLog AS IDL
        LEFT JOIN dbo.ItemDispatchItemLog AS IL
            ON IL.DispatchID = IDL.DispatchID
        LEFT JOIN dbo.ItemDispatchChaLog AS CL
            ON CL.DispatchID = IDL.DispatchID
        INNER JOIN RanWeb.dbo.AdminInfo AI
            ON AI.UserID = IDL.DispatchBy
        WHERE
            @Search IS NULL
            OR IL.DispatchItemName LIKE '%' + @Search + '%'
            OR CL.DispatchChaName = @Search
        GROUP BY
            IDL.DispatchID,
            IDL.DispatchBy,
            AI.UserName,
            IDL.DispatchNotes,
            IDL.CreatedAt
    )
    SELECT
        DS.*,
        COUNT(*) OVER() AS TotalRecords
    FROM DispatchSummary AS DS
    ORDER BY DS.CreatedAt DESC
    OFFSET @Page ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
