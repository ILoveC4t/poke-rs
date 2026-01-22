//! Status-related ability implementations.

use crate::state::{BattleState, Status};
use crate::moves::{MoveId, MoveCategory};

// ============================================================================
// Guts: 1.5x Attack if statused, ignore Burn reduction
// ============================================================================

pub fn on_modify_attack_guts(
    state: &BattleState,
    attacker: usize,
    _move_id: MoveId,
    _category: MoveCategory,
    attack: u16,
) -> u16 {
    if state.status[attacker] != Status::NONE {
        // 1.5x boost
        (attack * 3) / 2
    } else {
        attack
    }
}

pub fn on_ignore_status_damage_reduction_guts(
    _state: &BattleState,
    _entity: usize,
    status: Status,
) -> bool {
    // Ignores burn reduction
    status == Status::BURN
}
