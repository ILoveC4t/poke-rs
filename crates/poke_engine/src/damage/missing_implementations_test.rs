#[cfg(test)]
mod tests {
    use crate::state::{BattleState, Status};
    use crate::damage::{calculate_damage, Gen9};
    use crate::species::SpeciesId;
    use crate::types::Type;
    use crate::abilities::AbilityId;
    use crate::moves::{MoveId, MoveCategory};
    use crate::items::ItemId;
    use crate::entities::PokemonConfig;

    #[test]
    fn test_huge_power() {
        let mut state = BattleState::new();
        // Diggersby: Normal/Ground, Huge Power
        state.species[0] = SpeciesId::from_str("diggersby").unwrap();
        state.abilities[0] = AbilityId::Hugepower;
        state.stats[0][1] = 100; // 100 Atk
        state.types[0] = [Type::Normal, Type::Ground];
        state.level[0] = 50;

        // Fallback to Rattata's National Dex ID if name lookup fails
        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.stats[6][2] = 100; // 100 Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Level 50. Atk 100.
        // Huge Power -> 200 Atk.
        // Base Damage = floor(22 * 40 * 200/100 / 50) + 2 = 37.
        // STAB (1.5x) -> 55.
        // Min Roll (0.85) -> 46.

        assert!(result.min >= 46, "Huge Power should double attack (got {})", result.min);
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
        // Base damage = floor(22 * 90 * 1 / 50) + 2 = 41.
        // Min Roll (0.85) -> 34.

        assert!(result.min >= 34, "Strong Jaw should boost Bite (got {})", result.min);
    }

    #[test]
    fn test_body_press() {
        let mut state = BattleState::new();
        state.stats[0][1] = 10;  // Low Atk
        state.stats[0][2] = 200; // High Def
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Bodypress; // 80 BP

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // If it used Def (200):
        // Base = floor(22 * 80 * 200/100 / 50) + 2 = 72.
        // Min Roll (0.85) -> 61.

        assert!(result.min >= 60, "Body Press should use Defense (got {})", result.min);
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

        // Def 100.
        // Base = floor(22 * 80 * 1 / 50) + 2 = 37.
        // Min Roll -> 31.
        // If Huge Power applied (2x), damage would be ~61.

        assert!(result.max < 50, "Huge Power should NOT boost Body Press (got {})", result.max);
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

        // If it used Def (50):
        // Base = floor(22 * 80 * 100/50 / 50) + 2 = 72.
        // Min Roll -> 61.

        assert!(result.min >= 60, "Psyshock should target Defense (got {})", result.min);
    }

    #[test]
    fn test_ice_scales() {
        let mut state = BattleState::new();
        state.stats[0][3] = 100; // SpA
        state.level[0] = 50;

        state.stats[6][4] = 100; // SpD
        state.abilities[6] = AbilityId::Icescales;
        state.level[6] = 50;

        let move_id = MoveId::Icebeam; // Special, 90 BP

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Base = floor(22 * 90 * 1 / 50) + 2 = 41.
        // Roll 100% -> 41.
        // Ice Scales (0.5x) -> 20.

        assert!(result.max <= 25, "Ice Scales should halve special damage (got {})", result.max);
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
        // Base = floor(22 * 95 * 2 / 50) + 2 = 85.
        // Min Roll -> 72.

        assert!(result.min >= 70, "Foul Play should use target Attack (got {})", result.min);
    }

    #[test]
    fn test_neuroforce() {
        let mut state = BattleState::new();
        state.stats[0][3] = 100; // SpA
        state.level[0] = 50;
        state.abilities[0] = AbilityId::Neuroforce;

        state.stats[6][4] = 100; // SpD
        state.level[6] = 50;
        state.types[6] = [Type::Grass, Type::Grass]; // Weak to Fire

        let move_id = MoveId::Flamethrower; // Fire, Super Effective

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Neuroforce
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Neuroforce on super-effective hits
        assert!(result_with.min > result_without.min, 
            "Neuroforce should boost super effective damage (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_guts() {
        // Test that Guts affects damage when statused
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Guts;
        state.status[0] = Status::POISON; // Use poison instead of burn (burn halves physical attack)
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result_with_guts = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Guts/status
        state.abilities[0] = AbilityId::Noability; // No ability
        state.status[0] = Status::NONE;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // With Guts should do more damage than without
        assert!(result_with_guts.min > result_without.min, 
            "Guts should boost Attack when poisoned (with: {}, without: {})", 
            result_with_guts.min, result_without.min);
    }

    #[test]
    fn test_guts_no_body_press() {
        // Guts should NOT boost Body Press
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.stats[0][2] = 100; // Def
        state.abilities[0] = AbilityId::Guts;
        state.status[0] = Status::BURN;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Bodypress;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Body Press uses Defense, Guts shouldn't apply
        // Expected: ~69 damage (same as without Guts)

        assert!(result.max < 85, "Guts should NOT boost Body Press (got {})", result.max);
    }

    #[test]
    fn test_gorilla_tactics() {
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Gorillatactics;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical, 40 BP

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Gorilla Tactics
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Gorilla Tactics
        assert!(result_with.min > result_without.min, 
            "Gorilla Tactics should boost physical Attack (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_defeatist_low_hp() {
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Defeatist;
        state.hp[0] = 50;
        state.max_hp[0] = 100;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // HP at 50%: Defeatist applies (0.5x)
        // 50 Atk vs 100 Def -> ~18 damage

        assert!(result.max < 25, "Defeatist should halve Attack at 50% HP (got {})", result.max);
    }

    #[test]
    fn test_defeatist_high_hp() {
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Defeatist;
        state.hp[0] = 51;
        state.max_hp[0] = 100;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result_high_hp = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // At 50% HP
        state.hp[0] = 50;
        let result_low_hp = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // High HP should do more damage than low HP
        assert!(result_high_hp.min > result_low_hp.min, 
            "Defeatist should NOT apply above 50% HP (high: {}, low: {})", 
            result_high_hp.min, result_low_hp.min);
    }

    #[test]
    fn test_mega_launcher() {
        let mut state = BattleState::new();
        state.stats[0][3] = 100; // SpA
        state.abilities[0] = AbilityId::Megalauncher;
        state.level[0] = 50;

        state.stats[6][4] = 100; // SpD
        state.level[6] = 50;

        let move_id = MoveId::Aurasphere; // Pulse move, 80 BP

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Mega Launcher
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Mega Launcher
        assert!(result_with.min > result_without.min, 
            "Mega Launcher should boost pulse moves (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_fur_coat() {
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.abilities[6] = AbilityId::Furcoat;
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Normal: 100 Atk vs 100 Def -> ~35 damage
        // Fur Coat (2x Def): 100 Atk vs 200 Def -> ~18 damage

        assert!(result.max < 25, "Fur Coat should double Defense (got {})", result.max);
    }

    #[test]
    fn test_protosynthesis_sun() {
        use crate::damage::generations::Weather;
        
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk
        state.stats[0][2] = 50;  // Lower Def
        state.stats[0][3] = 50;  // Lower SpA
        state.stats[0][4] = 50;  // Lower SpD
        state.stats[0][5] = 50;  // Lower Spe
        state.abilities[0] = AbilityId::Protosynthesis;
        state.weather = Weather::Sun as u8;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Protosynthesis or sun
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Protosynthesis in Sun
        assert!(result_with.min > result_without.min, 
            "Protosynthesis should boost highest stat in Sun (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_protosynthesis_body_press() {
        use crate::damage::generations::Weather;
        
        let mut state = BattleState::new();
        state.stats[0][1] = 50;  // Lower Atk
        state.stats[0][2] = 100; // High Def (highest stat)
        state.stats[0][3] = 50;  // Lower SpA
        state.stats[0][4] = 50;  // Lower SpD
        state.stats[0][5] = 50;  // Lower Spe
        state.abilities[0] = AbilityId::Protosynthesis;
        state.weather = Weather::Sun as u8;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Bodypress; // Uses Defense

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Protosynthesis
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Protosynthesis
        assert!(result_with.min > result_without.min, 
            "Protosynthesis should boost Defense for Body Press (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_quark_drive_electric_terrain() {
        use crate::damage::generations::Terrain;
        
        let mut state = BattleState::new();
        state.stats[0][1] = 100; // Atk (highest)
        state.stats[0][2] = 50;  // Lower Def
        state.stats[0][3] = 50;  // Lower SpA
        state.stats[0][4] = 50;  // Lower SpD
        state.stats[0][5] = 50;  // Lower Spe
        state.abilities[0] = AbilityId::Quarkdrive;
        state.terrain = Terrain::Electric as u8;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result_with = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Without Quark Drive
        state.abilities[0] = AbilityId::Noability;
        let result_without = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should do more damage with Quark Drive
        assert!(result_with.min > result_without.min, 
            "Quark Drive should boost highest stat in Electric Terrain (with: {}, without: {})", 
            result_with.min, result_without.min);
    }

    #[test]
    fn test_quark_drive_speed() {
        use crate::damage::generations::Terrain;
        
        let mut state = BattleState::new();
        state.stats[0][1] = 50;  // Lower Atk
        state.stats[0][2] = 50;  // Lower Def
        state.stats[0][3] = 50;  // Lower SpA
        state.stats[0][4] = 50;  // Lower SpD
        state.stats[0][5] = 100; // High Spe (highest stat)
        state.abilities[0] = AbilityId::Quarkdrive;
        state.terrain = Terrain::Electric as u8;
        state.level[0] = 50;

        state.stats[6][2] = 100; // Def
        state.level[6] = 50;

        let move_id = MoveId::Tackle; // Physical

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Highest stat is Spe (100), but Speed boost doesn't affect damage
        // 50 Atk vs 100 Def -> ~18 damage (no boost)

        assert!(result.max < 25, "Quark Drive Speed boost should not affect damage (got {})", result.max);
    }
}
