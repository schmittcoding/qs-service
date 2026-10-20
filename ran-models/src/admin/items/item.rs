use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub item_uuid: Uuid,
    pub cha_num: i32,
    pub inven_type: u8,
    pub item_mid: i16,
    pub item_sid: i16,
    pub item_make_type: u8,
    pub item_costume_mid: i16,
    pub item_costume_sid: i16,
    pub item_pos_x: u8,
    pub item_pos_y: u8,
    pub item_remain: i16,
    pub item_create_date: DateTime<Local>,
    pub item_costume_expire_date: DateTime<Local>,
    pub item_basic_period_ex_time: DateTime<Local>,
    pub item_disguise_period_ex_time: DateTime<Local>,
    pub item_basic_period_ex_count: u8,
    pub item_disuise_period_ex_count: u8,
    pub item_durability: i32,
    pub item_durability_count: i32,
    pub point_trade_count: i32,
    pub item_attack: u8,
    pub item_defense: u8,
    pub item_random_type_1: u8,
    pub item_random_value_1: i32,
    pub item_random_type_2: u8,
    pub item_random_value_2: i32,
    pub item_random_type_3: u8,
    pub item_random_value_3: i32,
    pub item_random_type_4: u8,
    pub item_random_value_4: i32,
    pub item_resist_fire: u8,
    pub item_resist_ice: u8,
    pub item_resist_elec: u8,
    pub item_resist_poison: u8,
    pub item_resist_spirit: u8,
    pub item_pet_num: i32,
    pub item_vehicle_num: i32,
    pub costume_stat_end_date: DateTime<Local>,
    pub item_version: i32,
    pub item_option: i32,
    pub partition_num: u8,
    pub item_link_skill_mid: i16,
    pub item_link_skill_sid: i16,
    pub item_link_skill_level: i16,
    pub item_link_skill_target: i16,
    pub item_link_skill_occur_rate: f64,
    pub item_addon_type_1: i16,
    pub item_addon_value_1: i32,
    pub item_addon_type_2: i16,
    pub item_addon_value_2: i32,
    pub item_addon_type_3: i16,
    pub item_addon_value_3: i32,
    pub item_addon_type_4: i16,
    pub item_addon_value_4: i32,
    pub item_addon_type_5: i16,
    pub item_addon_value_5: i32,
    pub item_addon_type_6: i16,
    pub item_addon_value_6: i32,
    pub item_random_type_5: i16,
    pub item_random_value_5: i32,
}

impl Item {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            item_uuid: row
                .get::<Uuid, _>("ItemUUID")
                .unwrap_or_else(Uuid::nil),
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            inven_type: row.get::<u8, _>("InvenType").unwrap_or_default(),
            item_mid: row.get::<i16, _>("ItemMID").unwrap_or_default(),
            item_sid: row.get::<i16, _>("ItemSID").unwrap_or_default(),
            item_make_type: row.get::<u8, _>("ItemMakeType").unwrap_or_default(),
            item_costume_mid: row.get::<i16, _>("ItemCostumeMID").unwrap_or_default(),
            item_costume_sid: row.get::<i16, _>("ItemCostumeSID").unwrap_or_default(),
            item_pos_x: row.get::<u8, _>("ItemPosX").unwrap_or_default(),
            item_pos_y: row.get::<u8, _>("ItemPosY").unwrap_or_default(),
            item_remain: row.get::<i16, _>("ItemRemain").unwrap_or_default(),
            item_create_date: row
                .get::<NaiveDateTime, _>("ItemCreateDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            item_costume_expire_date: row
                .get::<NaiveDateTime, _>("ItemCostumeExpireDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            item_basic_period_ex_time: row
                .get::<NaiveDateTime, _>("ItemBasicPeriodExTime")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            item_disguise_period_ex_time: row
                .get::<NaiveDateTime, _>("ItemDisguisePeriodExTime")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            item_basic_period_ex_count: row.get::<u8, _>("ItemBasicPeriodExCount").unwrap_or_default(),
            item_disuise_period_ex_count: row.get::<u8, _>("ItemDisuisePeriodExCount").unwrap_or_default(),
            item_durability: row.get::<i32, _>("ItemDurability").unwrap_or_default(),
            item_durability_count: row.get::<i32, _>("ItemDurabilityCount").unwrap_or_default(),
            point_trade_count: row.get::<i32, _>("PointTradeCount").unwrap_or_default(),
            item_attack: row.get::<u8, _>("ItemAttack").unwrap_or_default(),
            item_defense: row.get::<u8, _>("ItemDefense").unwrap_or_default(),
            item_random_type_1: row.get::<u8, _>("ItemRandomType1").unwrap_or_default(),
            item_random_value_1: row.get::<i32, _>("ItemRandomValue1").unwrap_or_default(),
            item_random_type_2: row.get::<u8, _>("ItemRandomType2").unwrap_or_default(),
            item_random_value_2: row.get::<i32, _>("ItemRandomValue2").unwrap_or_default(),
            item_random_type_3: row.get::<u8, _>("ItemRandomType3").unwrap_or_default(),
            item_random_value_3: row.get::<i32, _>("ItemRandomValue3").unwrap_or_default(),
            item_random_type_4: row.get::<u8, _>("ItemRandomType4").unwrap_or_default(),
            item_random_value_4: row.get::<i32, _>("ItemRandomValue4").unwrap_or_default(),
            item_resist_fire: row.get::<u8, _>("ItemResistFire").unwrap_or_default(),
            item_resist_ice: row.get::<u8, _>("ItemResistIce").unwrap_or_default(),
            item_resist_elec: row.get::<u8, _>("ItemResistElec").unwrap_or_default(),
            item_resist_poison: row.get::<u8, _>("ItemResistPoison").unwrap_or_default(),
            item_resist_spirit: row.get::<u8, _>("ItemResistSpirit").unwrap_or_default(),
            item_pet_num: row.get::<i32, _>("ItemPetNum").unwrap_or_default(),
            item_vehicle_num: row.get::<i32, _>("ItemVehicleNum").unwrap_or_default(),
            costume_stat_end_date: row
                .get::<NaiveDateTime, _>("CostumeStatEndDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            item_version: row.get::<i32, _>("ItemVersion").unwrap_or_default(),
            item_option: row.get::<i32, _>("ItemOption").unwrap_or_default(),
            partition_num: row.get::<u8, _>("PartitionNum").unwrap_or_default(),
            item_link_skill_mid: row.get::<i16, _>("ItemLinkSkillMID").unwrap_or_default(),
            item_link_skill_sid: row.get::<i16, _>("ItemLinkSkillSID").unwrap_or_default(),
            item_link_skill_level: row.get::<i16, _>("ItemLinkSkillLevel").unwrap_or_default(),
            item_link_skill_target: row.get::<i16, _>("ItemLinkSkillTarget").unwrap_or_default(),
            item_link_skill_occur_rate: row.get::<f64, _>("ItemLinkSkillOccurRate").unwrap_or_default(),
            item_addon_type_1: row.get::<i16, _>("ItemAddonType1").unwrap_or_default(),
            item_addon_value_1: row.get::<i32, _>("ItemAddonValue1").unwrap_or_default(),
            item_addon_type_2: row.get::<i16, _>("ItemAddonType2").unwrap_or_default(),
            item_addon_value_2: row.get::<i32, _>("ItemAddonValue2").unwrap_or_default(),
            item_addon_type_3: row.get::<i16, _>("ItemAddonType3").unwrap_or_default(),
            item_addon_value_3: row.get::<i32, _>("ItemAddonValue3").unwrap_or_default(),
            item_addon_type_4: row.get::<i16, _>("ItemAddonType4").unwrap_or_default(),
            item_addon_value_4: row.get::<i32, _>("ItemAddonValue4").unwrap_or_default(),
            item_addon_type_5: row.get::<i16, _>("ItemAddonType5").unwrap_or_default(),
            item_addon_value_5: row.get::<i32, _>("ItemAddonValue5").unwrap_or_default(),
            item_addon_type_6: row.get::<i16, _>("ItemAddonType6").unwrap_or_default(),
            item_addon_value_6: row.get::<i32, _>("ItemAddonValue6").unwrap_or_default(),
            item_random_type_5: row.get::<i16, _>("ItemRandomType5").unwrap_or_default(),
            item_random_value_5: row.get::<i32, _>("ItemRandomValue5").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub cha_num: Option<i32>,
    pub item_main: Option<i16>,
    pub item_sub: Option<i16>,
    pub inven_type: Option<u8>,
}
