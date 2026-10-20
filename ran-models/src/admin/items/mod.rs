use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

pub mod dispatch;
pub mod item;

pub use item::{Item, ItemListQuery};

// #[derive(Serialize, Deserialize)]
// pub struct MongoItemMetadata {
//     pub r#type: String,
//     pub version: String,
// }

// #[derive(Serialize, Deserialize)]
// pub struct MongoItem {
//     pub item_mid: i32,
//     pub item_sid: i32,
//     pub item_name: String,
// }


#[derive(Serialize, Deserialize)]
pub struct ItemInfo {
    pub item_mid: i32,
    pub item_sid: i32,
    pub item_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ItemJsonFile {
    pub version: String,
    pub items: Vec<ItemInfo>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawItemExchangeLog {
    pub exchange_num: i64,
    pub item_mid: i32,
    pub item_sid: i32,
    pub make_type: u8, // EItemMakeType
    pub item_uuid: Uuid,
    pub item_amount: i32,
    pub item_from_flag: u8, // EIdType,
    pub item_from: i32,
    pub item_from_name: String,
    pub item_to_flag: u8, // EIdType,
    pub item_to: i32,
    pub item_to_name: String,
    pub exchange_flag: u8, // EExchangeFlag,
    pub exchange_flag_detail: i32, // EExchangeFlagDetail,
    pub exchange_date: DateTime<Local>,
    pub damage: u8,
    pub defense: u8,
    pub fire: u8,
    pub ice: u8,
    pub poison: u8,
    pub electric: u8,
    pub spirit: u8,
    pub costume_mid: Option<i32>,
    pub costume_sid: Option<i32>,
    pub trade_price: f64,
}

fn to_local(naive: NaiveDateTime) -> DateTime<Local> {
    Local
        .from_local_datetime(&naive)
        .single()
        .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
}

impl RawItemExchangeLog {
    pub fn from_row(row: &Row) -> Self {
        Self {
            exchange_num: row.get::<i64, _>("ExchangeNum").unwrap_or_default(),
            exchange_flag_detail: row.get::<i32, _>("ExchangeFlagDetail").unwrap_or_default(),
            exchange_date: row
                .get::<NaiveDateTime, _>("ExchangeDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            exchange_flag: row.get::<u8, _>("ExchangeFlag").unwrap_or_default(),

            item_mid: row.get::<i32, _>("NIDMain").unwrap_or_default(),
            item_sid: row.get::<i32, _>("NIDSub").unwrap_or_default(),
            make_type: row.get::<u8, _>("MakeType").unwrap_or_default(),

            item_uuid: row
                .get::<Uuid, _>("ItemUUID")
                .unwrap_or_else(Uuid::nil),
            item_amount: row.get::<i32, _>("ItemAmount").unwrap_or_default(),
            trade_price: row.get::<f64, _>("TradePrice").unwrap_or_default(),

            item_from_flag: row.get::<u8, _>("ItemFromFlag").unwrap_or_default(),
            item_from: row.get::<i32, _>("ItemFrom").unwrap_or_default(),
            item_from_name: row
                .get::<&str, _>("ItemFromName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            item_to_flag: row.get::<u8, _>("ItemToFlag").unwrap_or_default(),
            item_to: row.get::<i32, _>("ItemTo").unwrap_or_default(),
            item_to_name: row
                .get::<&str, _>("ItemToName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            damage: row.get::<u8, _>("Damage").unwrap_or_default(),
            defense: row.get::<u8, _>("Defense").unwrap_or_default(),
            fire: row.get::<u8, _>("Fire").unwrap_or_default(),
            ice: row.get::<u8, _>("Ice").unwrap_or_default(),
            poison: row.get::<u8, _>("Poison").unwrap_or_default(),
            electric: row.get::<u8, _>("Electric").unwrap_or_default(),
            spirit: row.get::<u8, _>("Spirit").unwrap_or_default(),

            costume_mid: row.get::<i32, _>("CostumeMID"),
            costume_sid: row.get::<i32, _>("CostumeSID"),
        }
    }
}

pub struct RawItemCostumeStatsLog {
    pub costume_stats_num: i64,
    pub cha_num: i32,
    pub item_mid: i16,
    pub item_sid: i16,
    pub make_type: u8,
    pub item_uuid: Uuid,
    pub fire_date: DateTime<Local>,
    pub stat_type_1: u8,
    pub stat_value_1: f64,
    pub stat_type_2: u8,
    pub stat_value_2: f64,
    pub stat_type_3: u8,
    pub stat_value_3: f64,
    pub stat_type_4: u8,
    pub stat_value_4: f64,
    pub stat_type_5: u8,
    pub stat_value_5: f64,
    pub stat_type_6: u8,
    pub stat_value_6: f64,
    pub stat_type_7: u8,
    pub stat_value_7: f64,
    pub costume_stats_date: DateTime<Local>
}

pub struct RawItemLinkSkillLog {
    pub link_skill_num: i64,
    pub item_mid: i16,
    pub item_sid: i16,
    pub make_type: u8,
    pub item_uuid: Uuid,
    pub link_skill_mid: i16,
    pub link_skill_sid: i16,
    pub link_skill_level: i16,
    pub link_skill_target: i16,
    pub link_skill_occur_rate: f64,
    pub link_skill_date: DateTime<Local>
}

pub struct RawItemRandomLog {
    pub random_num: i64,
    pub item_mid: i16,
    pub item_sid: i16,
    pub make_type: u8,
    pub item_uuid: Uuid,
    pub random_type_1: u8,
    pub random_value_1: i16,
    pub random_stamp_1: i16,
    pub random_type_2: u8,
    pub random_value_2: i16,
    pub random_stamp_2: i16,
    pub random_type_3: u8,
    pub random_value_3: i16,
    pub random_stamp_3: i16,
    pub random_type_4: u8,
    pub random_value_4: i16,
    pub random_stamp_4: i16,
    pub random_type_5: u8,
    pub random_value_5: i16,
    pub random_stamp_5: i16,
    pub insert_option: i32
}
pub struct RawItemAddonLog {
    pub addon_num: i64,
    pub item_mid: i16,
    pub item_sid: i16,
    pub make_type: u8,
    pub item_uuid: Uuid,
    pub addon_type_1: u8,
    pub addon_value_1: i16,
    pub addon_value_f1: i16,
    pub addon_type_2: u8,
    pub addon_value_2: i16,
    pub addon_value_f2: i16,
    pub addon_type_3: u8,
    pub addon_value_3: i16,
    pub addon_value_f3: i16,
    pub addon_type_4: u8,
    pub addon_value_4: i16,
    pub addon_value_f4: i16,
    pub addon_type_5: u8,
    pub addon_value_5: i16,
    pub addon_value_f5: i16,
    pub addon_type_6: u8,
    pub addon_value_6: i16,
    pub addon_value_f6: i16,
    pub addon_date: DateTime<Local>
}

pub struct RawItemBasicStatLog {
    pub basic_stat_num: i64,
    pub item_mid: i16,
    pub item_sid: i16,
    pub make_type: u8,
    pub item_uuid: Uuid,
    pub basic_attack_damage_min: i16,
    pub basic_attack_damage_max: i16,
    pub basic_defense: i16,
    pub basic_stat_date: DateTime<Local>
}

// pub enum EAuditType {
//     Exchange = "Exchange",
//     CostumeStats = "Costume Stats",
//     LinkSkill = "Link Skill",
//     Random = "Random",
//     Addon = "Addon",
//     BasicStats = "Basic Stats"
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemChaInfo {
    pub flag: String,
    pub id: i32,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemMovementMapInfo {
    pub map_mid: i32,
    pub map_sid: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemMovementMetadata {
    pub item_mid: i32,
    pub item_sid: i32,
    pub item_amount: i32,
    pub damage: u8,
    pub defense: u8,
    pub fire: u8,
    pub ice: u8,
    pub poison: u8,
    pub electric: u8,
    pub spirit: u8,
    pub trade_price: f64,
    pub costume_mid: Option<i32>,
    pub costume_sid: Option<i32>,
    pub costume_name: Option<String>,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ItemMovementLog {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,

//     pub item_uuid: String,
//     pub item_name: String,
//     pub item_from: ItemChaInfo,
//     pub item_to: ItemChaInfo,
//     pub make_type: String,
//     pub movement_num: i64,
//     pub movement_type: String,
//     pub movement_detail: String,
//     pub movement_date: DateTime<Local>,
//     pub metadata: ItemMovementMetadata,
// }

pub use super::item_bank::catalog::ItemCatalogOption as ItemMapOptions;
