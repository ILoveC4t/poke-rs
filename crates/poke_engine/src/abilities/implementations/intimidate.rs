use crate::state::BattleState;

pub fn intimidate(state: &mut BattleState, idx: usize) {
    // Determine opponent index (target active opponent)
    // Works for Singles. For Doubles, needs adjacency check.
    let side = if idx < 6 { 0 } else { 1 };
    let opponent_side = 1 - side;
    let opponent_idx = state.active[opponent_side] as usize;

    // TODO(TASK-E-PHASE2): Check blocking abilities (Clear Body, Hyper Cutter, White Smoke, Full Metal Body)
    // TODO(TASK-E-PHASE2): Check blocking items (Clear Amulet)
    // TODO(TASK-E-PHASE2): Check Mist side condition
    // TODO(TASK-E-PHASE2): Check Substitute
    // TODO(TASK-E-PHASE2): Check Mirror Armor (reflects stat drop)

    // -1 Attack
    let current_boost = state.boosts[opponent_idx][0]; // 0 is Atk in boosts array
    let new_boost = (current_boost - 1).max(-6);
    state.boosts[opponent_idx][0] = new_boost;
}
