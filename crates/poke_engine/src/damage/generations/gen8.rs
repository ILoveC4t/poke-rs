//! Generation 8 (Sword/Shield) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;
use crate::damage::Modifier;

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

    fn dynamax_hp_multiplier(&self) -> f32 {
        2.0
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        if has_adaptability { Modifier::DOUBLE } else { Modifier::ONE_POINT_FIVE }
    }
    
    // Terrain was 1.5x initially, then nerfed to 1.3x in later patches
    // Using 1.3x as the final value
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<Modifier> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(Modifier::ONE_POINT_THREE), // 1.3x
            (Terrain::Grassy, Type::Grass) => Some(Modifier::ONE_POINT_THREE),
            (Terrain::Psychic, Type::Psychic) => Some(Modifier::ONE_POINT_THREE),
            (Terrain::Misty, Type::Dragon) => Some(Modifier::HALF),      // 0.5x
            _ => None,
        }
    }
}
