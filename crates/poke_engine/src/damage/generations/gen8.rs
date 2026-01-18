//! Generation 8 (Sword/Shield) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;

/// Generation 8 mechanics (Pokémon Sword/Shield).
///
/// Key differences from Gen 9:
/// - Dynamax instead of Terastallization
/// - Terrain boost was 1.5x before being reduced to 1.3x mid-gen
/// - Hail instead of Snow
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen8;

impl GenMechanics for Gen8 {
    const GEN: u8 = 8;
    
    // No Terastallization
    fn has_terastallization(&self) -> bool {
        false
    }
    
    // Dynamax exists
    fn has_dynamax(&self) -> bool {
        true
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // Terrain was 1.5x initially, then nerfed to 1.3x in later patches
    // Using 1.3x as the final value
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<u16> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(5325), // 1.3x
            (Terrain::Grassy, Type::Grass) => Some(5325),
            (Terrain::Psychic, Type::Psychic) => Some(5325),
            (Terrain::Misty, Type::Dragon) => Some(2048),      // 0.5x
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gen8_features() {
        let gen = Gen8;
        
        assert!(gen.has_abilities());
        assert!(gen.has_held_items());
        assert!(gen.uses_physical_special_split());
        assert!(!gen.has_terastallization());
        assert!(!gen.has_mega_evolution());
        assert!(!gen.has_z_moves());
        assert!(gen.has_dynamax());
    }
    
    #[test]
    fn test_gen8_stab_no_tera() {
        let gen = Gen8;
        
        // Tera STAB should be ignored in Gen 8
        assert_eq!(gen.stab_multiplier(false, true), 6144);  // Still 1.5x, not 2.0x
        assert_eq!(gen.stab_multiplier(true, true), 8192);   // Adaptability still 2.0x
    }
}
