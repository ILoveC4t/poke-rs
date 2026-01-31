use crate::damage::Modifier;
use crate::moves::{MoveId, MoveTarget};
use crate::state::BattleState;

pub fn parental_bond(
    state: &BattleState,
    _attacker: usize,
    _defender: usize,
    move_id: MoveId,
) -> Option<Vec<Modifier>> {
    let move_data = move_id.data();

    // Does not apply to moves that are already multi-hit
    // Standard moves have (0, 0) or (1, 1). Multi-hit have min > 1.
    if move_data.multihit.0 > 1 {
        return None;
    }

    // Does not apply to status moves (though likely filtered before this)
    if move_data.category == crate::moves::MoveCategory::Status {
        return None;
    }

    // Does not apply to spread moves (moves that target multiple foes/all)
    match move_data.target {
        MoveTarget::AllAdjacentFoes | MoveTarget::AllAdjacent | MoveTarget::All => return None,
        _ => {}
    }

    // One-Hit KO moves are not affected
    if move_data.flags.contains(crate::moves::MoveFlags::OHKO) {
        return None;
    }

    // Fixed damage moves: Second hit deals same damage (1.0x logic)
    // We signal "Second Hit" by returning a modifier.
    // For fixed damage moves, the damage calculation logic must ignore the modifier value
    // and just strictly apply the fixed damage again.
    // However, if we return Modifier::ONE here, `calculate_damage` might think it's 1.0x BP.
    // Standard moves (Gen 7+): 0.25x BP.
    // Standard moves (Gen 6): 0.5x BP.
    
    // Check if it is a fixed damage move?
    // The runner handles fixed damage separately.
    // But this hook is just "What are the modifiers for extra hits?"
    
    let gen = state.generation;
    let modifier = if gen <= 6 {
        Modifier::HALF // 0.5x
    } else {
        Modifier::new(1024) // 0.25x
    };

    Some(vec![modifier])
}
