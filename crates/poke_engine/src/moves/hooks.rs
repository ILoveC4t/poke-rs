//! Move hook type definitions.
//!
//! This module defines the hook types for move-specific logic that modifies
//! damage calculation (e.g., conditional power boosts like Knock Off, Venoshock).

use crate::state::BattleState;
use crate::moves::Move;
use crate::types::Type;

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

/// Called during DamageContext creation to modify the move's type.
/// E.g. Weather Ball, Revelation Dance, Aura Wheel, Terrain Pulse.
pub type OnModifyType = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &'static Move,
    base_type: Type,
) -> Type;

/// Called during DamageContext creation to modify the move's effectiveness.
/// E.g. Freeze-Dry, Flying Press, Thousand Arrows.
/// type_chart provides lookup for (ModifyType, TargetType) -> Effectiveness
pub type OnModifyEffectiveness = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &'static Move,
    effectiveness: u8,
    type_chart: &dyn Fn(Type, Type) -> u8,
) -> u8;

/// Called to check if status damage reduction should be ignored (e.g. Facade ignoring Burn attack drop)
pub type OnIgnoreStatusDamageReduction = fn(
    state: &BattleState,
    attacker: usize,
    status: crate::state::Status,
) -> bool;

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
    
    /// Type modification function
    pub on_modify_type: Option<OnModifyType>,
    
    /// Effectiveness modification function
    pub on_modify_effectiveness: Option<OnModifyEffectiveness>,

    /// Ignore status damage reduction function
    pub on_ignore_status_damage_reduction: Option<OnIgnoreStatusDamageReduction>,
}

impl MoveHooks {
    /// Empty hooks (default)
    pub const NONE: Self = Self {
        on_base_power_condition: None,
        conditional_multiplier: 4096, // 1x
        on_modify_base_power: None,
        on_modify_type: None,
        on_modify_effectiveness: None,
        on_ignore_status_damage_reduction: None,
    };
}
