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

/// Ability identifiers
pub mod abilities {
    include!(concat!(env!("OUT_DIR"), "/abilities.rs"));
}

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

/// Battle state (SoA memory layout)
pub mod state;

/// Entity blueprints and spawning
pub mod entities;

// Re-export commonly used types
pub use abilities::AbilityId;
pub use entities::PokemonConfig;
pub use items::ItemId;
pub use moves::MoveId;
pub use natures::{BattleStat, NatureId};
pub use species::{Species, SpeciesId};
pub use state::BattleState;
pub use types::{Type, TypeEffectiveness, TypeImmunities};

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
}
