//! Type immunity abilities.
//!
//! Called via `OnTypeImmunity` to check if a move is blocked by an ability.
//! Returns true if immune.

use crate::state::BattleState;
use crate::types::Type;

/// Levitate: Immune to Ground moves (if not grounded)
pub fn levitate(
    state: &BattleState,
    defender: usize,
    move_type: Type,
) -> bool {
    // Note: is_grounded(defender) returns false if Levitate is active AND not negated.
    // However, if Iron Ball / Gravity is active, is_grounded returns true.
    // So if grounded, Levitate fails.
    // If not grounded, Levitate applies (and grants immunity to Ground).
    move_type == Type::Ground && !state.is_grounded(defender)
}

/// Flash Fire: Immune to Fire moves
pub fn flash_fire(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Fire
}

/// Volt Absorb: Immune to Electric moves
pub fn volt_absorb(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Electric
}

/// Water Absorb: Immune to Water moves
pub fn water_absorb(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Water
}

/// Storm Drain: Immune to Water moves
pub fn storm_drain(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Water
}

/// Lightning Rod: Immune to Electric moves
pub fn lightning_rod(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Electric
}

/// Sap Sipper: Immune to Grass moves
pub fn sap_sipper(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Grass
}

/// Motor Drive: Immune to Electric moves
pub fn motor_drive(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Electric
}

/// Dry Skin: Immune to Water moves
pub fn dry_skin(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Water
}

/// Earth Eater: Immune to Ground moves
pub fn earth_eater(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Ground
}
