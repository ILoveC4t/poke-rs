//! Damage calculation pipeline.
//!
//! This module implements a modular, pipeline-style damage calculator
//! focused on Generation 9 mechanics with extensibility for other generations.
//!
//! # Architecture
//!
//! The damage calculation follows the official Pokémon damage formula:
//! 1. **Base Damage**: `floor((floor(2 * Level / 5 + 2) * Power * Atk / Def) / 50) + 2`
//! 2. **Modifier Chain**: Sequential 4096-scale multipliers applied in strict order
//!
//! # Usage
//!
//! ```ignore
//! use poke_engine::damage::{calculate_damage, Gen9};
//!
//! let result = calculate_damage(
//!     Gen9,
//!     &battle_state,
//!     attacker_idx,
//!     defender_idx,
//!     move_id,
//!     false, // is_crit
//! );
//!
//! // result.rolls contains all 16 damage values (85-100% rolls)
//! ```

mod context;
mod formula;
mod modifiers;
pub mod generations;

pub use context::DamageContext;
pub use formula::{get_base_damage, pokeround, of16, of32, chain_mods};
pub use generations::{GenMechanics, Generation, Gen9};

use crate::moves::MoveId;
use crate::state::BattleState;

/// Result of a damage calculation.
#[derive(Clone, Debug)]
pub struct DamageResult {
    /// All 16 possible damage values (random roll 85-100)
    pub rolls: [u16; 16],
    
    /// Minimum damage (roll index 0)
    pub min: u16,
    
    /// Maximum damage (roll index 15)
    pub max: u16,
    
    /// Type effectiveness multiplier (4 = neutral, 8 = 2x, etc.)
    pub effectiveness: u8,
    
    /// Whether the attack was a critical hit
    pub is_crit: bool,
    
    /// Base power after modifications
    pub final_base_power: u16,
}

impl DamageResult {
    /// Create a zero-damage result (for immunities or status moves)
    pub fn zero() -> Self {
        Self {
            rolls: [0; 16],
            min: 0,
            max: 0,
            effectiveness: 0,
            is_crit: false,
            final_base_power: 0,
        }
    }
}

/// Check if a move deals fixed damage (not affected by stats/type).
///
/// Returns `Some(damage)` for fixed damage moves, `None` otherwise.
fn get_fixed_damage(
    move_id: MoveId,
    state: &BattleState,
    attacker: usize,
    defender: usize,
) -> Option<u16> {
    let move_name = move_id.data().name;
    let level = state.level[attacker] as u16;
    
    // Check for immunity first for relevant moves
    let defender_types = state.types[defender];
    
    match move_name {
        // ====================================================================
        // Level-based fixed damage
        // ====================================================================
        
        "Night Shade" => {
            // Ghost-type move, Normal-types are immune
            if defender_types[0] == crate::types::Type::Normal 
                || defender_types[1] == crate::types::Type::Normal {
                Some(0)
            } else {
                Some(level)
            }
        }
        "Seismic Toss" => {
            // Fighting-type move, Ghost-types are immune
            if defender_types[0] == crate::types::Type::Ghost 
                || defender_types[1] == crate::types::Type::Ghost {
                Some(0)
            } else {
                Some(level)
            }
        }
        
        // ====================================================================
        // Constant fixed damage (removed in Gen 5+)
        // ====================================================================
        
        "Dragon Rage" => {
            // Dragon-type, Fairy-types are immune (Gen 6+)
            if defender_types[0] == crate::types::Type::Fairy 
                || defender_types[1] == crate::types::Type::Fairy {
                Some(0)
            } else {
                Some(40)
            }
        }
        "Sonic Boom" => {
            // Normal-type, Ghost-types are immune
            if defender_types[0] == crate::types::Type::Ghost 
                || defender_types[1] == crate::types::Type::Ghost {
                Some(0)
            } else {
                Some(20)
            }
        }
        
        // ====================================================================
        // HP percentage-based damage
        // ====================================================================
        
        // Super Fang / Nature's Madness: 50% of target's current HP
        "Super Fang" | "Nature's Madness" => {
            // Normal-type (Super Fang) - Ghost immune
            // Fairy-type (Nature's Madness) - no immunities by type
            if move_name == "Super Fang" 
                && (defender_types[0] == crate::types::Type::Ghost 
                    || defender_types[1] == crate::types::Type::Ghost) {
                Some(0)
            } else {
                Some((state.hp[defender] / 2).max(1))
            }
        }
        
        // Guardian of Alola: 75% of target's current HP
        "Guardian of Alola" => {
            Some((state.hp[defender] * 3 / 4).max(1))
        }
        
        // Ruination: 50% of target's current HP (Gen 9)
        "Ruination" => {
            Some((state.hp[defender] / 2).max(1))
        }
        
        // ====================================================================
        // Attacker HP-based damage
        // ====================================================================
        
        // Final Gambit: damage = attacker's current HP (attacker faints)
        "Final Gambit" => {
            // Fighting-type, Ghost-types are immune
            if defender_types[0] == crate::types::Type::Ghost 
                || defender_types[1] == crate::types::Type::Ghost {
                Some(0)
            } else {
                Some(state.hp[attacker])
            }
        }
        
        // ====================================================================
        // Endeavor: special handling (reduces target to attacker's HP)
        // ====================================================================
        
        "Endeavor" => {
            // Normal-type, Ghost-types are immune
            if defender_types[0] == crate::types::Type::Ghost 
                || defender_types[1] == crate::types::Type::Ghost {
                Some(0)
            } else {
                let attacker_hp = state.hp[attacker];
                let defender_hp = state.hp[defender];
                if defender_hp > attacker_hp {
                    Some(defender_hp - attacker_hp)
                } else {
                    Some(0) // Fails if target HP <= attacker HP
                }
            }
        }
        
        // TODO: Implement these (require battle history tracking)
        // "Counter" => 2x physical damage received this turn
        // "Mirror Coat" => 2x special damage received this turn
        // "Metal Burst" => 1.5x damage received this turn
        // "Bide" => 2x stored damage over 2 turns
        // "Psywave" => Random damage between level * 0.5 and level * 1.5
        
        _ => None,
    }
}

/// Calculate damage for a move.
///
/// This is the main entry point for damage calculation. It handles:
/// - Base power computation (including weight-based moves, Technician, etc.)
/// - Effective stat calculation with boosts
/// - Full modifier chain (weather, crit, STAB, type effectiveness, etc.)
/// - Returns all 16 random roll values
///
/// # Arguments
///
/// * `gen` - Generation mechanics to use
/// * `state` - Current battle state
/// * `attacker` - Entity index of the attacker (0-11)
/// * `defender` - Entity index of the defender (0-11)
/// * `move_id` - The move being used
/// * `is_crit` - Whether this is a critical hit
///
/// # Returns
///
/// `DamageResult` containing all 16 damage rolls and metadata.
pub fn calculate_damage<G: GenMechanics>(
    gen: G,
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_id: MoveId,
    is_crit: bool,
) -> DamageResult {
    let move_data = move_id.data();
    
    // Status moves deal no damage
    if move_data.category == crate::moves::MoveCategory::Status {
        return DamageResult::zero();
    }
    
    // Check for fixed damage moves first
    if let Some(fixed_damage) = get_fixed_damage(move_id, state, attacker, defender) {
        return DamageResult {
            rolls: [fixed_damage; 16],
            min: fixed_damage,
            max: fixed_damage,
            effectiveness: 4, // Neutral
            is_crit: false,
            final_base_power: 0,
        };
    }
    
    // Power 0 moves with no special handling deal no damage
    if move_data.power == 0 {
        return DamageResult::zero();
    }
    
    // Create damage context
    let ctx = DamageContext::new(gen, state, attacker, defender, move_id, is_crit);

    // Delegate to standard formula (Gen 3+)
    // TODO: Delegate to gen.calculate_damage(&ctx) once trait is updated
    formula::calculate_standard(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PokemonConfig;
    use crate::moves::MoveId;
    
    #[test]
    fn test_basic_damage_calc() {
        // Set up a simple battle: Pikachu vs Bulbasaur
        let mut state = BattleState::new();
        
        // Spawn attacker (player 0, slot 0)
        if let Some(mut config) = PokemonConfig::from_str("pikachu") {
            config = config.level(50);
            config.spawn(&mut state, 0, 0);
        }
        
        // Spawn defender (player 1, slot 0)
        if let Some(mut config) = PokemonConfig::from_str("bulbasaur") {
            config = config.level(50);
            config.spawn(&mut state, 1, 0);
        }
        
        // Get Thunderbolt
        let thunderbolt = MoveId::from_str("thunderbolt").expect("thunderbolt should exist");
        
        // Calculate damage
        let result = calculate_damage(
            Gen9,
            &state,
            0,  // attacker = Pikachu
            6,  // defender = Bulbasaur (player 1, slot 0)
            thunderbolt,
            false,
        );
        
        // Should deal damage (Electric vs Grass/Poison = neutral)
        assert!(result.max > 0, "Should deal some damage");
        assert!(result.min <= result.max, "Min should be <= max");
        assert_eq!(result.rolls.len(), 16, "Should have 16 rolls");
    }
    
    #[test]
    fn test_type_immunity() {
        let mut state = BattleState::new();
        
        // Pikachu vs Gastly (Ghost/Poison - immune to Normal)
        if let Some(config) = PokemonConfig::from_str("pikachu") {
            config.level(50).spawn(&mut state, 0, 0);
        }
        if let Some(config) = PokemonConfig::from_str("gastly") {
            config.level(50).spawn(&mut state, 1, 0);
        }
        
        // Get a Ground move (Earthquake)
        let earthquake = MoveId::from_str("earthquake").expect("earthquake should exist");
        
        // Calculate damage (Ground vs Ghost = immune because Gastly has Levitate, 
        // but type-wise Ghost isn't immune to Ground - we need a better test)
        let result = calculate_damage(Gen9, &state, 0, 6, earthquake, false);
        
        // Gastly is Ghost/Poison - neither is immune to Ground
        // So this should deal super effective damage (2x to Poison)
        assert!(result.effectiveness > 0, "Ghost/Poison is not immune to Ground");
    }
}
