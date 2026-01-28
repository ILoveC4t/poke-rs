//! Generation 1 (Red/Blue/Yellow) mechanics.

use super::GenMechanics;
use crate::damage::formula::{apply_boost, of32};
use crate::damage::{DamageContext, DamageResult};
use crate::moves::MoveCategory;
use crate::types::Type;

/// Generation 1 mechanics.
///
/// Key differences:
/// - Special stat is shared (SpA = SpD)
/// - Critical hits double the level in damage formula
/// - Type effectiveness bugs (Psychic immune to Ghost)
/// - No items, abilities, or split
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen1;

impl GenMechanics for Gen1 {
    const GEN: u8 = 1;

    fn has_abilities(&self) -> bool {
        false
    }
    fn has_held_items(&self) -> bool {
        false
    }
    fn uses_physical_special_split(&self) -> bool {
        false
    }

    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8 {
        // Gen 1 specific type chart quirks
        // Let's reuse standard but override specific cases
        let mut m = crate::types::type_effectiveness(atk_type, def_type1, def_type2);

        // Fix Ghost vs Psychic (0x in Gen 1, normally 2x)
        if atk_type == Type::Ghost {
            if def_type1 == Type::Psychic || def_type2 == Some(Type::Psychic) {
                return 0;
            }
        }

        // Poison vs Bug (Gen 1: 2x)
        if atk_type == Type::Poison {
            if def_type1 == Type::Bug {
                // If standard says 1x (4), make it 2x (8)
                // If it was Bug/Grass (1x * 2x = 2x -> 8), then Gen 1 is (2x * 2x = 4x -> 16).
                // Just multiplying by 2 seems safe if standard is 1x.
                // Standard Poison vs Bug is 1x.
                m *= 2;
            }
            if def_type2 == Some(Type::Bug) {
                m *= 2;
            }
        }
        // Bug vs Poison (Gen 1: 2x)
        if atk_type == Type::Bug {
            if def_type1 == Type::Poison {
                // Standard Bug vs Poison is 0.5x (2). We want 2x (8).
                // So multiply by 4.
                m *= 4;
            }
            if def_type2 == Some(Type::Poison) {
                m *= 4;
            }
        }

        // Ice vs Fire (Gen 1: 1x, Standard: 0.5x)
        if atk_type == Type::Ice {
            if def_type1 == Type::Fire {
                // Standard 0.5x (2) -> 1x (4). Mult by 2.
                m *= 2;
            }
            if def_type2 == Some(Type::Fire) {
                m *= 2;
            }
        }

        m
    }

    fn calculate_damage(&self, ctx: &DamageContext<Self>) -> DamageResult {
        // Gen 1 Custom Damage Formula

        let _move_data = ctx.move_data;

        // 1. Determine Category (Type-based)
        let category = if is_type_special(ctx.move_type) {
            MoveCategory::Special
        } else {
            MoveCategory::Physical
        };

        // Gen 1 Crit: Level is doubled.
        // Stats: Ignore attacker's negative boosts and defender's positive boosts.

        let (mut atk_stat, def_stat) = if ctx.is_crit {
            // Indices: 1=Atk, 2=Def, 3=SpA, 4=SpD
            let (atk_idx, def_idx) = match category {
                MoveCategory::Physical => (1, 2),
                MoveCategory::Special => (3, 3), // SpA used for both
                _ => (0, 0),
            };

            let raw_atk = ctx.state.stats[ctx.attacker][atk_idx];
            let raw_def = ctx.state.stats[ctx.defender][def_idx];

            // Ignore ALL boosts (positive and negative)
            (raw_atk, raw_def)
        } else {
            // Use effective stats (boosted)
            let atk = match category {
                MoveCategory::Physical => apply_boost(
                    ctx.state.stats[ctx.attacker][1],
                    ctx.state.boosts[ctx.attacker][0],
                ),
                MoveCategory::Special => apply_boost(
                    ctx.state.stats[ctx.attacker][3],
                    ctx.state.boosts[ctx.attacker][2],
                ),
                _ => 0,
            };
            let def = match category {
                MoveCategory::Physical => apply_boost(
                    ctx.state.stats[ctx.defender][2],
                    ctx.state.boosts[ctx.defender][1],
                ),
                MoveCategory::Special => apply_boost(
                    ctx.state.stats[ctx.defender][3],
                    ctx.state.boosts[ctx.defender][2],
                ), // Use SpA boost for Special Defense too
                _ => 0,
            };
            (atk, def)
        };

        // Burn Mod (Gen 1): Halves Attack if burned and physical move.
        // Ignored on Crit.
        if !ctx.is_crit
            && category == MoveCategory::Physical
            && ctx.state.status[ctx.attacker].contains(crate::state::Status::BURN)
        {
            atk_stat /= 2;
        }

        // 3. Level
        let level = ctx.state.level[ctx.attacker] as u32;
        let effective_level = if ctx.is_crit { level * 2 } else { level };

        // 4. Base Power
        let power = ctx.base_power as u32;
        // Gen 1 doesn't have many BP modifiers (no items).

        // 5. Base Damage
        // Formula: min(997, floor(floor(floor(2 * L / 5 + 2) * A * P / D) / 50)) + 2

        if def_stat == 0 {
            return DamageResult::zero();
        }

        let level_term = 2 * effective_level / 5 + 2;

        // Gen 1 operations order
        let step1 = of32(level_term as u64 * atk_stat as u64 * power as u64);
        let step2 = step1 / (def_stat as u32);
        let step3 = step2 / 50;
        let base_damage_pre_mod = step3.min(997) + 2;

        // 6. Modifiers
        // STAB -> Type -> Random

        let mut damage = base_damage_pre_mod;

        // STAB (1.5x)
        if ctx.has_stab {
            damage = damage + (damage / 2);
        }

        // Type Effectiveness
        // In Gen 1: 0, 0.25 (message "not very effective"), 0.5 (not very), 1, 2, 4
        // Math: damage * 10 * effectiveness / 10 ??
        // Actually it performed multiplication: damage * eff_val / 10.
        // where eff_val: 0, 5, 10, 20, 40 (if 4x).

        // Reuse standard effectiveness calc which gives: 0, 1, 2, 4, 8, 16 (scale 4)
        // 0 -> 0x
        // 1 -> 0.25x
        // 2 -> 0.5x
        // 4 -> 1.0x
        // 8 -> 2.0x
        // 16 -> 4.0x
        // This maps perfectly if we divide by 4.

        if ctx.effectiveness != 4 {
            damage = damage * (ctx.effectiveness as u32) / 4;
        }

        // Random: 217 to 255
        // Generate rolls
        let mut rolls = [0u16; 16];
        // Standard rolls are 85..100.
        // Gen 1 random is uniform 217..255 / 255.
        // We need 16 values.
        // We'll approximate by picking 16 values in [217, 255].
        // 255 - 217 = 38 values. 16 steps.
        // stride = 38 / 15 = 2.53

        for i in 0..16 {
            let rnd = 217 + (i * 38 / 15);
            rolls[i] = (damage * (rnd as u32) / 255) as u16;
        }

        DamageResult {
            rolls,
            min: rolls[0],
            max: rolls[15],
            effectiveness: ctx.effectiveness,
            is_crit: ctx.is_crit,
            final_base_power: ctx.base_power,
        }
    }
}

fn is_type_special(t: Type) -> bool {
    matches!(
        t,
        Type::Fire
            | Type::Water
            | Type::Grass
            | Type::Ice
            | Type::Electric
            | Type::Psychic
            | Type::Dragon
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::calculate_damage;
    use crate::entities::PokemonConfig;
    use crate::moves::MoveId;
    use crate::state::BattleState;

    #[test]
    fn test_gen1_damage_basic() {
        let mut state = BattleState::new();

        // Pikachu (L50) vs Bulbasaur (L50)
        // Note: Species stats might differ in Gen 1 (Special split)
        // But our engine uses Species data which has SpA/SpD.
        // Gen 1 mechanics uses SpA for both.
        if let Some(mut config) = PokemonConfig::from_str("pikachu") {
            config = config.level(50);
            config.spawn(&mut state, 0, 0);
        }
        if let Some(mut config) = PokemonConfig::from_str("bulbasaur") {
            config = config.level(50);
            config.spawn(&mut state, 1, 0);
        }

        let thunderbolt = MoveId::from_str("thunderbolt").expect("thunderbolt");

        let result = calculate_damage(Gen1, &state, 0, 6, thunderbolt, false);

        assert!(result.max > 0, "Should deal damage");
        // Gen 1 randomness is different (217..255)
        // Max roll should correspond to 255/255 approx 1.0x
        // Min roll 217/255 approx 0.85x
        assert!(result.min < result.max);
    }

    #[test]
    fn test_gen1_psychic_immune_to_ghost() {
        let gen = Gen1;
        // Ghost vs Psychic = 0x
        assert_eq!(gen.type_effectiveness(Type::Ghost, Type::Psychic, None), 0);
    }
}
