#[cfg(test)]
mod tests {
    use crate::damage::{calculate_damage, Gen9};
    use crate::entities::PokemonConfig;
    use crate::moves::MoveId;
    use crate::state::BattleState;
    use crate::abilities::AbilityId;

    #[test]
    fn test_scrappy_normal_vs_ghost() {
        let mut state = BattleState::new();
        // Attacker: Kangaskhan with Scrappy
        let mut attacker = PokemonConfig::from_str("kangaskhan").unwrap().level(50);
        attacker.ability = Some(AbilityId::Scrappy);
        attacker.spawn(&mut state, 0, 0);

        // Defender: Gengar (Ghost/Poison)
        PokemonConfig::from_str("gengar").unwrap().level(50).spawn(&mut state, 1, 0);

        // Move: Pound (Normal)
        let move_id = MoveId::Pound;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false); // 6 is defender index (player 1 slot 0)

        // Should hit (1x effectiveness forced)
        // Without Scrappy, Normal vs Ghost is 0. With Scrappy, it is Neutral (4).
        // Since Gengar is also Poison, Normal vs Poison is Neutral.
        // So overall should be Neutral (4).
        assert!(result.max > 0, "Damage should be > 0 with Scrappy");
        assert_eq!(result.effectiveness, 4, "Effectiveness should be 1x (4) with Scrappy, got {}", result.effectiveness);
    }

    #[test]
    fn test_scrappy_fighting_vs_ghost() {
        let mut state = BattleState::new();
        // Attacker: Machamp with Scrappy
        let mut attacker = PokemonConfig::from_str("machamp").unwrap().level(50);
        attacker.ability = Some(AbilityId::Scrappy);
        attacker.spawn(&mut state, 0, 0);

        // Defender: Gengar (Ghost/Poison)
        PokemonConfig::from_str("gengar").unwrap().level(50).spawn(&mut state, 1, 0);

        // Move: Karate Chop (Fighting)
        let move_id = MoveId::Karatechop;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Fighting vs Ghost is 0. With Scrappy, it is Neutral.
        // Fighting vs Poison is Resistant (0.5x).
        // Total: 0.5x (2).
        assert!(result.max > 0, "Damage should be > 0 with Scrappy");
        assert_eq!(result.effectiveness, 2, "Effectiveness should be 0.5x (2) vs Ghost/Poison with Scrappy, got {}", result.effectiveness);
    }

    #[test]
    fn test_mindseye_normal_vs_ghost() {
        let mut state = BattleState::new();
        // Attacker: Ursaluna-Bloodmoon with Mind's Eye
        // Using "Ursaluna" as base, verifying ability variant name
        let mut attacker = PokemonConfig::from_str("ursaluna").unwrap().level(50);
        attacker.ability = Some(AbilityId::Mindseye);
        attacker.spawn(&mut state, 0, 0);

        // Defender: Misdreavus (Pure Ghost)
        PokemonConfig::from_str("misdreavus").unwrap().level(50).spawn(&mut state, 1, 0);

        // Move: Tackle (Normal)
        let move_id = MoveId::Tackle;

        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Normal vs Ghost is 0. With Mind's Eye, it is Neutral.
        // Total: 1x (4).
        assert!(result.max > 0, "Damage should be > 0 with Mind's Eye");
        assert_eq!(result.effectiveness, 4, "Effectiveness should be 1x (4) with Mind's Eye, got {}", result.effectiveness);
    }
}
