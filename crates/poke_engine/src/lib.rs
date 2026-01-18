//! poke_engine - High-performance Pokemon battle simulation engine
//!
//! This library provides stack-allocated, cache-friendly data structures
//! optimized for Monte Carlo and Minimax AI analysis.

#![allow(clippy::transmute_int_to_non_zero)]

/// Type definitions and type chart
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

/// Nature definitions and stat modifiers
pub mod natures {
    include!(concat!(env!("OUT_DIR"), "/natures.rs"));
}

/// Ability identifiers and hooks
pub mod abilities;

/// Species data and lookup
pub mod species {
    include!(concat!(env!("OUT_DIR"), "/species.rs"));
}

/// Move identifiers
pub mod moves {
    include!(concat!(env!("OUT_DIR"), "/moves.rs"));
}

/// Item identifiers
pub mod items {
    include!(concat!(env!("OUT_DIR"), "/items.rs"));
}

/// Terrain definitions
pub mod terrains {
    include!(concat!(env!("OUT_DIR"), "/terrains.rs"));
}

/// Battle state (SoA memory layout)
pub mod state;

/// Entity blueprints and spawning
pub mod entities;

/// Damage calculation pipeline
pub mod damage;

// Re-export commonly used types
pub use abilities::AbilityId;
pub use entities::PokemonConfig;
pub use items::ItemId;
pub use moves::{Move, MoveCategory, MoveFlags, MoveId};
pub use natures::{BattleStat, NatureId};
pub use terrains::TerrainId;
pub use species::{Species, SpeciesId};
pub use state::BattleState;
pub use types::{Type, TypeEffectiveness, TypeImmunities};
pub use damage::{calculate_damage, DamageResult, Gen9, Generation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_lookup() {
        assert_eq!(Type::from_str("fire"), Some(Type::Fire));
        assert_eq!(Type::from_str("Fire"), Some(Type::Fire));
        assert_eq!(Type::from_str("invalid"), None);
    }

    #[test]
    fn test_type_effectiveness() {
        use types::{type_effectiveness, TYPE_CHART};

        // Fire vs Grass = 2x
        assert_eq!(
            TYPE_CHART[Type::Grass as usize][Type::Fire as usize],
            TypeEffectiveness::SuperEffective
        );

        // Water vs Fire = 2x
        assert_eq!(type_effectiveness(Type::Water, Type::Fire, None), 8);

        // Ground vs Flying = 0x
        assert_eq!(type_effectiveness(Type::Ground, Type::Flying, None), 0);

        // Ice vs Grass/Flying = 4x
        assert_eq!(
            type_effectiveness(Type::Ice, Type::Grass, Some(Type::Flying)),
            16
        );
    }

    #[test]
    fn test_nature_modifiers() {
        // Adamant: +Atk, -SpA
        let adamant = NatureId::from_str("adamant").unwrap();
        assert_eq!(adamant.stat_modifier(BattleStat::Atk), 11);
        assert_eq!(adamant.stat_modifier(BattleStat::SpA), 9);
        assert_eq!(adamant.stat_modifier(BattleStat::Spe), 10);
        assert!(!adamant.is_neutral());

        // Hardy: neutral
        let hardy = NatureId::from_str("hardy").unwrap();
        assert!(hardy.is_neutral());
        assert_eq!(hardy.stat_modifier(BattleStat::Atk), 10);
    }

    #[test]
    fn test_species_lookup() {
        let pikachu = SpeciesId::from_str("pikachu").expect("pikachu should exist");
        let data = pikachu.data();
        assert_eq!(data.base_stats[0], 35); // HP
        assert_eq!(data.primary_type(), Type::Electric);
        assert!(data.secondary_type().is_none());
    }

    #[test]
    fn test_form_base_species() {
        let mega = SpeciesId::from_str("venusaurmega").expect("venusaurmega should exist");
        let base = mega.base();
        let base_direct = SpeciesId::from_str("venusaur").expect("venusaur should exist");
        assert_eq!(base, base_direct);

        // Base species returns itself
        assert_eq!(base_direct.base(), base_direct);
    }

    #[test]
    fn test_ability_lookup() {
        let levitate = AbilityId::from_str("levitate").expect("levitate should exist");
        assert_eq!(levitate, AbilityId::Levitate);
    }

    #[test]
    fn test_item_lookup() {
        let ability_shield =
            ItemId::from_str("abilityshield").expect("Ability Shield should exist");
        let data = ability_shield.data();
        assert_eq!(data.fling_power, 30);
    }

    #[test]
    fn test_move_data_and_terrain() {
        // Test Electric Terrain move
        let et = MoveId::from_str("electricterrain").expect("electricterrain should exist");
        let data = et.data();
        assert_eq!(data.name, "Electric Terrain");
        assert_eq!(data.terrain, TerrainId::Electric);
        assert_eq!(data.category, MoveCategory::Status);

        // Test normal move (Thunderbolt)
        let tbolt = MoveId::from_str("thunderbolt").expect("thunderbolt should exist");
        let tbolt_data = tbolt.data();
        assert_eq!(tbolt_data.name, "Thunderbolt");
        assert_eq!(tbolt_data.terrain, TerrainId::None);
        assert_eq!(tbolt_data.category, MoveCategory::Special);
        assert_eq!(tbolt_data.power, 90);
    }
}
