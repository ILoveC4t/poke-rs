use crate::state::BattleState;

/// Heavy Metal: Doubles the Pokemon's weight.
pub fn heavy_metal(_state: &BattleState, _entity: usize, weight: u16) -> u16 {
    weight.saturating_mul(2)
}

/// Light Metal: Halves the Pokemon's weight.
pub fn light_metal(_state: &BattleState, _entity: usize, weight: u16) -> u16 {
    weight / 2
}
