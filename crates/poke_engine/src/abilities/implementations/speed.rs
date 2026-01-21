//! Speed-modifying abilities.
//!
//! Called via `OnModifySpeed` during effective speed calculation.

use crate::state::BattleState;
use crate::damage::generations::{Weather, Terrain};

/// Chlorophyll: 2x Speed in Sun
pub fn chlorophyll(state: &BattleState, _entity: usize, speed: u16) -> u16 {
    let weather = Weather::from_u8(state.weather);
    if matches!(weather, Weather::Sun | Weather::HarshSun) {
        speed.saturating_mul(2)
    } else {
        speed
    }
}

/// Swift Swim: 2x Speed in Rain
pub fn swift_swim(state: &BattleState, _entity: usize, speed: u16) -> u16 {
    let weather = Weather::from_u8(state.weather);
    if matches!(weather, Weather::Rain | Weather::HeavyRain) {
        speed.saturating_mul(2)
    } else {
        speed
    }
}

/// Sand Rush: 2x Speed in Sandstorm
pub fn sand_rush(state: &BattleState, _entity: usize, speed: u16) -> u16 {
    let weather = Weather::from_u8(state.weather);
    if weather == Weather::Sand {
        speed.saturating_mul(2)
    } else {
        speed
    }
}

/// Slush Rush: 2x Speed in Hail/Snow
pub fn slush_rush(state: &BattleState, _entity: usize, speed: u16) -> u16 {
    let weather = Weather::from_u8(state.weather);
    if matches!(weather, Weather::Hail | Weather::Snow) {
        speed.saturating_mul(2)
    } else {
        speed
    }
}

/// Surge Surfer: 2x Speed in Electric Terrain
pub fn surge_surfer(state: &BattleState, _entity: usize, speed: u16) -> u16 {
    let terrain = Terrain::from_u8(state.terrain);
    if terrain == Terrain::Electric {
        speed.saturating_mul(2)
    } else {
        speed
    }
}
