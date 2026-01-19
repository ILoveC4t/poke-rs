//! Base power modifying abilities.
//!
//! These are called via `OnModifyBasePower` during the damage calculation pipeline.

use crate::state::BattleState;
use crate::moves::{Move, MoveFlags};

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

// TODO: Sheer Force - 1.3x for moves with secondary effects, disables those effects
// pub fn sheer_force(...) -> u16

// TODO: Sand Force - 1.3x for Rock/Ground/Steel moves in Sandstorm
// pub fn sand_force(...) -> u16

// TODO: Analytic - 1.3x if moving last
// pub fn analytic(...) -> u16

// TODO: Strong Jaw - 1.5x for bite moves
// pub fn strong_jaw(...) -> u16

// TODO: Mega Launcher - 1.5x for pulse/aura moves
// pub fn mega_launcher(...) -> u16
