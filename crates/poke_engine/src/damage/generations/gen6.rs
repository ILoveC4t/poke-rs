//! Generation 6 (X/Y, Omega Ruby/Alpha Sapphire) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;

/// Generation 6 mechanics (Pokémon X/Y/ORAS).
///
/// Key differences:
/// - Mega Evolution introduced
/// - Critical hits changed from 2.0x to 1.5x
/// - Terrain introduced (Electric, Grassy, Misty)
/// - No Z-Moves or Dynamax
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen6;

impl GenMechanics for Gen6 {
    const GEN: u8 = 6;
    
    // Mega Evolution exists
    fn has_mega_evolution(&self) -> bool {
        true
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // Terrain was 1.5x
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<u16> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(6144),
            (Terrain::Grassy, Type::Grass) => Some(6144),
            // Psychic Terrain didn't boost damage in Gen 6 (only priority blocking)
            (Terrain::Misty, Type::Dragon) => Some(2048),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gen6_crit() {
        let gen = Gen6;
        // Gen 6 introduced 1.5x crit
        assert_eq!(gen.crit_multiplier(), 6144);
    }
    
    #[test]
    fn test_gen6_features() {
        let gen = Gen6;
        
        assert!(gen.has_abilities());
        assert!(gen.has_held_items());
        assert!(gen.uses_physical_special_split());
        assert!(!gen.has_terastallization());
        assert!(gen.has_mega_evolution());
        assert!(!gen.has_z_moves());
        assert!(!gen.has_dynamax());
    }
}
