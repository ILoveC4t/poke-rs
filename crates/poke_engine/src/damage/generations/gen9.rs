//! Generation 9 (Scarlet/Violet) mechanics.
//!
//! This is the canonical, default implementation. All trait defaults
//! in `GenMechanics` reflect Gen 9 behavior.

use super::GenMechanics;

/// Generation 9 mechanics (Pokémon Scarlet/Violet).
///
/// Key features:
/// - Terastallization
/// - 1.5x critical hit multiplier
/// - 1.3x terrain boost
/// - Snow weather (replaced Hail damage with Defense boost)
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen9;

impl GenMechanics for Gen9 {
    const GEN: u8 = 9;

    // All defaults match Gen 9, so no overrides needed.
    // Explicit implementations can be added here for documentation
    // or if the defaults change.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::generations::{Terrain, Weather};
    use crate::damage::Modifier;
    use crate::types::Type;

    #[test]
    fn test_gen9_crit_multiplier() {
        let gen = Gen9;
        assert_eq!(gen.crit_multiplier(), Modifier::ONE_POINT_FIVE); // 1.5x
    }

    #[test]
    fn test_gen9_stab() {
        let gen = Gen9;

        // Normal STAB
        assert_eq!(gen.stab_multiplier(false, false), Modifier::ONE_POINT_FIVE); // 1.5x

        // Adaptability STAB
        assert_eq!(gen.stab_multiplier(true, false), Modifier::DOUBLE); // 2.0x

        // Tera STAB
        assert_eq!(gen.stab_multiplier(false, true), Modifier::DOUBLE); // 2.0x
    }

    #[test]
    fn test_gen9_weather() {
        let gen = Gen9;

        // Sun boosts Fire
        assert_eq!(
            gen.weather_modifier(Weather::Sun, Type::Fire),
            Some(Modifier::ONE_POINT_FIVE)
        );
        // Sun weakens Water
        assert_eq!(
            gen.weather_modifier(Weather::Sun, Type::Water),
            Some(Modifier::HALF)
        );

        // Rain boosts Water
        assert_eq!(
            gen.weather_modifier(Weather::Rain, Type::Water),
            Some(Modifier::ONE_POINT_FIVE)
        );
        // Rain weakens Fire
        assert_eq!(
            gen.weather_modifier(Weather::Rain, Type::Fire),
            Some(Modifier::HALF)
        );

        // No effect on neutral types
        assert_eq!(gen.weather_modifier(Weather::Sun, Type::Electric), None);
    }

    #[test]
    fn test_gen9_terrain() {
        let gen = Gen9;
        use crate::moves::MoveId;

        // Electric Terrain boosts Electric moves for grounded Pokemon
        // We pass MoveId::Thunderbolt as a dummy Electric move
        assert_eq!(
            gen.terrain_modifier(
                Terrain::Electric,
                MoveId::Thunderbolt,
                Type::Electric,
                true,
                true
            ),
            Some(Modifier::ONE_POINT_THREE)
        );

        // Not grounded (attacker) = no boost
        assert_eq!(
            gen.terrain_modifier(
                Terrain::Electric,
                MoveId::Thunderbolt,
                Type::Electric,
                false,
                true
            ),
            None
        );

        // Misty Terrain weakens Dragon (if target grounded)
        // MoveId::Dragonclaw as dummy
        assert_eq!(
            gen.terrain_modifier(Terrain::Misty, MoveId::Dragonclaw, Type::Dragon, true, true),
            Some(Modifier::HALF)
        );

        // Grassy Terrain reduces Earthquake (if target grounded)
        assert_eq!(
            gen.terrain_modifier(
                Terrain::Grassy,
                MoveId::Earthquake,
                Type::Ground,
                true,
                true
            ),
            Some(Modifier::HALF)
        );

        // Psychic Terrain boosts Psychic moves for grounded Pokemon
        assert_eq!(
            gen.terrain_modifier(
                Terrain::Psychic,
                MoveId::Psychic,
                Type::Psychic,
                true,
                true
            ),
            Some(Modifier::ONE_POINT_THREE)
        );

        // Not grounded (attacker) = no boost
        assert_eq!(
            gen.terrain_modifier(
                Terrain::Psychic,
                MoveId::Psychic,
                Type::Psychic,
                false,
                true
            ),
            None
        );
    }

    #[test]
    fn test_gen9_features() {
        let gen = Gen9;

        assert!(gen.has_abilities());
        assert!(gen.has_held_items());
        assert!(gen.uses_physical_special_split());
        assert!(gen.has_terastallization());
        assert!(!gen.has_mega_evolution());
        assert!(!gen.has_z_moves());
        assert!(!gen.has_dynamax());
    }
}
