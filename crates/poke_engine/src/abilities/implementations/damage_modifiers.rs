//! Base power modifying abilities.
//!
//! These are called via `OnModifyBasePower` during the damage calculation pipeline.

use crate::state::BattleState;
use crate::moves::{Move, MoveFlags};
use crate::damage::{Modifier, apply_modifier};

/// Technician: 1.5x power for moves with BP ≤ 60
pub fn technician(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _move_data: &Move,
    bp: u16,
) -> u16 {
    if bp <= 60 {
        bp * 3 / 2
    } else {
        bp
    }
}

/// Iron Fist: 1.2x power for punch moves
pub fn iron_fist(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.flags.contains(MoveFlags::PUNCH) {
        bp * 6 / 5
    } else {
        bp
    }
}

/// Tough Claws: 1.3x power for contact moves
pub fn tough_claws(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.flags.contains(MoveFlags::CONTACT) {
        // 1.3x = 5461/4096, but we can approximate as bp * 13 / 10
        // For exact 4096-scale: (bp as u32 * 5461 / 4096) as u16
        (bp as u32 * 5461 / 4096) as u16
    } else {
        bp
    }
}

// =============================================================================
// Implemented Abilities
// =============================================================================

/// Reckless: 1.2x for recoil moves
pub fn reckless(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.flags.contains(MoveFlags::RECOIL) {
        // 1.2x (4915/4096)
        (bp as u32 * 4915 / 4096) as u16
    } else {
        bp
    }
}

/// Steelworker: 1.5x for Steel moves
pub fn steelworker(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.primary_type == crate::types::Type::Steel {
        bp * 3 / 2
    } else {
        bp
    }
}

/// Water Bubble: 2x for Water moves
pub fn water_bubble(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.primary_type == crate::types::Type::Water {
        bp * 2
    } else {
        bp
    }
}

/// Punk Rock: 1.3x for sound moves (Base Power increase)
pub fn punk_rock(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    // Note: Flag is generated from JSON key "sound"
    if move_data.flags.contains(MoveFlags::SOUND) {
        // 1.3x (5325/4096)
        (bp as u32 * 5325 / 4096) as u16
    } else {
        bp
    }
}

/// Rivalry: 1.25x for same gender, 0.75x for opposite gender
pub fn rivalry(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    _move_data: &Move,
    bp: u16,
) -> u16 {
    use crate::entities::Gender;
    
    let attacker_gender = state.gender[attacker];
    let defender_gender = state.gender[defender];
    
    if attacker_gender != Gender::Genderless && defender_gender != Gender::Genderless {
        if attacker_gender == defender_gender {
            // 1.25x (5120/4096)
            (bp as u32 * 5120 / 4096) as u16
        } else {
            // 0.75x (3072/4096)
            (bp as u32 * 3072 / 4096) as u16
        }
    } else {
        bp
    }
}

/// Sheer Force: 1.3x for moves with secondary effects (also disables those effects)
pub fn sheer_force(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.flags.contains(MoveFlags::HAS_SECONDARY_EFFECTS) {
        // 1.3x (5325/4096)
        (bp as u32 * 5325 / 4096) as u16
    } else {
        bp
    }
}

/// Sand Force: 1.3x for Rock/Ground/Steel moves in Sandstorm
pub fn sand_force(
    state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    use crate::types::Type;
    
    // Import Weather from damage module (not abilities::weather)
    // This provides the from_u8() method for converting BattleState.weather
    use crate::damage::generations::Weather;
    
    if Weather::from_u8(state.weather) == Weather::Sand {
        // Note: Using primary_type from move_data. Type-changing abilities like
        // Pixilate aren't implemented yet. When they are, the hook system may
        // need to be extended to pass the modified type.
        if matches!(move_data.primary_type, Type::Rock | Type::Ground | Type::Steel) {
            // 1.3x (5325/4096)
            (bp as u32 * 5325 / 4096) as u16
        } else {
            bp
        }
    } else {
        bp
    }
}

/// Analytic: 1.3x if moving last
pub fn analytic(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    use crate::state::TurnOrder;
    // We use the attacker's actual priority, but assume 0 for the defender
    // as we don't know their move.
    // If the attacker is slower (or moves later due to priority), compare_turn_order returns Second.
    if state.compare_turn_order(attacker, move_data.priority, defender, 0) == TurnOrder::Second {
        // 1.3x (5325/4096)
        apply_modifier(bp as u32, Modifier::ONE_POINT_THREE) as u16
    } else {
        bp
    }
}

/// -ate abilities: 1.2x for converted Normal moves
/// (Aerilate, Pixilate, Refrigerate, Galvanize)
pub fn ate_boost(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_data: &Move,
    bp: u16,
) -> u16 {
    if move_data.primary_type == crate::types::Type::Normal {
        // 1.2x (4915/4096)
        apply_modifier(bp as u32, Modifier::ONE_POINT_TWO) as u16
    } else {
        bp
    }
}

// TODO: Strong Jaw - 1.5x for bite moves
// pub fn strong_jaw(...) -> u16

// TODO: Mega Launcher - 1.5x for pulse/aura moves
// pub fn mega_launcher(...) -> u16
