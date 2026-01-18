//! Damage calculation context.
//!
//! The `DamageContext` struct holds all relevant information for a single
//! damage calculation, including the generation mechanics, battle state,
//! and accumulated modifiers.

use crate::abilities::AbilityId;
use crate::moves::{Move, MoveCategory, MoveId};
use crate::state::BattleState;
use crate::types::Type;
use super::generations::GenMechanics;

/// Context for a single damage calculation.
///
/// This struct is passed through the modifier pipeline, accumulating
/// multipliers and flags as each stage processes it.
pub struct DamageContext<'a, G: GenMechanics> {
    /// Generation mechanics
    pub gen: G,
    
    /// Reference to battle state
    pub state: &'a BattleState,
    
    // ========================================================================
    // Participants
    // ========================================================================
    
    /// Attacker's entity index (0-11)
    pub attacker: usize,
    
    /// Defender's entity index (0-11)
    pub defender: usize,
    
    // ========================================================================
    // Move Information
    // ========================================================================
    
    /// Move being used
    pub move_id: MoveId,
    
    /// Static move data
    pub move_data: &'static Move,
    
    /// Base power (may be modified by abilities/items)
    pub base_power: u16,
    
    /// Effective move category (can differ from move_data.category for special cases)
    pub category: MoveCategory,
    
    /// Move type (can be modified by abilities like Pixilate)
    pub move_type: Type,
    
    // ========================================================================
    // Calculation Flags
    // ========================================================================
    
    /// Whether this is a critical hit
    pub is_crit: bool,
    
    /// Whether this is a spread move hitting multiple targets
    pub is_spread: bool,
    
    /// Whether the attacker is grounded (for terrain)
    pub attacker_grounded: bool,
    
    /// Whether the defender is grounded (for terrain)
    pub defender_grounded: bool,
    
    // ========================================================================
    // Modifiers
    // ========================================================================
    
    /// Accumulated modifier chain (4096 = 1.0x)
    pub chain_mod: u32,
    
    /// Type effectiveness (4 = 1x, 8 = 2x, etc.)
    pub effectiveness: u8,
    
    /// Whether STAB applies
    pub has_stab: bool,
    
    /// Whether Adaptability applies (2x STAB instead of 1.5x)
    pub has_adaptability: bool,
    
    /// Whether this is a Tera STAB (Gen 9)
    pub is_tera_stab: bool,
    
    // ========================================================================
    // Attacker/Defender cached info
    // ========================================================================
    
    /// Attacker's ability
    pub attacker_ability: AbilityId,
    
    /// Defender's ability
    pub defender_ability: AbilityId,
}

impl<'a, G: GenMechanics> DamageContext<'a, G> {
    /// Create a new damage context.
    pub fn new(
        gen: G,
        state: &'a BattleState,
        attacker: usize,
        defender: usize,
        move_id: MoveId,
        is_crit: bool,
    ) -> Self {
        let move_data = move_id.data();
        let attacker_types = state.types[attacker];
        let move_type = move_data.primary_type;
        
        // Check STAB
        let has_stab = move_type == attacker_types[0] || move_type == attacker_types[1];
        
        // Check Adaptability
        let attacker_ability = state.abilities[attacker];
        let has_adaptability = attacker_ability == AbilityId::Adaptability;
        
        // Check if grounded (simplified - doesn't account for all edge cases)
        // TODO: Add proper grounded check (Levitate, Air Balloon, Magnet Rise, etc.)
        let attacker_grounded = !matches!(attacker_types[0], Type::Flying) 
            && !matches!(attacker_types[1], Type::Flying)
            && attacker_ability != AbilityId::Levitate;
        let defender_grounded = !matches!(state.types[defender][0], Type::Flying)
            && !matches!(state.types[defender][1], Type::Flying)
            && state.abilities[defender] != AbilityId::Levitate;
        
        // Calculate type effectiveness
        let def_type1 = state.types[defender][0];
        let def_type2 = state.types[defender][1];
        let def_type2_opt = if def_type2 != def_type1 { Some(def_type2) } else { None };
        let effectiveness = gen.type_effectiveness(move_type, def_type1, def_type2_opt);
        
        Self {
            gen,
            state,
            attacker,
            defender,
            move_id,
            move_data,
            base_power: move_data.power,
            category: move_data.category,
            move_type,
            is_crit,
            is_spread: false, // Set by caller if applicable
            attacker_grounded,
            defender_grounded,
            chain_mod: 4096,
            effectiveness,
            has_stab,
            has_adaptability,
            is_tera_stab: false, // TODO: Set when Tera is implemented
            attacker_ability,
            defender_ability: state.abilities[defender],
        }
    }
    
    /// Apply a 4096-scale modifier to the chain.
    #[inline]
    pub fn apply_mod(&mut self, modifier: u16) {
        if modifier != 4096 {
            self.chain_mod = super::formula::apply_modifier(self.chain_mod, modifier);
        }
    }
    
    /// Get the attack stat index based on move category.
    /// Returns (attack_index, defense_index) for stats array.
    pub fn get_stat_indices(&self) -> (usize, usize) {
        match self.category {
            MoveCategory::Physical => (1, 2),  // Atk vs Def
            MoveCategory::Special => (3, 4),   // SpA vs SpD
            MoveCategory::Status => (0, 0),    // Shouldn't happen
        }
    }
    
    /// Check if attacker is burned.
    pub fn is_burned(&self) -> bool {
        use crate::state::Status;
        self.state.status[self.attacker].contains(Status::BURN)
    }

    /// Get the attacker's major status condition.
    pub fn attacker_status(&self) -> crate::state::Status {
        self.state.status[self.attacker]
    }
    
    /// Check if a screen is active for the defender's side.
    pub fn has_screen(&self, is_physical: bool) -> bool {
        use crate::state::SideConditions;
        
        let side = if self.defender >= 6 { 1 } else { 0 };
        let conditions = self.state.side_conditions[side];
        
        if conditions.contains(SideConditions::AURORA_VEIL) {
            return true;
        }
        
        if is_physical {
            conditions.contains(SideConditions::REFLECT)
        } else {
            conditions.contains(SideConditions::LIGHT_SCREEN)
        }
    }
}
