use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefClass {
    pub cha_class: i32,
    pub class_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefSchool {
    pub cha_school: i32,
    pub school_name: String,
    pub school_abbr: String,
}

/// Maps a raw game ChaClass value (which varies by gender) to a simplified
/// event class ID that is gender-neutral.
///
/// Event class IDs:
///   1 = Shaman, 2 = Brawler, 3 = Assassin, 4 = Scientist,
///   5 = Swordsman, 6 = Magician, 7 = Archer, 8 = Extreme, 9 = Shaper
///
/// Returns `None` for unrecognized game class values.
pub fn map_game_class_to_event_class(game_class: i32) -> Option<i32> {
    match game_class {
        8 | 512 => Some(1),        // Shaman
        1 | 64 => Some(2),         // Brawler
        4096 | 8192 => Some(3),    // Assassin
        1024 | 2048 => Some(4),    // Scientist
        2 | 128 => Some(5),        // Swordsman
        16384 | 32768 => Some(6),  // Magician
        4 | 256 => Some(7),        // Archer
        16 | 32 => Some(8),        // Extreme
        262144 | 524288 => Some(9), // Shaper
        _ => None,
    }
}

/// Returns the display name for a school ID.
pub fn school_name(cha_school: i32) -> &'static str {
    match cha_school {
        0 => "Sacred Gate",
        1 => "Mystic Peak",
        2 => "Phoenix",
        _ => "Unknown",
    }
}

/// Returns the display name for an event class ID.
pub fn event_class_name(event_class: i32) -> &'static str {
    match event_class {
        1 => "Shaman",
        2 => "Brawler",
        3 => "Assassin",
        4 => "Scientist",
        5 => "Swordsman",
        6 => "Magician",
        7 => "Archer",
        8 => "Extreme",
        9 => "Shaper",
        _ => "Unknown",
    }
}
