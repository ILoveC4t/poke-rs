//! Generation 7 (Sun/Moon, Ultra Sun/Ultra Moon) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;
use crate::damage::Modifier;

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
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        if has_adaptability { Modifier::DOUBLE } else { Modifier::ONE_POINT_FIVE }
    }
    
    // Terrain was 1.5x in Gen 7
    // TODO: Psychic Terrain boost should apply when ATTACKER is grounded,
    //       not defender. Current implementation checks defender grounding.
    //       Fix terrain_modifier call site to pass attacker grounding for boost.
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<Modifier> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(Modifier::ONE_POINT_FIVE), // 1.5x
            (Terrain::Grassy, Type::Grass) => Some(Modifier::ONE_POINT_FIVE),
            (Terrain::Psychic, Type::Psychic) => Some(Modifier::ONE_POINT_FIVE),
            (Terrain::Misty, Type::Dragon) => Some(Modifier::HALF),      // 0.5x
            _ => None,
        }
    }
}
