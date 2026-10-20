use ran_models::events::category::EventCategory;
use ran_models::events::category_config::CategoryConfig;
use ran_models::events::event::EventDefinition;
use uuid::Uuid;

use crate::gamemanager::GameDB;
use crate::gamemanager::admin::characters::GameCharacterTrait;
use crate::webmanager::WebDB;

/// Dispatch registration validation by event category.
/// Returns Ok(()) if the character is eligible, or Err with a user-facing message.
pub async fn validate_registration(
    category: &EventCategory,
    web_db: &WebDB,
    game_db: &GameDB,
    event: &EventDefinition,
    event_id: Uuid,
    cha_num: i32,
    _user_num: i32,
    guild_num: i32,
    cha_school: i32,
    cha_level: i32,
) -> Result<(), String> {
    match category {
        EventCategory::Gvg | EventCategory::GvgKoth => {
            validate_gvg_registration(web_db, game_db, event, event_id, cha_num, guild_num, cha_school, cha_level).await
        }
        EventCategory::Koth => {
            validate_koth_registration(event, cha_level).await
        }
        EventCategory::LevelCapRace => {
            validate_level_cap_race_registration(event, cha_level).await
        }
        EventCategory::SpeedRun => {
            Err("SpeedRun registration is not yet supported".to_string())
        }
    }
}

/// GvG validation: must have guild, guild leader school must match, same-school guild members
async fn validate_gvg_registration(
    web_db: &WebDB,
    game_db: &GameDB,
    event: &EventDefinition,
    event_id: Uuid,
    _cha_num: i32,
    guild_num: i32,
    cha_school: i32,
    cha_level: i32,
) -> Result<(), String> {
    // Must have a guild
    if guild_num <= 0 {
        return Err("You must be in a guild to join GvG.".to_string());
    }

    // Level check
    if cha_level < event.min_level {
        return Err(format!(
            "Character must be at least level {}.",
            event.min_level
        ));
    }

    // Guild leader school must match character school
    match game_db.get_guild_leader_school(guild_num).await {
        Ok(Some((leader_school, leader_name))) if leader_school != cha_school => {
            let leader_school_name = ran_models::events::ref_data::school_name(leader_school);
            let char_school_name = ran_models::events::ref_data::school_name(cha_school);
            return Err(format!(
                "Your guild leader ({}) is from {}. Your character is in {}. Guild leader and member must be in the same school for GvG.",
                leader_name, leader_school_name, char_school_name
            ));
        }
        Err(e) => {
            return Err(format!("Failed to verify guild leader: {}", e));
        }
        _ => {}
    }

    // All guild members already registered must be same school
    match web_db.get_guild_registered_school(event_id, guild_num).await {
        Ok(Some(existing_school)) if existing_school != cha_school => {
            let existing_name = ran_models::events::ref_data::school_name(existing_school);
            let char_school_name = ran_models::events::ref_data::school_name(cha_school);
            return Err(format!(
                "Your guild already has members registered from {}. All guild members must be in the same school for GvG. Your character is in {}.",
                existing_name, char_school_name
            ));
        }
        Err(e) => {
            return Err(format!("Failed to verify guild school: {}", e));
        }
        _ => {}
    }

    Ok(())
}

/// KOTH validation: individual registration, level check only
async fn validate_koth_registration(
    event: &EventDefinition,
    cha_level: i32,
) -> Result<(), String> {
    if cha_level < event.min_level {
        return Err(format!(
            "Character must be at least level {}.",
            event.min_level
        ));
    }
    Ok(())
}

/// LevelCapRace validation: individual, level check + account age check (placeholder for account age)
async fn validate_level_cap_race_registration(
    event: &EventDefinition,
    cha_level: i32,
) -> Result<(), String> {
    if cha_level < event.min_level {
        return Err(format!(
            "Character must be at least level {}.",
            event.min_level
        ));
    }

    // Account age check from category_config
    let _max_account_age_days = match &event.category_config {
        Some(CategoryConfig::LevelCapRace(c)) => c.max_account_age_days,
        _ => 0,
    };
    // Account age validation will be performed by the route handler
    // which has access to UserDB. We just store the constraint here.

    Ok(())
}
