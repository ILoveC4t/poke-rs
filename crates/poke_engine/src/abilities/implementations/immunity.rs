//! Type immunity abilities.
//!
//! Called via `OnTypeImmunity` to check if a move is blocked by an ability.
//! Returns true if immune.

use crate::state::BattleState;
use crate::types::Type;

/// Levitate: Immune to Ground moves
pub fn levitate(
    _state: &BattleState,
    _defender: usize,
    move_type: Type,
) -> bool {
    move_type == Type::Ground
}

// =============================================================================
// TODO: Future implementations
// =============================================================================

// TODO: Flash Fire - Fire immunity + boost flag
// pub fn flash_fire(state: &BattleState, defender: usize, move_type: Type) -> bool {
//     if move_type == Type::Fire {
//         // Set flash fire activated flag on defender
//         true
//     } else {
//         false
//     }
// }

// TODO: Volt Absorb - Electric immunity + heal
// pub fn volt_absorb(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Water Absorb - Water immunity + heal
// pub fn water_absorb(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Storm Drain - Water immunity + SpA boost
// pub fn storm_drain(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Lightning Rod - Electric immunity + SpA boost
// pub fn lightning_rod(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Sap Sipper - Grass immunity + Atk boost
// pub fn sap_sipper(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Motor Drive - Electric immunity + Speed boost
// pub fn motor_drive(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Dry Skin - Water immunity + heal, Fire weakness
// pub fn dry_skin(state: &mut BattleState, defender: usize, move_type: Type) -> bool

// TODO: Earth Eater - Ground immunity + heal
// pub fn earth_eater(state: &mut BattleState, defender: usize, move_type: Type) -> bool
