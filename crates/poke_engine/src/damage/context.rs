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
use super::Modifier;

/// Context for a single damage calculation.
///
/// This struct is passed through the modifier pipeline, accumulating
/// multipliers and flags as each stage processes it.
#[derive(Clone, Copy)]
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
        let mut move_type = move_data.primary_type;

        let attacker_ability = state.abilities[attacker];

        // Ability Type Changes (Aerilate, Pixilate, Refrigerate, Galvanize, Normalize, Liquid Voice)
        // Note: These override the base type for effectiveness and STAB calculation.
        match attacker_ability {
            AbilityId::Aerilate if move_type == Type::Normal => move_type = Type::Flying,
            AbilityId::Pixilate if move_type == Type::Normal => move_type = Type::Fairy,
            AbilityId::Refrigerate if move_type == Type::Normal => move_type = Type::Ice,
            AbilityId::Galvanize if move_type == Type::Normal => move_type = Type::Electric,
            AbilityId::Normalize => move_type = Type::Normal,
            AbilityId::Liquidvoice if move_data.flags.contains(crate::moves::MoveFlags::SOUND) => {
                move_type = Type::Water;
            }
            _ => {}
        }
        
        // Check STAB
        let has_stab = move_type == attacker_types[0] || move_type == attacker_types[1];
        
        // Check Adaptability
        let has_adaptability = attacker_ability == AbilityId::Adaptability;
        
        // Check if grounded
        let attacker_grounded = state.is_grounded(attacker);
        let defender_grounded = state.is_grounded(defender);
        
        // Calculate type effectiveness with immunity overrides
        let mut effectiveness = super::effectiveness::calculate_effectiveness(
            move_type,
            defender,
            state,
            attacker_ability,
            G::GEN,
            |mv_type, def_type, _| gen.type_effectiveness(mv_type, def_type, None),
        );
        
        // Mold Breaker check
        let has_mold_breaker = matches!(attacker_ability, AbilityId::Moldbreaker | AbilityId::Teravolt | AbilityId::Turboblaze);
        let raw_defender_ability = state.abilities[defender];

        let defender_ability = if has_mold_breaker && raw_defender_ability.is_breakable() {
            AbilityId::Noability
        } else {
            raw_defender_ability
        };

        // Check for ability-granted immunity (Levitate, Flash Fire, etc.)
        if effectiveness > 0 {
            effectiveness = Self::check_ability_immunity(state, attacker, defender, defender_ability, move_type, effectiveness);
        }
        
        // Determine category (respect Physical/Special split)
        let category = if move_data.category == MoveCategory::Status {
            MoveCategory::Status
        } else if gen.uses_physical_special_split() {
            move_data.category
        } else {
            match move_type {
                Type::Normal | Type::Fighting | Type::Flying | Type::Ground |
                Type::Rock | Type::Bug | Type::Ghost | Type::Poison | Type::Steel => MoveCategory::Physical,
                _ => MoveCategory::Special,
            }
        };

        Self {
            gen,
            state,
            attacker,
            defender,
            move_id,
            move_data,
            base_power: move_data.power,
            category,
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
            defender_ability,
        }
    }
    
    /// Apply a 4096-scale modifier to the chain.
    #[inline]
    pub fn apply_mod(&mut self, modifier: Modifier) {
        if modifier != Modifier::ONE {
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
        let side = if self.defender >= 6 { 1 } else { 0 };
        let conditions = self.state.side_conditions[side];
        
        if conditions.aurora_veil_turns > 0 {
            return true;
        }
        
        if is_physical {
            conditions.reflect_turns > 0
        } else {
            conditions.light_screen_turns > 0
        }
    }
    
    /// Check if an ability grants immunity to a move type.
    /// Returns 0 (immune) if the ability blocks the move, otherwise returns original effectiveness.
    fn check_ability_immunity(
        state: &BattleState,
        _attacker: usize,
        defender: usize,
        ability: AbilityId,
        move_type: Type,
        effectiveness: u8,
    ) -> u8 {
        use crate::abilities::ABILITY_REGISTRY;
        
        // Note: Mold Breaker check is handled in DamageContext::new by passing suppressed ability
        
        if let Some(Some(hooks)) = ABILITY_REGISTRY.get(ability as usize) {
            if let Some(hook) = hooks.on_type_immunity {
                if hook(state, defender, move_type) {
                    return 0; // Immune
                }
            }
        }
        effectiveness
    }
}
