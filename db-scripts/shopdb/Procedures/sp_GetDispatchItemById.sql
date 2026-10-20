USE RanShop
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetDispatchItemById]
    @DispatchID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

    SELECT
        IDL.DispatchID,
        IDL.CreatedAt,
        IDL.DispatchNotes,
        AI.UserID,
        AI.UserName
    FROM dbo.ItemDispatchLog IDL
    INNER JOIN RanWeb.dbo.AdminInfo AI
        ON AI.UserID = IDL.DispatchBy
    WHERE IDL.DispatchID = @DispatchID

    SELECT
        CL.CreatedAt,
        CL.DispatchChaID,
        CL.DispatchChaName,
        CL.DispatchChaNum,
        CL.DispatchID,
        CL.DispatchUserNum,
        CI.ChaLevel AS DispatchChaLevel,
        UI.UserID AS DispatchUserID
    FROM dbo.ItemDispatchChaLog CL
    INNER JOIN RanGameS1.dbo.ChaInfo CI
        ON CI.ChaNum = CL.DispatchChaNum
    INNER JOIN RanUser.dbo.GSUserInfo UI
        ON UI.UserNum = CI.UserNum
    WHERE CL.DispatchID = @DispatchID

    SELECT
        IL.CreatedAt,
        IL.DispatchID,
        IL.DispatchItemID,
        IL.DispatchItemMain,
        IL.DispatchItemName,
        IL.DispatchItemSub,
        IL.DispatchProductNum
    FROM dbo.ItemDispatchItemLog IL
    WHERE IL.DispatchID = @DispatchID
END
GO
