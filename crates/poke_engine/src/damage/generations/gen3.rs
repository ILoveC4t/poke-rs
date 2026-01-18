//! Generation 3 (Ruby/Sapphire/Emerald, FireRed/LeafGreen) mechanics.

use super::GenMechanics;
use crate::types::Type;

/// Generation 3 mechanics (Pokémon RSE/FRLG).
///
/// Key features:
/// - Abilities introduced
/// - Type-based Physical/Special (no per-move split)
/// - 2.0x crit multiplier
/// - No items affecting damage the same way (no Life Orb, etc.)
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen3;

impl GenMechanics for Gen3 {
    const GEN: u8 = 3;
    
    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> u16 {
        8192
    }
    
    // No physical/special split - type determines category
    fn uses_physical_special_split(&self) -> bool {
        false
    }
    
    // STAB - Adaptability didn't exist until Gen 4
    fn stab_multiplier(&self, _has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        6144 // Always 1.5x
    }
    
    // No terrain
    fn terrain_modifier(&self, _terrain: super::Terrain, _move_type: Type, _is_grounded: bool) -> Option<u16> {
        None
    }
}

/// Determine if a type is Physical or Special in Gen 1-3.
///
/// Before Gen 4, move category was determined by type:
/// - Physical: Normal, Fighting, Flying, Ground, Rock, Bug, Ghost, Poison, Steel
/// - Special: Fire, Water, Grass, Electric, Psychic, Ice, Dragon, Dark
#[allow(dead_code)]
pub fn is_type_physical_gen3(move_type: Type) -> bool {
    matches!(
        move_type,
        Type::Normal
            | Type::Fighting
            | Type::Flying
            | Type::Ground
            | Type::Rock
            | Type::Bug
            | Type::Ghost
            | Type::Poison
            | Type::Steel
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gen3_no_split() {
        let gen = Gen3;
        assert!(!gen.uses_physical_special_split());
    }
    
    #[test]
    fn test_gen3_type_physical() {
        // Physical types
        assert!(is_type_physical_gen3(Type::Normal));
        assert!(is_type_physical_gen3(Type::Fighting));
        assert!(is_type_physical_gen3(Type::Ghost)); // Ghost was Physical pre-split!
        
        // Special types
        assert!(!is_type_physical_gen3(Type::Fire));
        assert!(!is_type_physical_gen3(Type::Water));
        assert!(!is_type_physical_gen3(Type::Psychic));
        assert!(!is_type_physical_gen3(Type::Dark));
    }
    
    #[test]
    fn test_gen3_stab_no_adaptability() {
        let gen = Gen3;
        // Adaptability didn't exist
        assert_eq!(gen.stab_multiplier(true, false), 6144); // Still 1.5x
    }
}
