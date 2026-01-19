//! Final damage modifiers (post-random roll).
//!
//! Split into attacker modifiers (OnAttackerFinalMod) and defender modifiers (OnDefenderFinalMod).
//! Order: Attacker mods apply first, then defender mods.

use crate::state::BattleState;
use crate::moves::MoveCategory;
use crate::damage::apply_modifier;

// =============================================================================
// Attacker Final Modifiers
// =============================================================================

/// Tinted Lens: 2x damage on "not very effective" hits
pub fn tinted_lens(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _is_crit: bool,
    damage: u32,
) -> u32 {
    // effectiveness: 4 = 1x, 2 = 0.5x, 1 = 0.25x
    if effectiveness < 4 {
        apply_modifier(damage, 8192) // 2x in 4096-scale
    } else {
        damage
    }
}

/// Sniper: 1.5x damage on critical hits
pub fn sniper(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    is_crit: bool,
    damage: u32,
) -> u32 {
    if is_crit {
        apply_modifier(damage, 6144) // 1.5x
    } else {
        damage
    }
}

// TODO: Neuroforce - 1.25x on super-effective hits
// pub fn neuroforce(...) -> u32

// =============================================================================
// Defender Final Modifiers
// =============================================================================

/// Multiscale: 0.5x damage when at full HP
pub fn multiscale(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _effectiveness: u8,
    _category: MoveCategory,
    _is_contact: bool,
    damage: u32,
) -> u32 {
    if state.hp[defender] == state.max_hp[defender] {
        apply_modifier(damage, 2048) // 0.5x
    } else {
        damage
    }
}

// TODO: Shadow Shield - identical to Multiscale
// pub fn shadow_shield(...) -> u32 { multiscale(...) }

/// Filter / Solid Rock / Prism Armor: 0.75x on super-effective hits
pub fn filter(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _category: MoveCategory,
    _is_contact: bool,
    damage: u32,
) -> u32 {
    if effectiveness > 4 {
        apply_modifier(damage, 3072) // 0.75x
    } else {
        damage
    }
}

// TODO: Fluffy - 0.5x contact damage, 2x Fire damage
// TODO: Ice Scales - 0.5x special damage
// TODO: Punk Rock - 0.5x sound-based damage
