#[cfg(test)]
mod tests {
    use crate::damage::{calculate_damage, Gen9};
    use crate::entities::PokemonConfig;
    use crate::moves::MoveId;
    use crate::state::BattleState;
    use crate::damage::generations::Weather;

    #[test]
    fn test_struggle_vs_ghost() {
        let mut state = BattleState::new();
        // Attacker: Pikachu
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        // Defender: Gengar (Ghost/Poison)
        PokemonConfig::from_str("gengar").unwrap().level(50).spawn(&mut state, 1, 0);

        let move_id = MoveId::Struggle;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false); // 6 is defender index

        // Should hit (1x effectiveness forced)
        assert!(result.max > 0);
        assert_eq!(result.effectiveness, 4); // 1x
    }

    #[test]
    fn test_weather_ball_sun() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("venusaur").unwrap().level(50).spawn(&mut state, 1, 0);

        // Set Sun
        state.weather = Weather::Sun as u8;

        let move_id = MoveId::Weatherball;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be Fire type (Super Effective vs Grass/Poison - 2x)
        assert_eq!(result.effectiveness, 8); // 2x
        // 100 BP * 1.5 (Sun boost) = 150
        assert_eq!(result.final_base_power, 150);
    }

    #[test]
    fn test_weather_ball_rain() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("charizard").unwrap().level(50).spawn(&mut state, 1, 0);

        // Set Rain
        state.weather = Weather::Rain as u8;

        let move_id = MoveId::Weatherball;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be Water type (Super Effective vs Fire/Flying - 2x)
        assert_eq!(result.effectiveness, 8); // 2x
        // 100 BP * 1.5 (Rain boost) = 150
        assert_eq!(result.final_base_power, 150);
    }

    #[test]
    fn test_weather_ball_sand() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("charizard").unwrap().level(50).spawn(&mut state, 1, 0);

        // Set Sand
        state.weather = Weather::Sand as u8;

        let move_id = MoveId::Weatherball;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be Rock type (Super Effective vs Fire/Flying - 4x)
        assert_eq!(result.effectiveness, 16); // 4x
        // 100 BP (No Sand boost for Rock)
        assert_eq!(result.final_base_power, 100);
    }

    #[test]
    fn test_weather_ball_snow() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("garchomp").unwrap().level(50).spawn(&mut state, 1, 0); // Dragon/Ground

        // Set Snow
        state.weather = Weather::Snow as u8;

        let move_id = MoveId::Weatherball;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be Ice type (Super Effective vs Dragon/Ground - 4x)
        assert_eq!(result.effectiveness, 16); // 4x
        // 100 BP (No Snow boost for Ice)
        assert_eq!(result.final_base_power, 100);
    }

    #[test]
    fn test_weather_ball_no_weather() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("pikachu").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("gengar").unwrap().level(50).spawn(&mut state, 1, 0); // Ghost/Poison

        // No weather
        state.weather = Weather::None as u8;

        let move_id = MoveId::Weatherball;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be Normal type (Immune vs Ghost - 0x)
        assert_eq!(result.effectiveness, 0); // 0x
        // 50 BP
        assert_eq!(result.final_base_power, 50);
    }

    #[test]
    fn test_flying_press() {
        let mut state = BattleState::new();
        PokemonConfig::from_str("hawlucha").unwrap().level(50).spawn(&mut state, 0, 0);

        // Target: Abomasnow (Grass/Ice)
        // Fighting vs Grass (1x), Ice (2x) -> 2x
        // Flying vs Grass (2x), Ice (1x) -> 2x
        // Total = 4x.

        PokemonConfig::from_str("abomasnow").unwrap().level(50).spawn(&mut state, 1, 0);

        let move_id = MoveId::Flyingpress;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        assert_eq!(result.effectiveness, 16); // 4x
    }

    #[test]
    fn test_thousand_arrows() {
        let mut state = BattleState::new();
        // Zygarde vs Charizard (Fire/Flying)
        PokemonConfig::from_str("zygarde").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("charizard").unwrap().level(50).spawn(&mut state, 1, 0); // Fire/Flying

        let move_id = MoveId::Thousandarrows;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Normally Ground vs Fire/Flying is 0x (Immune).
        // With override: Ground vs Fire/Normal (Effectively)
        // Ground vs Fire (2x). Ground vs Normal (1x).
        // Total = 2x.
        assert_eq!(result.effectiveness, 8); // 2x
    }

    #[test]
    fn test_freeze_dry() {
        let mut state = BattleState::new();
        // Glalie vs Blastoise (Water)
        PokemonConfig::from_str("glalie").unwrap().level(50).spawn(&mut state, 0, 0);
        PokemonConfig::from_str("blastoise").unwrap().level(50).spawn(&mut state, 1, 0);

        let move_id = MoveId::Freezedry;
        let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);

        // Should be 2x vs Water
        assert_eq!(result.effectiveness, 8); // 2x

        // Vs Kingdra (Water/Dragon)
        // Ice vs Water (2x special). Ice vs Dragon (2x).
        // Total = 4x.
        PokemonConfig::from_str("kingdra").unwrap().level(50).spawn(&mut state, 1, 1); // Slot 1
        let result2 = calculate_damage(Gen9, &state, 0, 7, move_id, false); // 7 is player 1 slot 1

        assert_eq!(result2.effectiveness, 16); // 4x
    }
}
