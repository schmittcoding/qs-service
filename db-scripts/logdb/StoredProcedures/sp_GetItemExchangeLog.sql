USE RanLogS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetItemExchangeLog]
    @ItemFrom INT = NULL,
    @ItemTo INT = NULL,
    @ItemID UNIQUEIDENTIFIER = NULL,
    @Page INT = 1,
    @PageSize INT = 10
AS
BEGIN
    SET NOCOUNT ON;

    -- Compute pagination offsets
    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    SELECT 
        COUNT(*) AS TotalRecords
    FROM RanLogS1.dbo.ItemLog IL WITH (NOLOCK)
    WHERE
        (@ItemId IS NULL OR IL.ItemUUID = @ItemId) AND
        (@ItemTo IS NULL OR IL.ItemTo = @ItemTo) AND
        (@ItemFrom IS NULL OR IL.ItemFrom = @ItemFrom)
    
    SELECT 
        IL.ExchangeNum,
        IL.ExchangeFlag,
        IL.ExchangeFlagDetail,
        IL.ExchangeDate,
        ITL.ItemName,
        IL.NIDMain,
        IL.NIDSub,
        IL.MakeType,
        IL.ItemUUID,
        IL.ItemAmount,
        IL.ItemFromFlag,
        IL.ItemFrom,
        FCI.ChaName AS ItemFromName,
        IL.ItemToFlag,
        IL.ItemTo,
        TCI.ChaName AS ItemToName,
        IL.Damage,
        IL.Defense,
        IL.Fire,
        IL.Ice,
        IL.Poison,
        IL.Electric,
        IL.Spirit,
        IL.CostumeMID,
        IL.CostumeSID,
        IL.TradePrice
    FROM RanLogS1.dbo.ItemLog IL WITH (NOLOCK)
    LEFT JOIN RanGameS1.dbo.ChaInfo FCI ON FCI.ChaNum = IL.ItemFrom
    LEFT JOIN RanGameS1.dbo.ChaInfo TCI ON TCI.ChaNum = IL.ItemTo
    LEFT JOIN RanLogS1.dbo.ItemList ITL ON ITL.ItemMain = IL.NIDMain AND ITL.ItemSub = IL.NIDSub
    WHERE
        (@ItemId IS NULL OR IL.ItemUUID = @ItemId) AND
        (@ItemTo IS NULL OR IL.ItemTo = @ItemTo) AND
        (@ItemFrom IS NULL OR IL.ItemFrom = @ItemFrom)
    ORDER BY ExchangeNum DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
    
END

CREATE NONCLUSTERED INDEX IX_ItemLog_ItemUUID ON RanLogS1.dbo.ItemLog(ItemUUID);
CREATE NONCLUSTERED INDEX IX_ItemLog_ItemTo ON RanLogS1.dbo.ItemLog(ItemTo);
CREATE NONCLUSTERED INDEX IX_ItemLog_ItemFrom ON RanLogS1.dbo.ItemLog(ItemFrom);
CREATE NONCLUSTERED INDEX IX_ChaInfo_ChaNum ON RanGameS1.dbo.ChaInfo(ChaNum);