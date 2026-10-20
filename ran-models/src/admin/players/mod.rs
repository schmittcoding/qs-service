use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tiberius::{Row};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UserAccountInfo {
    /*
        UserNum
        UserID
        UserPass <-- Action
        UserType
        UserLoginState ??
        UserAvailable <-- Action
        CreateDate
        LastLoginDate
        UserBlock <-- Action
        UserBlockDate
        UserEmail <-- Action
        UserPass2 // InGame Pin <-- Action
        UserPin // WebPin <-- Action
        WebPoints
     */

    pub user_id: Uuid,
    pub user_num: i64,
    pub user_name: String,
    // pub user_pass: String,
    pub user_type: i16,
    pub user_login_state: u8,
    pub user_available: u8,
    pub user_block: i32,
    pub user_block_date: DateTime<Local>,
    pub user_create_date: DateTime<Local>,
    pub last_login_date: DateTime<Local>,
    pub user_email: String,
    // pub user_game_pin: String,
    // pub user_web_pin: String,
    pub user_web_points: f64,
    pub user_verified: u8,
}

pub struct UserAccountLoginInfo {
    /*
        LoginNum
        UserNum
        LogInOut <-- State 0 = Out, 1 = In
        LogDate
        LogIpAddress
     */
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UserCharacterList {
    pub cha_num: i32,
    pub cha_name: String,
    pub cha_class: i32,
    pub cha_school: i32,
    pub cha_sex: i16,
    pub cha_level: i32,
    pub cha_money: f64,
    pub cha_online: i32,

    pub cha_deleted: i32,
    pub cha_deleted_date: DateTime<Local>,
}

pub struct UserGuildInfo {
    /*
        GuNum
        ChaNum
        GuDeputy
        GuName
        GuNotice
        GuMoney
        GuIncomeMoney
        GuRank
        GuMakeTime

     */
}

pub struct UserCharacterActionLog {
    /*
        ActionNum
        ChaNum
        Type
        TargetNum
        TargetType
        BrightPoint
        LifePoint
        ExpPoint
        ActionMoney
        ActionDate
        baseMainMapID
        baseSubMapID
        gaeaMainMapID
        gaeaSubMapID
     */
}

pub struct UserCharacterItemLog {
    
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct BanAccountHistory {
    pub block_num: i32,
    pub block_date: DateTime<Local>,
    pub block_state: i32,
    pub block_user: i32,
    pub block_memo: String,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub created_at: DateTime<Local>
}

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
pub struct BanAccountRequestPayload {
    pub block_date: Option<String>,
    pub status: i16,
    pub notes: Option<String>
}

pub struct GetPlayersDBResult {
    pub data: Vec<Row>,
    pub total_records: i32
}

impl BanAccountHistory {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            admin_id: row
                .get::<Uuid, _>("AdminID")
                .unwrap_or_else(Uuid::nil),
            admin_name: row
                .get::<&str, _>("AdminName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            block_date: row
                .get::<NaiveDateTime, _>("BlockDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
            block_memo: row
                .get::<&str, _>("Memo")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            block_num: row.get::<i32, _>("BlockNum").unwrap_or_default(),
            block_state: row.get::<i32, _>("BlockState").unwrap_or(1),
            block_user: row.get::<i32, _>("BlockUser").unwrap_or_default(),
            created_at: row
                .get::<NaiveDateTime, _>("LogDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
        }
        
    }
}

impl UserCharacterList {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            cha_num: row.get::<i32, _>("ChaNum").unwrap_or_default(),
            cha_name: row
                .get::<&str, _>("ChaName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            cha_class: row.get::<i32, _>("ChaClass").unwrap_or(1),
            cha_sex: row.get::<u8, _>("ChaSex").unwrap() as i16,
            cha_school: row.get::<i32, _>("ChaSchool").unwrap_or(1),
            cha_level: row.get::<i32, _>("ChaLevel").unwrap_or(1),

            cha_money: row.get::<f64, _>("ChaMoney").unwrap_or(0.0),

            cha_online: row.get::<i32, _>("ChaOnline").unwrap_or(0),
            
            cha_deleted: row.get::<i32, _>("ChaDeleted").unwrap_or(0),
            cha_deleted_date: row
                .get::<NaiveDateTime, _>("ChaDeletedDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct UserLoginHistoryRecord {
    pub login_num: i64,
    pub user_id: String,
    pub login_logout: i32,
    pub log_date: String,
    pub log_ip_address: String,
    pub hwid: String,
}

impl UserAccountInfo {
    pub fn from_row(row: &Row) -> Self {
        let to_local = |naive: NaiveDateTime| -> DateTime<Local> {
            Local
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
        };

        Self {
            user_id: row
                .get::<Uuid, _>("UserID")
                .unwrap_or_else(Uuid::nil), // Default to nil UUID if conversion fails
            user_num: row.get::<i64, _>("UserNum").unwrap_or_default(),
            user_name: row
                .get::<&str, _>("UserName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),

            user_email: row.get::<&str, _>("UserEmail").map(|s| s.to_string()).unwrap_or_default(),

            user_type: row.get::<i16, _>("UserType").unwrap_or(1),

            user_available: row
                .try_get::<bool, _>("UserAvailable")
                .ok()
                .flatten()
                .unwrap_or(false) as u8,

            user_block: row.get::<i32, _>("UserBlock").unwrap_or(1),
            user_block_date: row
                .get::<NaiveDateTime, _>("UserBlockDate")
                .map(|naive: NaiveDateTime| to_local(naive))
                .unwrap_or_default(),

            user_create_date: row
                .get::<NaiveDateTime, _>("CreateDate")
                .map(|naive| to_local(naive))
                .unwrap_or_default(),

            last_login_date: row
                .get::<NaiveDateTime, _>("LastLoginDate")
                .map(|naive| to_local(naive))
                .unwrap_or_default(),

            user_login_state: row
                .try_get::<bool, _>("UserLoginState")
                .ok()
                .flatten()
                .unwrap_or(false) as u8,
            user_verified: row
                .try_get::<bool, _>("UserVerified")
                .ok()
                .flatten()
                .unwrap_or(false) as u8,
            user_web_points: row.get::<f64, _>("WebPoints").unwrap_or(0.0),
        }
    }
}

impl UserLoginHistoryRecord {
    pub fn from_row(row: &Row) -> Self {
        let log_date = row
            .get::<NaiveDateTime, _>("LogDate")
            .map(|naive: NaiveDateTime| {
                Local
                    .from_local_datetime(&naive)
                    .single()
                    .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
            })
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| String::from("1970-01-01T00:00:00+00:00"));

        Self {
            login_num: row.get::<i64, _>("LoginNum").unwrap_or_default(),
            user_id: row
                .get::<&str, _>("UserID")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            login_logout: row.get::<i32, _>("LogInOut").unwrap_or(0),
            log_date,
            log_ip_address: row
                .get::<&str, _>("LogIpAddress")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            hwid: row
                .try_get::<&str, _>("HWID")
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        }
    }
}