//! Item hook implementations.

use crate::moves::MoveCategory;
use crate::species::SpeciesId;
use crate::state::BattleState;
use crate::damage::apply_modifier;
use std::sync::LazyLock;

// Lazily-initialized constants for commonly checked species.
// These are parsed once on first access and cached, eliminating repeated
// string parsing overhead in the hot path of damage calculation.
static CUBONE: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("cubone"));
static MAROWAK: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("marowak"));
static PIKACHU: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("pikachu"));


// Assault Vest: 1.5x SpD, but can only use damaging moves.
// The move restriction part is handled elsewhere (or not at all yet).
pub fn on_modify_defense_assault_vest(
    _state: &BattleState,
    _defender: usize,
    _attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16 {
    if category == MoveCategory::Special {
        apply_modifier(defense.into(), 6144).max(1) as u16 // 1.5x
    } else {
        defense
    }
}

// Eviolite: 1.5x Def and SpD if the holder can evolve.
// NOTE: This is currently disabled because evolution data is not yet available in Species struct.
// TODO: Add evolution data to Species and implement proper Eviolite logic.
pub fn on_modify_defense_eviolite(
    _state: &BattleState,
    _defender: usize,
    _attacker: usize,
    _category: MoveCategory,
    defense: u16,
) -> u16 {
    // Evolution data not yet implemented
    defense
}

// Thick Club: 2x Atk for Cubone or Marowak.
// NOTE: This function is not currently used because Thick Club is filtered out by build.rs
// due to being marked as "isNonstandard": "Past" in the items data.
#[allow(dead_code)]
pub fn on_modify_attack_thick_club(
    state: &BattleState,
    attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        let species = state.species[attacker];
        if let (Some(cubone), Some(marowak)) = (*CUBONE, *MAROWAK) {
            if species == cubone || species == marowak {
                return apply_modifier(attack.into(), 8192).max(1) as u16; // 2x
            }
        }
    }
    attack
}

// Light Ball: 2x Atk and SpA for Pikachu.
pub fn on_modify_attack_light_ball(
    state: &BattleState,
    attacker: usize,
    _category: MoveCategory,
    attack: u16,
) -> u16 {
    if let Some(pikachu) = *PIKACHU {
        if state.species[attacker] == pikachu {
            return apply_modifier(attack.into(), 8192).max(1) as u16; // 2x
        }
    }
    attack
}

// Choice Band: 1.5x Attack, but locks user into one move.
pub fn on_modify_attack_choice_band(
    _state: &BattleState,
    _attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        apply_modifier(attack.into(), 6144).max(1) as u16 // 1.5x
    } else {
        attack
    }
}

// Choice Specs: 1.5x Special Attack, but locks user into one move.
pub fn on_modify_attack_choice_specs(
    _state: &BattleState,
    _attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Special {
        apply_modifier(attack.into(), 6144).max(1) as u16 // 1.5x
    } else {
        attack
    }
}
