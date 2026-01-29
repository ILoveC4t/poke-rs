use crate::moves::{MoveCategory, MoveFlags, MoveId};
use crate::state::BattleState;
use crate::types::Type;

pub fn prankster(_state: &BattleState, _idx: usize, move_id: MoveId, base: i8) -> i8 {
    if move_id.data().category == MoveCategory::Status {
        base + 1
    } else {
        base
    }
}

pub fn gale_wings(state: &BattleState, idx: usize, move_id: MoveId, base: i8) -> i8 {
    // Gen 7+: Only at full HP
    if state.hp[idx] == state.max_hp[idx] && move_id.data().primary_type == Type::Flying {
        base + 1
    } else {
        base
    }
}

pub fn triage(_state: &BattleState, _idx: usize, move_id: MoveId, base: i8) -> i8 {
    // Check if move has healing flag
    // We assume MoveFlags::HEAL exists. If not generated, this will fail and need adjustment.
    // Given standard showdown data, 'heal' is a flag.
    if move_id.data().flags.contains(MoveFlags::HEAL) {
        base + 3
    } else {
        base
    }
}
