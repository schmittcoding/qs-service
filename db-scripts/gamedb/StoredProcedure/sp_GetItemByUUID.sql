USE RanGameS1
GO

SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE OR ALTER PROCEDURE [dbo].[sp_GetItemByUUID]
    @ItemUUID UNIQUEIDENTIFIER
AS
BEGIN
    SET NOCOUNT ON;

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
    WHERE ItemUUID = @ItemUUID;
END
GO
