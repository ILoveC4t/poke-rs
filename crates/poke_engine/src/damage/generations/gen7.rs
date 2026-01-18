//! Generation 7 (Sun/Moon, Ultra Sun/Ultra Moon) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;

/// Generation 7 mechanics (Pokémon Sun/Moon/USUM).
///
/// Key differences from Gen 8:
/// - Z-Moves instead of Dynamax
/// - Terrain boost was 1.5x
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen7;

impl GenMechanics for Gen7 {
    const GEN: u8 = 7;
    
    // Z-Moves exist
    fn has_z_moves(&self) -> bool {
        true
    }
    
    // Mega Evolution exists
    fn has_mega_evolution(&self) -> bool {
        true
    }

    fn can_mega_evolve(&self) -> bool {
        true
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // Terrain was 1.5x in Gen 7
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<u16> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(6144), // 1.5x
            (Terrain::Grassy, Type::Grass) => Some(6144),
            (Terrain::Psychic, Type::Psychic) => Some(6144),
            (Terrain::Misty, Type::Dragon) => Some(2048),      // 0.5x
            _ => None,
        }
    }
}
