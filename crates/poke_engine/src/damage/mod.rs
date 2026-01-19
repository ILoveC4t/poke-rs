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
mod special_moves;
#[cfg(test)]
mod special_moves_tests;
#[cfg(test)]
mod conditional_moves_tests;

pub use context::DamageContext;
pub use formula::{get_base_damage, pokeround, of16, of32, chain_mods, apply_modifier};
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
// get_fixed_damage moved to special_moves/fixed.rs

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
    calculate_damage_with_overrides(gen, state, attacker, defender, move_id, is_crit, None)
}

/// Calculate damage for a move with optional overrides.
///
/// This is used by fixtures or specialized callers that need to
/// override base power (e.g., Z-Moves in tests).
pub fn calculate_damage_with_overrides<G: GenMechanics>(
    gen: G,
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_id: MoveId,
    is_crit: bool,
    base_power_override: Option<u16>,
) -> DamageResult {
    let move_data = move_id.data();
    
    // Status moves deal no damage
    if move_data.category == crate::moves::MoveCategory::Status {
        return DamageResult::zero();
    }
    
    // Check for fixed damage moves first
    if let Some(fixed_damage) = special_moves::get_fixed_damage(move_id, state, attacker, defender) {
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
    if move_data.power == 0 && !special_moves::is_variable_power(move_id) {
        return DamageResult::zero();
    }
    
    // Create damage context
    let mut ctx = DamageContext::new(gen, state, attacker, defender, move_id, is_crit);
    if let Some(bp) = base_power_override {
        ctx.base_power = bp;
    }
    
    // Apply special move overrides (e.g. Weather Ball, Struggle, Flying Press)
    special_moves::apply_special_moves(&mut ctx);

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
        
        // Pikachu vs Gengar (Ghost/Poison)
        // Note: We use Gengar because Gastly has Levitate (immune to Ground).
        // Gengar has Cursed Body (Gen 9), so it should be hit by Ground.
        if let Some(config) = PokemonConfig::from_str("pikachu") {
            config.level(50).spawn(&mut state, 0, 0);
        }
        if let Some(config) = PokemonConfig::from_str("gengar") {
            config.level(50).spawn(&mut state, 1, 0);
        }
        
        // Get a Ground move (Earthquake)
        let earthquake = MoveId::from_str("earthquake").expect("earthquake should exist");
        
        // Calculate damage
        let result = calculate_damage(Gen9, &state, 0, 6, earthquake, false);
        
        // Gengar is Ghost/Poison - neither is immune to Ground type-wise.
        // So this should deal super effective damage (2x to Poison)
        assert!(result.effectiveness > 0, "Ghost/Poison is not immune to Ground");
    }
}
