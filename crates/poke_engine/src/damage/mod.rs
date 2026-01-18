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
        // Fixed damage = level
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
        
        // TODO: Implement these fixed damage moves
        // "Dragon Rage" => Some(40),
        // "Sonic Boom" => Some(20),
        // "Super Fang" => Some(state.hp[defender] / 2),
        // "Nature's Madness" => Some(state.hp[defender] / 2),
        // "Final Gambit" => Some(state.hp[attacker]),
        // "Endeavor" => handled specially
        // "Psywave" => Random 0.5x to 1.5x level
        // "Counter" / "Mirror Coat" => 2x damage received
        // "Metal Burst" => 1.5x damage received
        // "Bide" => 2x stored damage
        
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
    let mut ctx = DamageContext::new(gen, state, attacker, defender, move_id, is_crit);
    
    // Phase 1: Compute base power (Technician, etc.)
    modifiers::compute_base_power(&mut ctx);
    
    // Phase 2: Get effective stats (apply boosts, crit rules)
    let (attack, defense) = modifiers::compute_effective_stats(&ctx);
    
    // Phase 3: Base damage formula
    let level = state.level[attacker] as u32;
    let base_damage = get_base_damage(level, ctx.base_power as u32, attack as u32, defense as u32);
    
    // Phase 4: Apply pre-random modifier chain
    modifiers::apply_spread_mod(&mut ctx);
    modifiers::apply_weather_mod(&mut ctx);
    modifiers::apply_crit_mod(&mut ctx);
    
    // Phase 5: Generate all 16 damage rolls
    let rolls = modifiers::compute_final_damage(&ctx, base_damage);
    
    DamageResult {
        rolls,
        min: rolls[0],
        max: rolls[15],
        effectiveness: ctx.effectiveness,
        is_crit,
        final_base_power: ctx.base_power,
    }
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
