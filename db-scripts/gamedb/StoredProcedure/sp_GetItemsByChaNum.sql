USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetItemsByChaNum]
    @ChaNum INT,
    @Page INT = 1,
    @PageSize INT = 20,
    @ItemMID SMALLINT = NULL,
    @ItemSID SMALLINT = NULL,
    @InvenType TINYINT = NULL
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @Offset INT = (@Page - 1) * @PageSize;

    SELECT
        ItemUUID,
        ChaNum,
        InvenType,
        ItemMID,
        ItemSID,
        ItemMakeType,
        ItemCostumeMID,
        ItemCostumeSID,
        ItemPosX,
        ItemPosY,
        ItemRemain,
        ItemCreateDate,
        ItemCostumeExpireDate,
        ItemBasicPeriodExTime,
        ItemDisguisePeriodExTime,
        ItemBasicPeriodExCount,
        ItemDisuisePeriodExCount,
        ItemDurability,
        ItemDurabilityCount,
        PointTradeCount,
        ItemAttack,
        ItemDefense,
        ItemRandomType1,
        ItemRandomValue1,
        ItemRandomType2,
        ItemRandomValue2,
        ItemRandomType3,
        ItemRandomValue3,
        ItemRandomType4,
        ItemRandomValue4,
        ItemResistFire,
        ItemResistIce,
        ItemResistElec,
        ItemResistPoison,
        ItemResistSpirit,
        ItemPetNum,
        ItemVehicleNum,
        CostumeStatEndDate,
        ItemVersion,
        ItemOption,
        PartitionNum,
        ItemLinkSkillMID,
        ItemLinkSkillSID,
        ItemLinkSkillLevel,
        ItemLinkSkillTarget,
        ItemLinkSkillOccurRate,
        ItemAddonType1,
        ItemAddonValue1,
        ItemAddonType2,
        ItemAddonValue2,
        ItemAddonType3,
        ItemAddonValue3,
        ItemAddonType4,
        ItemAddonValue4,
        ItemAddonType5,
        ItemAddonValue5,
        ItemAddonType6,
        ItemAddonValue6,
        ItemRandomType5,
        ItemRandomValue5
    FROM RanGameS1.dbo.Item WITH (NOLOCK)
    WHERE ChaNum = @ChaNum
      AND (@ItemMID   IS NULL OR ItemMID   = @ItemMID)
      AND (@ItemSID   IS NULL OR ItemSID   = @ItemSID)
      AND (@InvenType IS NULL OR InvenType = @InvenType)
    ORDER BY ItemCreateDate DESC
    OFFSET @Offset ROWS
    FETCH NEXT @PageSize ROWS ONLY;
END
GO
