//! Item hook implementations.

use crate::moves::MoveCategory;
use crate::species::SpeciesId;
use crate::state::BattleState;
use crate::damage::{apply_modifier, Modifier};
use std::sync::LazyLock;

// Lazily-initialized constants for commonly checked species.
// These are parsed once on first access and cached, eliminating repeated
// string parsing overhead in the hot path of damage calculation.
static CUBONE: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("cubone"));
static MAROWAK: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("marowak"));
static PIKACHU: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("pikachu"));
static CLAMPERL: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("clamperl"));
static LATIOS: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("latios"));
static LATIAS: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("latias"));
static DITTO: LazyLock<Option<SpeciesId>> = LazyLock::new(|| SpeciesId::from_str("ditto"));


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
        apply_modifier(defense.into(), Modifier::ONE_POINT_FIVE).max(1) as u16 // 1.5x
    } else {
        defense
    }
}

// Eviolite: 1.5x Def and SpD if the holder can evolve.
pub fn on_modify_defense_eviolite(
    state: &BattleState,
    defender: usize,
    _attacker: usize,
    _category: MoveCategory,
    defense: u16,
) -> u16 {
    let species = state.species[defender].data();
    if species.flags & crate::species::FLAG_NFE != 0 {
        return apply_modifier(defense.into(), Modifier::ONE_POINT_FIVE).max(1) as u16;
    }
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
                return apply_modifier(attack.into(), Modifier::DOUBLE).max(1) as u16; // 2x
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
            return apply_modifier(attack.into(), Modifier::DOUBLE).max(1) as u16; // 2x
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
        apply_modifier(attack.into(), Modifier::ONE_POINT_FIVE).max(1) as u16 // 1.5x
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
        apply_modifier(attack.into(), Modifier::ONE_POINT_FIVE).max(1) as u16 // 1.5x
    } else {
        attack
    }
}

// Deep Sea Tooth: 2x SpA for Clamperl.
pub fn on_modify_attack_deep_sea_tooth(
    state: &BattleState,
    attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Special {
        if let Some(clamperl) = *CLAMPERL {
            if state.species[attacker] == clamperl {
                return apply_modifier(attack.into(), Modifier::DOUBLE).max(1) as u16;
            }
        }
    }
    attack
}

// Deep Sea Scale: 2x SpD for Clamperl.
pub fn on_modify_defense_deep_sea_scale(
    state: &BattleState,
    defender: usize,
    _attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16 {
    if category == MoveCategory::Special {
        if let Some(clamperl) = *CLAMPERL {
            if state.species[defender] == clamperl {
                return apply_modifier(defense.into(), Modifier::DOUBLE).max(1) as u16;
            }
        }
    }
    defense
}

// Soul Dew: 1.2x SpA for Latios/Latias.
pub fn on_modify_attack_soul_dew(
    state: &BattleState,
    attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Special {
        if let (Some(latios), Some(latias)) = (*LATIOS, *LATIAS) {
            let species = state.species[attacker];
            if species == latios || species == latias {
                return apply_modifier(attack.into(), Modifier::ONE_POINT_TWO).max(1) as u16;
            }
        }
    }
    attack
}

// Soul Dew: 1.2x SpD for Latios/Latias.
pub fn on_modify_defense_soul_dew(
    state: &BattleState,
    defender: usize,
    _attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16 {
    if category == MoveCategory::Special {
        if let (Some(latios), Some(latias)) = (*LATIOS, *LATIAS) {
            let species = state.species[defender];
            if species == latios || species == latias {
                return apply_modifier(defense.into(), Modifier::ONE_POINT_TWO).max(1) as u16;
            }
        }
    }
    defense
}

// Metal Powder: 2x Def for Ditto.
pub fn on_modify_defense_metal_powder(
    state: &BattleState,
    defender: usize,
    _attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        if let Some(ditto) = *DITTO {
            if state.species[defender] == ditto {
                return apply_modifier(defense.into(), Modifier::DOUBLE).max(1) as u16;
            }
        }
    }
    defense
}

// ============================================================================
// Attacker Final Modifiers
// ============================================================================

// Life Orb: 1.3x damage (5324/4096)
pub fn on_attacker_final_mod_life_orb(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    _is_crit: bool,
    damage: u32,
) -> u32 {
    apply_modifier(damage, Modifier::LIFE_ORB)
}

// Expert Belt: 1.2x damage on super effective hits
pub fn on_attacker_final_mod_expert_belt(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _is_crit: bool,
    damage: u32,
) -> u32 {
    if effectiveness > 4 {
        apply_modifier(damage, Modifier::ONE_POINT_TWO)
    } else {
        damage
    }
}

// ============================================================================
// Type-Boosting Base Power Modifiers
// ============================================================================

use crate::types::Type;
use crate::moves::Move;

// Generic type-boost helper - uses dynamic move_type (after type-changing effects)
fn type_boost_bp(move_type: Type, boost_type: Type, bp: u16) -> u16 {
    if move_type == boost_type {
        apply_modifier(bp.into(), Modifier::ONE_POINT_TWO).max(1) as u16
    } else {
        bp
    }
}

pub fn on_modify_bp_charcoal(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Fire, bp)
}

pub fn on_modify_bp_mystic_water(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Water, bp)
}

pub fn on_modify_bp_miracle_seed(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Grass, bp)
}

pub fn on_modify_bp_magnet(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Electric, bp)
}

pub fn on_modify_bp_never_melt_ice(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ice, bp)
}

pub fn on_modify_bp_black_belt(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Fighting, bp)
}

pub fn on_modify_bp_poison_barb(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Poison, bp)
}

pub fn on_modify_bp_soft_sand(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ground, bp)
}

pub fn on_modify_bp_sharp_beak(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Flying, bp)
}

pub fn on_modify_bp_twisted_spoon(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Psychic, bp)
}

pub fn on_modify_bp_silver_powder(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Bug, bp)
}

pub fn on_modify_bp_hard_stone(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Rock, bp)
}

pub fn on_modify_bp_spell_tag(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ghost, bp)
}

pub fn on_modify_bp_dragon_fang(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Dragon, bp)
}

pub fn on_modify_bp_black_glasses(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Dark, bp)
}

pub fn on_modify_bp_metal_coat(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Steel, bp)
}

pub fn on_modify_bp_silk_scarf(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Normal, bp)
}

// ============================================================================
// Plate Items (1.2x type boost, same as corresponding items)
// ============================================================================

pub fn on_modify_bp_flame_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Fire, bp)
}

pub fn on_modify_bp_splash_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Water, bp)
}

pub fn on_modify_bp_meadow_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Grass, bp)
}

pub fn on_modify_bp_zap_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Electric, bp)
}

pub fn on_modify_bp_icicle_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ice, bp)
}

pub fn on_modify_bp_fist_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Fighting, bp)
}

pub fn on_modify_bp_toxic_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Poison, bp)
}

pub fn on_modify_bp_earth_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ground, bp)
}

pub fn on_modify_bp_sky_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Flying, bp)
}

pub fn on_modify_bp_mind_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Psychic, bp)
}

pub fn on_modify_bp_insect_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Bug, bp)
}

pub fn on_modify_bp_stone_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Rock, bp)
}

pub fn on_modify_bp_spooky_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Ghost, bp)
}

pub fn on_modify_bp_draco_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Dragon, bp)
}

pub fn on_modify_bp_dread_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Dark, bp)
}

pub fn on_modify_bp_iron_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Steel, bp)
}

pub fn on_modify_bp_pixie_plate(_state: &BattleState, _attacker: usize, _defender: usize, _move_data: &Move, move_type: Type, bp: u16) -> u16 {
    type_boost_bp(move_type, Type::Fairy, bp)
}

// ============================================================================
// Metronome Item (consecutive move bonus)
// ============================================================================

/// Metronome: Boosts damage of consecutively used moves.
///
/// Scaling: 1.0x base, +0.2x per consecutive use of the same move, up to 2.0x.
/// The multiplier is applied to base power.
///
/// Edge cases handled by state tracking:
/// - Reset on switching moves
/// - Reset on move failure/Protect block
/// - Reset on switch-out
/// - Uses effective move (after Sleep Talk, Copycat, etc.)
pub fn on_modify_bp_metronome(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    move_data: &Move,
    _move_type: Type,
    bp: u16,
) -> u16 {
    // Only apply to damaging moves with base power > 0
    if bp == 0 || move_data.power == 0 {
        return bp;
    }

    let multiplier = state.metronome_multiplier(attacker);
    if multiplier == 4096 {
        return bp; // No boost (1.0x)
    }

    // Apply multiplier using the same rounding as other modifiers
    apply_modifier(bp.into(), Modifier::new(multiplier)).max(1) as u16
}

// ============================================================================
// Speed Modifiers
// ============================================================================

// Choice Scarf: 1.5x Speed
pub fn on_modify_speed_choice_scarf(
    _state: &BattleState,
    _entity: usize,
    speed: u16,
) -> u16 {
    // 1.5x (Speed * 3 / 2)
    (speed as u32 * 3 / 2).min(u16::MAX as u32) as u16
}

// Iron Ball: 0.5x Speed
pub fn on_modify_speed_iron_ball(
    _state: &BattleState,
    _entity: usize,
    speed: u16,
) -> u16 {
    speed / 2
}

// ============================================================================
// Grounding Modifiers
// ============================================================================

// Air Balloon: Ungrounded (returns Some(false))
pub fn on_check_grounded_air_balloon(
    _state: &BattleState,
    _entity: usize,
) -> Option<bool> {
    Some(false)
}

// Iron Ball: Grounded (returns Some(true))
pub fn on_check_grounded_iron_ball(
    _state: &BattleState,
    _entity: usize,
) -> Option<bool> {
    Some(true)
}

// Heavy-Duty Boots: Immune to all entry hazards
pub fn on_hazard_immunity_heavy_duty_boots(
    _state: &BattleState,
    _entity: usize,
    _hazard: crate::state::Hazard,
) -> bool {
    true
}
