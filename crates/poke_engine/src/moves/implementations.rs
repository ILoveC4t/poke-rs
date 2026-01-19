//! Move hook implementations.
//!
//! Contains the condition check functions for moves with conditional power boosts.

use crate::state::{BattleState, Status};
use crate::moves::Move;

// ============================================================================
// Knock Off: 1.5x if target has removable item
// ============================================================================

pub fn knockoff_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    let item = state.items[defender];
    if item == crate::items::ItemId::None {
        return false;
    }
    let item_data = item.data();
    !item_data.is_unremovable
}

// ============================================================================
// Venoshock: 2x if target is poisoned
// ============================================================================

pub fn venoshock_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    state.status[defender].intersects(Status::POISON | Status::TOXIC)
}

// ============================================================================
// Hex: 2x if target has any major status condition
// ============================================================================

pub fn hex_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    state.status[defender] != Status::NONE
}

// ============================================================================
// Brine: 2x if target is at or below 50% HP
// ============================================================================

pub fn brine_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    let hp = state.hp[defender];
    let max_hp = state.max_hp[defender];
    hp * 2 <= max_hp
}
