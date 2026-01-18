//! Generation 2 (Gold/Silver/Crystal) mechanics.

use super::{GenMechanics, Terrain};
use crate::types::Type;

/// Generation 2 mechanics.
///
/// Key features:
/// - Split Special into SpA and SpD
/// - Held items introduced
/// - Steel and Dark types introduced
/// - Type effectiveness: Steel resists Ghost/Dark (unlike Gen 6+)
/// - 2.0x crit multiplier
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen2;

impl GenMechanics for Gen2 {
    const GEN: u8 = 2;

    // Gen 2 has items, no abilities
    fn has_abilities(&self) -> bool { false }
    fn has_held_items(&self) -> bool { true }
    fn uses_physical_special_split(&self) -> bool { false }

    // 2.0x crit
    fn crit_multiplier(&self) -> u16 {
        8192 // 2.0x
    }

    // Type chart overrides
    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8 {
        // Standard chart calculation
        let mut mult = crate::types::type_effectiveness(atk_type, def_type1, def_type2);

        // Gen 2-5: Ghost and Dark are NVE (0.5x) against Steel
        // Gen 6+: Neutral (1x)
        // Our standard chart (Gen 9) has them as Neutral.
        // We need to apply resistance if defender is Steel.

        let is_steel = def_type1 == Type::Steel || def_type2 == Some(Type::Steel);
        if is_steel {
            if atk_type == Type::Ghost || atk_type == Type::Dark {
                // If it was neutral (4), make it resistant (2)
                // If it was super effective (8), make it neutral (4) - e.g. Steel/Psychic vs Ghost
                // But wait, type_effectiveness returns the aggregate multiplier.
                // We can't easily "undo" just the Steel part without knowing the other type's interaction.
                // However, we know Steel is the ONLY type that changed its resistance to Ghost/Dark.
                // In Gen 9, Steel vs Ghost/Dark is 1x.
                // In Gen 2, Steel vs Ghost/Dark is 0.5x.
                // So we basically halve the multiplier.
                mult /= 2;
            }
        }

        mult
    }

    // No terrain
    fn terrain_modifier(&self, _terrain: Terrain, _move_type: Type, _is_grounded: bool) -> Option<u16> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    #[test]
    fn test_gen2_steel_resistance() {
        let gen = Gen2;

        // Steel vs Ghost
        // Gen 2: Steel resists Ghost (0.5x)
        // Standard: Neutral (1.0x)
        // type_effectiveness returns 4 scale (4=1x, 2=0.5x)

        // Ghost attacking Steel
        assert_eq!(gen.type_effectiveness(Type::Ghost, Type::Steel, None), 2); // 0.5x

        // Dark attacking Steel
        assert_eq!(gen.type_effectiveness(Type::Dark, Type::Steel, None), 2); // 0.5x

        // Fire attacking Steel (Super effective -> 2x = 8)
        assert_eq!(gen.type_effectiveness(Type::Fire, Type::Steel, None), 8);
    }
}
