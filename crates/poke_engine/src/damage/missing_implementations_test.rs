#[cfg(test)]
mod tests {
    use crate::state::BattleState;
    use crate::damage::{calculate_damage, Gen9};
    use crate::species::SpeciesId;
    use crate::types::Type;
    use crate::abilities::AbilityId;
    use crate::moves::{MoveId, MoveCategory};
    use crate::items::ItemId;

    #[test]
    fn test_huge_power() {
        let mut state = BattleState::new();
        // Diggersby: Normal/Ground, Huge Power
        state.species[0] = SpeciesId::from_str("diggersby").unwrap();
        state.abilities[0] = AbilityId::Hugepower;
        state.stats[0][1] = 100; // 100 Atk
        state.types[0] = [Type::Normal, Type::Ground];
        state.level[0] = 50;

        state.species[6] = SpeciesId::from_str("rattata").unwrap();
        state.stats[6][2] = 100; // 100 Def
        state.level[6] = 50;

        // Move: Tackle (40 BP)
        let move_id = MoveId::Tackle;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Base: 100 Atk vs 100 Def.
        // Huge Power: 200 Atk.
        // Base Damage ~ (42 * 40 * 200/100 / 50 + 2) = 69.

        assert!(result.min > 50, "Huge Power should double attack (got {})", result.min);
    }

    #[test]
    fn test_strong_jaw() {
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Strongjaw;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        // Bite: 60 BP
        let move_id = MoveId::Bite;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Normal BP 60. Strong Jaw -> 90.
        // Base damage ~ (42 * 90 * 100/100 / 50 + 2) = 77.

        assert!(result.min > 60, "Strong Jaw should boost Bite (got {})", result.min);
    }

    #[test]
    fn test_body_press() {
        let mut state = BattleState::new();
        state.stats[0][1] = 10;  // Low Atk
        state.stats[0][2] = 200; // High Def
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Bodypress;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // If it used Def (200): (42 * 80 * 200/100 / 50) + 2 = 136.

        assert!(result.min > 100, "Body Press should use Defense (got {})", result.min);
    }

    #[test]
    fn test_body_press_huge_power() {
        // Huge Power should NOT boost Body Press
        let mut state = BattleState::new();
        state.stats[0][1] = 10;  // Low Atk
        state.stats[0][2] = 100; // Def
        state.abilities[0] = AbilityId::Hugepower;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Bodypress; // 80 BP

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // If Huge Power applies: Def 200. Damage ~136.
        // If not: Def 100. Damage ~69.

        assert!(result.max < 100, "Huge Power should NOT boost Body Press (got {})", result.max);
    }

    #[test]
    fn test_psyshock() {
        let mut state = BattleState::new();
        state.stats[0][3] = 100; // SpA
        state.level[0] = 50;

        state.stats[6][2] = 50;  // Low Def
        state.stats[6][4] = 200; // High SpD
        state.level[6] = 50;

        let move_id = MoveId::Psyshock; // Special, 80 BP

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // If it used Def (50):  (42 * 80 * 100/50 / 50) = 136.
        // If it used SpD (200): (42 * 80 * 100/200 / 50) = 35.

        assert!(result.min > 100, "Psyshock should target Defense (got {})", result.min);
    }

    #[test]
    fn test_ice_scales() {
        let mut state = BattleState::new();
        state.stats[0][3] = 100; // SpA
        state.level[0] = 50;

        state.stats[6][4] = 100; // SpD
        state.abilities[6] = AbilityId::Icescales;
        state.level[6] = 50;

        let move_id = MoveId::Icebeam; // Special

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Normal damage ~85.
        // Ice Scales (0.5x) ~42.

        assert!(result.max < 60, "Ice Scales should halve special damage (got {})", result.max);
    }

    #[test]
    fn test_foul_play() {
        let mut state = BattleState::new();
        state.stats[0][1] = 10; // Attacker Low Atk
        state.level[0] = 50;

        state.stats[6][1] = 200; // Defender High Atk
        state.stats[6][2] = 100; // Defender Def
        state.level[6] = 50;

        let move_id = MoveId::Foulplay; // 95 BP

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should use Defender Atk (200).
        // (42 * 95 * 200/100 / 50) + 2 = ~161.
        // If uses Attacker Atk (10): ~10.

        assert!(result.min > 100, "Foul Play should use target Attack (got {})", result.min);
    }
}
