//! Generation 5 (Black/White, Black 2/White 2) mechanics.

use super::GenMechanics;
use crate::types::Type;
use crate::damage::Modifier;

/// Generation 5 mechanics (Pokémon Black/White/B2W2).
///
/// Key differences from Gen 6:
/// - Critical hits are 2.0x (not 1.5x)
/// - Steel resists Ghost/Dark
/// - No Mega Evolution
/// - No Terrain
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen5;

impl GenMechanics for Gen5 {
    const GEN: u8 = 5;
    
    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> Modifier {
        Modifier::DOUBLE // 2.0x
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        if has_adaptability { Modifier::DOUBLE } else { Modifier::ONE_POINT_FIVE }
    }
    
    // No terrain in Gen 5
    fn terrain_modifier(&self, _terrain: super::Terrain, _move_type: crate::types::Type, _is_grounded: bool) -> Option<Modifier> {
        None
    }

    // Steel resists Ghost/Dark
    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8 {
        let mut mult = crate::types::type_effectiveness(atk_type, def_type1, def_type2);

        let is_steel = def_type1 == Type::Steel || def_type2 == Some(Type::Steel);
        if is_steel {
            if atk_type == Type::Ghost || atk_type == Type::Dark {
                mult /= 2;
            }
        }
        
        mult
    }
}
