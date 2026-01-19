//! Generation 6 (X/Y, Omega Ruby/Alpha Sapphire) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;
use crate::damage::Modifier;

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

    fn can_mega_evolve(&self) -> bool {
        true
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        if has_adaptability { Modifier::DOUBLE } else { Modifier::ONE_POINT_FIVE }
    }
    
    // Terrain was 1.5x
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<Modifier> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(Modifier::ONE_POINT_FIVE),
            (Terrain::Grassy, Type::Grass) => Some(Modifier::ONE_POINT_FIVE),
            // Psychic Terrain didn't boost damage in Gen 6 (only priority blocking)
            (Terrain::Misty, Type::Dragon) => Some(Modifier::HALF),
            _ => None,
        }
    }
}
