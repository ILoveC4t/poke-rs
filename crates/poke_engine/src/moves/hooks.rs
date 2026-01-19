//! Move hook type definitions.
//!
//! This module defines the hook types for move-specific logic that modifies
//! damage calculation (e.g., conditional power boosts like Knock Off, Venoshock).

use crate::state::BattleState;
use crate::moves::Move;
use crate::items::ItemId;

// ============================================================================
// Move Hook Type Definitions  
// ============================================================================

/// Called during base power calculation to check if a move's power should be boosted.
/// Returns true if the condition is met (e.g., target has removable item for Knock Off).
pub type OnBasePowerCondition = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &'static Move,
) -> bool;

/// Called during base power calculation to modify the base power.
/// More flexible than OnBasePowerCondition for moves with variable BP formulas.
pub type OnModifyBasePower = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &'static Move,
    bp: u16,
) -> u16;

// ============================================================================
// MoveHooks Struct
// ============================================================================

/// Hook table for moves with conditional effects.
#[derive(Clone, Copy, Default)]
pub struct MoveHooks {
    /// Condition check for simple multiplier boosts
    pub on_base_power_condition: Option<OnBasePowerCondition>,
    
    /// Multiplier to apply when condition is true (4096 scale, e.g., 6144 = 1.5x)
    pub conditional_multiplier: u16,
    
    /// Custom base power modification function
    pub on_modify_base_power: Option<OnModifyBasePower>,
}

impl MoveHooks {
    /// Empty hooks (default)
    pub const NONE: Self = Self {
        on_base_power_condition: None,
        conditional_multiplier: 4096, // 1x
        on_modify_base_power: None,
    };
}
