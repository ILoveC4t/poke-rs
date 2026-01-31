//! Final damage modifiers (post-random roll).
//!
//! Split into attacker modifiers (OnAttackerFinalMod) and defender modifiers (OnDefenderFinalMod).
//! Order: Attacker mods apply first, then defender mods.

use crate::damage::{apply_modifier, Modifier};
use crate::moves::{Move, MoveCategory, MoveFlags};
use crate::state::BattleState;
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
        apply_modifier(damage, Modifier::DOUBLE) // 2x in 4096-scale
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
        apply_modifier(damage, Modifier::ONE_POINT_FIVE) // 1.5x
    } else {
        damage
    }
}

/// Neuroforce: 1.25x damage on super-effective hits
pub fn neuroforce(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    effectiveness: u8,
    _is_crit: bool,
    damage: u32,
) -> u32 {
    if effectiveness > 4 {
        // 1.25x = 5/4 = 1.25
        apply_modifier(damage, Modifier::new(5120))
    } else {
        damage
    }
}

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
    _move_data: &Move,
    damage: u32,
) -> u32 {
    if state.hp[defender] == state.max_hp[defender] {
        apply_modifier(damage, Modifier::HALF) // 0.5x
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
    _move_data: &Move,
    damage: u32,
) -> u32 {
    if effectiveness > 4 {
        apply_modifier(damage, Modifier::FILTER) // 0.75x
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
    move_data: &Move,
    mut damage: u32,
) -> u32 {
    if move_data.flags.contains(MoveFlags::CONTACT) {
        damage = apply_modifier(damage, Modifier::HALF); // 0.5x
    }
    if move_type == Type::Fire {
        damage = apply_modifier(damage, Modifier::DOUBLE); // 2x
    }
    damage
}

/// Ice Scales: 0.5x special damage
pub fn ice_scales(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    _move_type: Type,
    category: MoveCategory,
    _move_data: &Move,
    damage: u32,
) -> u32 {
    if category == MoveCategory::Special {
        apply_modifier(damage, Modifier::HALF)
    } else {
        damage
    }
}

/// Punk Rock: 0.5x sound-based damage (Defender side)
pub fn punk_rock(
    _state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _effectiveness: u8,
    _move_type: Type,
    _category: MoveCategory,
    move_data: &Move,
    damage: u32,
) -> u32 {
    if move_data.flags.contains(MoveFlags::SOUND) {
        apply_modifier(damage, Modifier::HALF)
    } else {
        damage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{Move, MoveCategory, MoveFlags, MoveTarget};
    use crate::state::BattleState;
    use crate::terrains::TerrainId;
    use crate::types::Type;

    fn make_test_move(flags: MoveFlags) -> Move {
        Move {
            name: "Test Move",
            primary_type: Type::Normal,
            category: MoveCategory::Physical,
            power: 50,
            accuracy: 100,
            pp: 10,
            priority: 0,
            flags,
            terrain: TerrainId::None,
            target: MoveTarget::Normal,
            multihit: (0, 0),
        }
    }

    #[test]
    fn test_fluffy() {
        let state = BattleState::new();
        // 100 damage base
        let base_damage = 100;

        // Case 1: Non-Fire, Contact (0.5x)
        let contact_move = make_test_move(MoveFlags::CONTACT);
        let damage = fluffy(
            &state,
            0,
            1,
            4,
            Type::Normal,
            MoveCategory::Physical,
            &contact_move,
            base_damage,
        );
        assert_eq!(damage, 50, "Fluffy should halve contact damage");

        // Case 2: Fire, Non-Contact (2x)
        let fire_move = make_test_move(MoveFlags::empty());
        let damage = fluffy(
            &state,
            0,
            1,
            4,
            Type::Fire,
            MoveCategory::Special,
            &fire_move,
            base_damage,
        );
        assert_eq!(damage, 200, "Fluffy should double fire damage");

        // Case 3: Fire, Contact (0.5x * 2x = 1x)
        let fire_contact_move = make_test_move(MoveFlags::CONTACT);
        let damage = fluffy(
            &state,
            0,
            1,
            4,
            Type::Fire,
            MoveCategory::Physical,
            &fire_contact_move,
            base_damage,
        );
        assert_eq!(
            damage, 100,
            "Fluffy should be neutral for Fire Contact moves"
        );

        // Case 4: Non-Fire, Non-Contact (1x)
        let non_contact_move = make_test_move(MoveFlags::empty());
        let damage = fluffy(
            &state,
            0,
            1,
            4,
            Type::Normal,
            MoveCategory::Special,
            &non_contact_move,
            base_damage,
        );
        assert_eq!(damage, 100, "Fluffy should not affect other moves");
    }

    #[test]
    fn test_multiscale() {
        let mut state = BattleState::new();
        let defender = 1;
        state.max_hp[defender] = 100;
        let dummy_move = make_test_move(MoveFlags::empty());

        // Case 1: Full HP (0.5x)
        state.hp[defender] = 100;
        let damage = multiscale(
            &state,
            0,
            defender,
            4,
            Type::Normal,
            MoveCategory::Physical,
            &dummy_move,
            100,
        );
        assert_eq!(damage, 50, "Multiscale should halve damage at full HP");

        // Case 2: Not Full HP (1x)
        state.hp[defender] = 99;
        let damage = multiscale(
            &state,
            0,
            defender,
            4,
            Type::Normal,
            MoveCategory::Physical,
            &dummy_move,
            100,
        );
        assert_eq!(
            damage, 100,
            "Multiscale should not affect damage when not at full HP"
        );
    }

    #[test]
    fn test_punk_rock() {
        let state = BattleState::new();
        let base_damage = 100;

        // Case 1: Sound move (0.5x)
        let sound_move = make_test_move(MoveFlags::SOUND);
        let damage = punk_rock(
            &state,
            0,
            1,
            4,
            Type::Normal,
            MoveCategory::Special,
            &sound_move,
            base_damage,
        );
        assert_eq!(damage, 50, "Punk Rock should halve sound damage");

        // Case 2: Non-sound move (1x)
        let normal_move = make_test_move(MoveFlags::empty());
        let damage = punk_rock(
            &state,
            0,
            1,
            4,
            Type::Normal,
            MoveCategory::Physical,
            &normal_move,
            base_damage,
        );
        assert_eq!(damage, 100, "Punk Rock should not affect non-sound moves");
    }
}
