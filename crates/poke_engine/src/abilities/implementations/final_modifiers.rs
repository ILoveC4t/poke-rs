//! Final damage modifiers (post-random roll).
//!
//! Split into attacker modifiers (OnAttackerFinalMod) and defender modifiers (OnDefenderFinalMod).
//! Order: Attacker mods apply first, then defender mods.

use crate::state::BattleState;
use crate::moves::MoveCategory;
use crate::damage::apply_modifier;
use crate::types::Type;

// =============================================================================
// Attacker Final Modifiers
// =============================================================================

/// Tinted Lens: 2x damage on "not very effective" hits
pub fn tinted_lens(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _is_crit: bool,
    damage: u32,
) -> u32 {
    // effectiveness: 4 = 1x, 2 = 0.5x, 1 = 0.25x
    if effectiveness < 4 {
        apply_modifier(damage, 8192) // 2x in 4096-scale
    } else {
        damage
    }
}

/// Sniper: 1.5x damage on critical hits
pub fn sniper(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    is_crit: bool,
    damage: u32,
) -> u32 {
    if is_crit {
        apply_modifier(damage, 6144) // 1.5x
    } else {
        damage
    }
}

// TODO: Neuroforce - 1.25x on super-effective hits
// pub fn neuroforce(...) -> u32

// =============================================================================
// Defender Final Modifiers
// =============================================================================

/// Multiscale / Shadow Shield: 0.5x damage when at full HP
pub fn multiscale(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _effectiveness: u8,
    _move_type: Type,
    _category: MoveCategory,
    _is_contact: bool,
    damage: u32,
) -> u32 {
    if state.hp[defender] == state.max_hp[defender] {
        apply_modifier(damage, 2048) // 0.5x
    } else {
        damage
    }
}

/// Filter / Solid Rock / Prism Armor: 0.75x on super-effective hits
pub fn filter(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _move_type: Type,
    _category: MoveCategory,
    _is_contact: bool,
    damage: u32,
) -> u32 {
    if effectiveness > 4 {
        apply_modifier(damage, 3072) // 0.75x
    } else {
        damage
    }
}

/// Fluffy: 0.5x contact damage, 2x Fire damage
pub fn fluffy(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    move_type: Type,
    _category: MoveCategory,
    is_contact: bool,
    mut damage: u32,
) -> u32 {
    if is_contact {
        damage = apply_modifier(damage, 2048); // 0.5x
    }
    if move_type == Type::Fire {
        damage = apply_modifier(damage, 8192); // 2x
    }
    damage
}

// TODO: Ice Scales - 0.5x special damage
// TODO: Punk Rock - 0.5x sound-based damage

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::BattleState;
    use crate::types::Type;
    use crate::moves::MoveCategory;

    #[test]
    fn test_fluffy() {
        let state = BattleState::new();
        // 100 damage base
        let base_damage = 100;

        // Case 1: Non-Fire, Contact (0.5x)
        let damage = fluffy(
            &state, 0, 1, 4,
            Type::Normal, MoveCategory::Physical, true, // is_contact = true
            base_damage
        );
        assert_eq!(damage, 50, "Fluffy should halve contact damage");

        // Case 2: Fire, Non-Contact (2x)
        let damage = fluffy(
            &state, 0, 1, 4,
            Type::Fire, MoveCategory::Special, false, // is_contact = false
            base_damage
        );
        assert_eq!(damage, 200, "Fluffy should double fire damage");

        // Case 3: Fire, Contact (0.5x * 2x = 1x)
        let damage = fluffy(
            &state, 0, 1, 4,
            Type::Fire, MoveCategory::Physical, true, // is_contact = true
            base_damage
        );
        assert_eq!(damage, 100, "Fluffy should be neutral for Fire Contact moves");

        // Case 4: Non-Fire, Non-Contact (1x)
        let damage = fluffy(
            &state, 0, 1, 4,
            Type::Normal, MoveCategory::Special, false,
            base_damage
        );
        assert_eq!(damage, 100, "Fluffy should not affect other moves");
    }

    #[test]
    fn test_multiscale() {
        let mut state = BattleState::new();
        let defender = 1;
        state.max_hp[defender] = 100;

        // Case 1: Full HP (0.5x)
        state.hp[defender] = 100;
        let damage = multiscale(
            &state, 0, defender, 4,
            Type::Normal, MoveCategory::Physical, false,
            100
        );
        assert_eq!(damage, 50, "Multiscale should halve damage at full HP");

        // Case 2: Not Full HP (1x)
        state.hp[defender] = 99;
        let damage = multiscale(
            &state, 0, defender, 4,
            Type::Normal, MoveCategory::Physical, false,
            100
        );
        assert_eq!(damage, 100, "Multiscale should not affect damage when not at full HP");
    }
}
