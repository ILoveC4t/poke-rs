//! Attack/Defense stat modifying abilities.
//!
//! Called via `OnModifyAttack` or `OnModifyDefense` during stat calculation.

use crate::state::BattleState;
use crate::moves::MoveCategory;

/// Hustle: 1.5x Attack for physical moves (accuracy penalty handled elsewhere)
pub fn hustle(
    _state: &BattleState,
    _attacker: usize,
    category: MoveCategory,
    attack: u16,
) -> u16 {
    if category == MoveCategory::Physical {
        attack * 3 / 2
    } else {
        attack
    }
}

// =============================================================================
// TODO: Future implementations
// =============================================================================

// TODO: Pure Power / Huge Power - 2x Attack
// pub fn pure_power(...) -> u16 { attack * 2 }

// TODO: Guts - 1.5x Attack when statused (+ ignore burn penalty)
// pub fn guts(state: &BattleState, attacker: usize, category: MoveCategory, attack: u16) -> u16

// TODO: Gorilla Tactics - 1.5x Attack (locked into one move)
// pub fn gorilla_tactics(...) -> u16

// TODO: Solar Power - 1.5x Sp.Atk in Sun
// pub fn solar_power(...) -> u16

// TODO: Defeatist - 0.5x Atk/SpA when below 50% HP
// pub fn defeatist(...) -> u16

// TODO: Slow Start - 0.5x Atk/Speed for 5 turns
// pub fn slow_start(...) -> u16

// TODO: Flower Gift - 1.5x Atk in Sun (ally support)
// pub fn flower_gift(...) -> u16

// TODO: Plus/Minus - 1.5x SpA with partner having Plus/Minus
// pub fn plus_minus(...) -> u16

// =============================================================================
// Defense modifiers
// =============================================================================

// TODO: Marvel Scale - 1.5x Def when statused
// pub fn marvel_scale(state: &BattleState, defender: usize, _attacker: usize, category: MoveCategory, defense: u16) -> u16

// TODO: Fur Coat - 2x Defense
// pub fn fur_coat(...) -> u16

// TODO: Grass Pelt - 1.5x Def in Grassy Terrain
// pub fn grass_pelt(...) -> u16
