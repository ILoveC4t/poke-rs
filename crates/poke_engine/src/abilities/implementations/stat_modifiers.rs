//! Attack/Defense stat modifying abilities.
//!
//! Called via `OnModifyAttack` or `OnModifyDefense` during stat calculation.

use crate::state::{BattleState, Status};
use crate::moves::{MoveCategory, MoveId};
use crate::damage::{apply_modifier, Modifier};

/// Hustle: 1.5x Attack for physical moves (accuracy penalty handled elsewhere)
pub fn hustle(
    _state: &BattleState,
    _attacker: usize,
    _move_id: MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        apply_modifier(attack as u32, Modifier::ONE_POINT_FIVE).max(1) as u16
    } else {
        attack
    }
}

/// Pure Power / Huge Power: 2x Attack
pub fn huge_power(
    _state: &BattleState,
    _attacker: usize,
    move_id: MoveId,
    _category: MoveCategory,
    attack: u16,
) -> u16 {
    // Huge Power does not boost Body Press (which uses Defense)
    if move_id == MoveId::Bodypress {
        return attack;
    }
    attack.saturating_mul(2)
}

/// Guts: 1.5x Attack when statused
pub fn guts(
    state: &BattleState,
    attacker: usize,
    move_id: MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    // Guts does not boost Body Press
    if move_id == MoveId::Bodypress {
        return attack;
    }

    let status = state.status[attacker];
    if status != Status::NONE && category == MoveCategory::Physical {
        apply_modifier(attack as u32, Modifier::ONE_POINT_FIVE).max(1) as u16
    } else {
        attack
    }
}

/// Gorilla Tactics: 1.5x Attack
pub fn gorilla_tactics(
    _state: &BattleState,
    _attacker: usize,
    _move_id: MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        apply_modifier(attack as u32, Modifier::ONE_POINT_FIVE).max(1) as u16
    } else {
        attack
    }
}

/// Defeatist: 0.5x Attack/SpA when HP <= 50%
pub fn defeatist(
    state: &BattleState,
    attacker: usize,
    _move_id: MoveId,
    _category: MoveCategory,
    attack: u16,
) -> u16 {
    if state.hp[attacker] * 2 <= state.max_hp[attacker] {
        attack / 2
    } else {
        attack
    }
}

// =============================================================================
// Paradox Abilities
// =============================================================================

/// Helper for Protosynthesis/Quark Drive stat calculation
fn calculate_paradox_boost(
    state: &BattleState,
    attacker: usize,
    attack: u16,
    stat_index: usize, // 0=HP(invalid), 1=Atk, 2=Def, 3=SpA, 4=SpD, 5=Spe
) -> u16 {
    // 1. Determine highest stat (ignoring stat modifiers, but including nature/ivs/evs - which are in .stats)
    // The spec says "highest stat", usually referring to the raw stat value.
    // We compare Atk, Def, SpA, SpD, Spe (indices 1-5).
    let stats = state.stats[attacker];
    let mut best_stat_idx = 1;
    let mut best_stat_val = stats[1];

    for i in 2..=5 {
        if stats[i] > best_stat_val {
            best_stat_val = stats[i];
            best_stat_idx = i;
        }
    }

    // If the current stat being calculated matches the best stat, apply boost.
    if stat_index == best_stat_idx {
        if stat_index == 5 {
            // Speed gets 1.5x
            apply_modifier(attack as u32, Modifier::ONE_POINT_FIVE).max(1) as u16
        } else {
            // Others get 1.3x
            apply_modifier(attack as u32, Modifier::ONE_POINT_THREE).max(1) as u16
        }
    } else {
        attack
    }
}

/// Protosynthesis: Boost highest stat in Sun or with Booster Energy
pub fn protosynthesis(
    state: &BattleState,
    attacker: usize,
    move_id: MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    use crate::damage::generations::Weather;
    use crate::items::ItemId;

    let weather = Weather::from_u8(state.weather);
    let item = state.items[attacker];

    // Check condition
    if matches!(weather, Weather::Sun | Weather::HarshSun) || item == ItemId::Boosterenergy {
        // Determine if we are calculating the highest stat
        let stat_idx = if move_id == MoveId::Bodypress {
            2 // Body Press uses Defense
        } else if category == MoveCategory::Physical {
            1 // Attack
        } else {
            3 // Sp. Attack
        };
        calculate_paradox_boost(state, attacker, attack, stat_idx)
    } else {
        attack
    }
}

/// Quark Drive: Boost highest stat in Electric Terrain or with Booster Energy
pub fn quark_drive(
    state: &BattleState,
    attacker: usize,
    move_id: MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    use crate::damage::generations::Terrain;
    use crate::items::ItemId;

    let terrain = Terrain::from_u8(state.terrain);
    let item = state.items[attacker];

    if terrain == Terrain::Electric || item == ItemId::Boosterenergy {
        let stat_idx = if move_id == MoveId::Bodypress {
            2 // Body Press uses Defense
        } else if category == MoveCategory::Physical {
            1 // Attack
        } else {
            3 // Sp. Attack
        };
        calculate_paradox_boost(state, attacker, attack, stat_idx)
    } else {
        attack
    }
}


// =============================================================================
// Defense modifiers
// =============================================================================

// TODO: Marvel Scale - 1.5x Def when statused
// pub fn marvel_scale(state: &BattleState, defender: usize, _attacker: usize, category: MoveCategory, defense: u16) -> u16

/// Fur Coat: 2x Defense
pub fn fur_coat(
    _state: &BattleState,
    _defender: usize,
    _attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        defense.saturating_mul(2)
    } else {
        defense
    }
}

// TODO: Grass Pelt - 1.5x Def in Grassy Terrain
// pub fn grass_pelt(...) -> u16
