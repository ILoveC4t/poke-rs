use crate::state::BattleState;
use crate::moves::MoveId;
use super::weather::{Weather, Terrain};

/// Called when a Pokemon switches in (after hazards)
pub type OnSwitchIn = fn(state: &mut BattleState, switched_idx: usize);

/// Called during turn ordering to modify move priority
pub type OnModifyPriority = fn(state: &BattleState, attacker: usize, move_id: MoveId, base_priority: i8) -> i8;

/// Called immediately before a move is executed
pub type OnBeforeMove = fn(state: &mut BattleState, attacker: usize, move_id: MoveId);

/// Called during damage calculation to modify damage
pub type OnModifyDamage = fn(state: &BattleState, attacker: usize, defender: usize, damage: u16) -> u16;

/// Called after damage has been dealt
pub type OnAfterDamage = fn(state: &mut BattleState, attacker: usize, defender: usize, damage: u16);

/// Called when a stat boost is applied to modify the stage change
pub type OnStatChange = fn(change: i8) -> i8;

#[derive(Clone, Copy, Default)]
pub struct AbilityHooks {
    pub on_switch_in: Option<OnSwitchIn>,
    pub on_modify_priority: Option<OnModifyPriority>,
    pub on_before_move: Option<OnBeforeMove>,
    pub on_modify_damage: Option<OnModifyDamage>,
    pub on_after_damage: Option<OnAfterDamage>,
    pub on_stat_change: Option<OnStatChange>,
}

impl AbilityHooks {
    /// Empty hooks (default)
    pub const NONE: Self = Self {
        on_switch_in: None,
        on_modify_priority: None,
        on_before_move: None,
        on_modify_damage: None,
        on_after_damage: None,
        on_stat_change: None,
    };

    /// Helper to set weather
    pub fn set_weather(state: &mut BattleState, weather: Weather, turns: u8) {
        state.weather = weather as u8;
        state.weather_turns = turns;
    }

    /// Helper to set terrain
    pub fn set_terrain(state: &mut BattleState, terrain: Terrain, turns: u8) {
        state.terrain = terrain as u8;
        state.terrain_turns = turns;
    }
}
