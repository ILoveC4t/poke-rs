//! Type effectiveness calculation with immunity overrides.
//!
//! This module provides a reusable function for calculating type effectiveness
//! with all modifiers applied (Ring Target, Iron Ball, Scrappy, etc.).

use crate::abilities::AbilityId;
use crate::items::ItemId;
use crate::state::BattleState;
use crate::types::Type;

/// Calculate type effectiveness with all immunity overrides applied.
///
/// This handles:
/// - Ring Target (negates all type immunities)
/// - Iron Ball / Gravity grounding for Ground vs Flying
/// - Scrappy / Mind's Eye (Normal/Fighting vs Ghost)
///
/// Returns effectiveness on 4-scale (0=immune, 1=0.25x, 2=0.5x, 4=1x, 8=2x, 16=4x).
///
/// # Arguments
/// * `move_type` - The type of the attacking move
/// * `defender` - Index of the defending Pokémon
/// * `state` - Reference to battle state
/// * `attacker_ability` - Ability of the attacker (for Scrappy/Mind's Eye)
/// * `generation` - Current generation number (for gen-specific mechanics)
/// * `type_eff_fn` - Function to get base type effectiveness (from GenMechanics)
pub fn calculate_effectiveness<F>(
    move_type: Type,
    defender: usize,
    state: &BattleState,
    attacker_ability: AbilityId,
    generation: u8,
    type_eff_fn: F,
) -> u8
where
    F: Fn(Type, Type, Option<Type>) -> u8,
{
    let def_type1 = state.types[defender][0];
    let def_type2 = state.types[defender][1];

    // Helper to get effectiveness against a single type
    // respecting immunity overrides (Ring Target, Iron Ball/Gravity)
    let get_single_eff = |type_to_check: Type| -> u8 {
        let base_eff = type_eff_fn(move_type, type_to_check, None);

        if base_eff == 0 {
            // Check Ring Target (negates ALL type immunities)
            if state.items[defender] == ItemId::Ringtarget {
                return 4; // 1x
            }

            // Check Iron Ball / Gravity vs Flying (Ground moves only)
            if move_type == Type::Ground
                && type_to_check == Type::Flying
                && state.is_grounded(defender)
            {
                // Gen 5+ mechanics: Grounded Flying types resist Ground (0.5x)
                // Gen 4- mechanics: Grounded Flying types take Neutral from Ground (1x)
                return if generation >= 5 { 2 } else { 4 };
            }

            // Scrappy / Mind's Eye: Allow Normal/Fighting to hit Ghost
            if type_to_check == Type::Ghost
                && (move_type == Type::Normal || move_type == Type::Fighting)
                && (attacker_ability == AbilityId::Scrappy || attacker_ability == AbilityId::Mindseye)
            {
                return 4; // 1x (Neutral)
            }
        }
        base_eff
    };

    let eff1 = get_single_eff(def_type1);
    let eff2 = if def_type2 != def_type1 {
        get_single_eff(def_type2)
    } else {
        4 // 1x (Neutral)
    };

    // Combine effectiveness (4 scale: 4*4/4 = 4)
    (eff1 as u16 * eff2 as u16 / 4) as u8
}
