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
            if matches!(move_id, MoveId::Earthquake | MoveId::Bulldoze | MoveId::Magnitude) {
                return Some(Modifier::HALF);
            }
        }

        // Boosts (1.5x in Gen 7)
        if attacker_grounded {
            match (terrain, move_type) {
                (Terrain::Electric, Type::Electric) => return Some(Modifier::ONE_POINT_FIVE), // 1.5x
                (Terrain::Grassy, Type::Grass) => return Some(Modifier::ONE_POINT_FIVE),
                (Terrain::Psychic, Type::Psychic) => return Some(Modifier::ONE_POINT_FIVE),
                _ => {}
            }
        }

        // Misty Terrain: 0.5x Dragon reduction (if target grounded)
        if defender_grounded && terrain == Terrain::Misty && move_type == Type::Dragon {
            return Some(Modifier::HALF);
        }

        None
    }
}
