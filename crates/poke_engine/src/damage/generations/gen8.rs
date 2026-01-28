//! Generation 8 (Sword/Shield) mechanics.

use super::{GenMechanics, Terrain};
use crate::damage::Modifier;
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

    fn dynamax_hp_multiplier(&self) -> f32 {
        2.0
    }

    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> Modifier {
        if has_adaptability {
            Modifier::DOUBLE
        } else {
            Modifier::ONE_POINT_FIVE
        }
    }

    // Terrain was 1.5x initially, then nerfed to 1.3x in later patches
    // Using 1.3x as the final value
    fn terrain_modifier(
        &self,
        terrain: Terrain,
        move_id: crate::moves::MoveId,
        move_type: Type,
        attacker_grounded: bool,
        defender_grounded: bool,
    ) -> Option<Modifier> {
        use crate::moves::MoveId;

        // Grassy Terrain: Halves Earthquake, Bulldoze, Magnitude if TARGET is grounded
        if terrain == Terrain::Grassy && defender_grounded {
            if matches!(
                move_id,
                MoveId::Earthquake | MoveId::Bulldoze | MoveId::Magnitude
            ) {
                return Some(Modifier::HALF);
            }
        }

        // Boosts (1.3x in Gen 8)
        if attacker_grounded {
            match (terrain, move_type) {
                (Terrain::Electric, Type::Electric) => return Some(Modifier::ONE_POINT_THREE), // 1.3x
                (Terrain::Grassy, Type::Grass) => return Some(Modifier::ONE_POINT_THREE),
                (Terrain::Psychic, Type::Psychic) => return Some(Modifier::ONE_POINT_THREE),
                _ => {}
            }
        }

        // Misty Terrain: 0.5x Dragon reduction
        if defender_grounded && terrain == Terrain::Misty && move_type == Type::Dragon {
            return Some(Modifier::HALF);
        }

        None
    }
}
