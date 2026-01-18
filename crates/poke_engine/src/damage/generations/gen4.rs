//! Generation 4 (Diamond/Pearl/Platinum, HeartGold/SoulSilver) mechanics.

use super::GenMechanics;
use crate::types::Type;

/// Generation 4 mechanics (Pokémon DPPt/HGSS).
///
/// Key features:
/// - Physical/Special split introduced (moves have their own category)
/// - 2.0x crit multiplier
/// - Steel resists Ghost/Dark
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen4;

impl GenMechanics for Gen4 {
    const GEN: u8 = 4;
    
    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> u16 {
        8192
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // No terrain
    fn terrain_modifier(&self, _terrain: super::Terrain, _move_type: crate::types::Type, _is_grounded: bool) -> Option<u16> {
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
