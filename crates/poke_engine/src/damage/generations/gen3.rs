//! Generation 3 (Ruby/Sapphire/Emerald, FireRed/LeafGreen) mechanics.

use super::GenMechanics;
use crate::types::Type;
use crate::damage::Modifier;

/// Generation 3 mechanics (Pokémon RSE/FRLG).
///
/// Key features:
/// - Abilities introduced
/// - Type-based Physical/Special (no per-move split)
/// - 2.0x crit multiplier
/// - Steel resists Ghost/Dark
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen3;

impl GenMechanics for Gen3 {
    const GEN: u8 = 3;

    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> Modifier {
        Modifier::DOUBLE
    }

    // No physical/special split - type determines category
    fn uses_physical_special_split(&self) -> bool {
        false
    }

    // STAB - Adaptability didn't exist until Gen 4
    fn stab_multiplier(&self, _has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        Modifier::ONE_POINT_FIVE // Always 1.5x
    }

    // No terrain
    fn terrain_modifier(
        &self,
        _terrain: super::Terrain,
        _move_id: crate::moves::MoveId,
        _move_type: Type,
        _attacker_grounded: bool,
        _defender_grounded: bool,
    ) -> Option<Modifier> {
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
