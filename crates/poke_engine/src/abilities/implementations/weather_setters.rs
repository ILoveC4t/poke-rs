use crate::state::BattleState;
use crate::abilities::hooks::AbilityHooks;
use crate::abilities::weather::{Weather, Terrain};

pub fn drizzle(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_weather(state, Weather::Rain, 5);
}

pub fn drought(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_weather(state, Weather::Sun, 5);
}

pub fn sand_stream(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_weather(state, Weather::Sand, 5);
}

pub fn snow_warning(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_weather(state, Weather::Snow, 5);
}

pub fn electric_surge(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_terrain(state, Terrain::Electric, 5);
}

pub fn grassy_surge(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_terrain(state, Terrain::Grassy, 5);
}

pub fn misty_surge(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_terrain(state, Terrain::Misty, 5);
}

pub fn psychic_surge(state: &mut BattleState, _idx: usize) {
    AbilityHooks::set_terrain(state, Terrain::Psychic, 5);
}
