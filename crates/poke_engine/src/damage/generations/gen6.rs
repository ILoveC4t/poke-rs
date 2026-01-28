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

    // Terrain was 1.5x (and no Psychic Terrain)
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

        // Boosts (1.5x in Gen 6)
        if attacker_grounded {
            match (terrain, move_type) {
                (Terrain::Electric, Type::Electric) => return Some(Modifier::ONE_POINT_FIVE),
                (Terrain::Grassy, Type::Grass) => return Some(Modifier::ONE_POINT_FIVE),
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
