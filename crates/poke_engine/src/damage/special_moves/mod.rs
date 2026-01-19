pub mod fixed;
pub mod power;
#[cfg(test)]
mod tests;

use crate::damage::context::DamageContext;
use crate::damage::generations::{GenMechanics, Weather};
use crate::moves::MoveId;
use crate::types::Type;

pub use fixed::get_fixed_damage;
pub use power::{is_variable_power, modify_base_power};

/// Apply special move logic that overrides standard mechanics.
///
/// This handles moves like Weather Ball (changes type/power), Struggle (typeless),
/// Flying Press (dual type), etc.
///
/// This mutates the `DamageContext` directly.
pub fn apply_special_moves<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    match ctx.move_id {
        MoveId::Struggle => {
            // Typeless damage, hits Ghost, fixed 50 BP
            ctx.effectiveness = 4; // 1x (Neutral)
            ctx.has_stab = false;
            ctx.base_power = 50;
            // Note: Recoil is handled in battle loop
        }

        MoveId::Weatherball => {
            let weather = Weather::from_u8(ctx.state.weather);
            // TODO: Weather Ball gets 2x power in weather (50 -> 100), but we also need
            //       to apply weather damage boost (1.5x for matching type). Currently
            //       only type/power change is handled; weather boost is applied separately
            //       but tests expect combined 1.5x effect on top of 100 BP.
            let (new_type, power) = match weather {
                Weather::Sun | Weather::HarshSun => (Type::Fire, 100),
                Weather::Rain | Weather::HeavyRain => (Type::Water, 100),
                Weather::Sand => (Type::Rock, 100),
                Weather::Hail | Weather::Snow => (Type::Ice, 100),
                _ => (Type::Normal, 50),
            };

            // If weather is active, type and power change
            if power == 100 {
                ctx.move_type = new_type;
                ctx.base_power = 100;

                // Gen 3: Update category based on new type (Physical/Special split)
                if !ctx.gen.uses_physical_special_split() {
                     ctx.category = if matches!(new_type, 
                        Type::Fire | Type::Water | Type::Grass | Type::Electric | 
                        Type::Psychic | Type::Ice | Type::Dragon | Type::Dark) {
                         crate::moves::MoveCategory::Special
                     } else {
                         crate::moves::MoveCategory::Physical
                     };
                }

                // Recalculate STAB
                // Handle Forecast: Castform changes type to match weather (except Sand)
                let is_forecast = ctx.attacker_ability == crate::abilities::AbilityId::Forecast;
                let forecasts_matches = match weather {
                    Weather::Sun | Weather::HarshSun | Weather::Rain | Weather::HeavyRain | Weather::Hail | Weather::Snow => true,
                    _ => false,
                };

                let attacker_types = ctx.state.types[ctx.attacker];
                ctx.has_stab = new_type == attacker_types[0] || new_type == attacker_types[1]
                    || (is_forecast && forecasts_matches);

                // Recalculate Effectiveness
                let def_types = ctx.state.types[ctx.defender];
                let type2 = if def_types[1] != def_types[0] { Some(def_types[1]) } else { None };
                ctx.effectiveness = ctx.gen.type_effectiveness(new_type, def_types[0], type2);
            }
        }

        MoveId::Flyingpress => {
            // Dual type effectiveness: Fighting (Base) + Flying
            // ctx.effectiveness currently holds Fighting effectiveness

            let def_types = ctx.state.types[ctx.defender];
            let type2 = if def_types[1] != def_types[0] { Some(def_types[1]) } else { None };

            // Calculate Flying effectiveness
            let flying_eff = ctx.gen.type_effectiveness(Type::Flying, def_types[0], type2);

            // Combine: (Fighting * Flying) / 4 (since 4 is 1x)
            // Example: 2x (8) * 2x (8) = 64 / 4 = 16 (4x)
            // Example: 1x (4) * 0.5x (2) = 8 / 4 = 2 (0.5x)
            ctx.effectiveness = ((ctx.effectiveness as u16 * flying_eff as u16) / 4) as u8;
        }

        MoveId::Thousandarrows => {
            // Ground move that hits Flying types neutrally
            let def_types = ctx.state.types[ctx.defender];

            // Check if target is Flying
            let t1 = def_types[0];
            let t2 = if def_types[1] != def_types[0] { Some(def_types[1]) } else { None };

            let t1_is_flying = t1 == Type::Flying;
            let t2_is_flying = t2 == Some(Type::Flying);

            if t1_is_flying || t2_is_flying {
                // Re-calculate effectiveness replacing Flying with Normal (Neutral vs Ground)
                // This ensures we bypass the immunity while preserving other type interactions
                let effective_t1 = if t1_is_flying { Type::Normal } else { t1 };
                let effective_t2 = if t2_is_flying { Some(Type::Normal) } else { t2 };

                ctx.effectiveness = ctx.gen.type_effectiveness(Type::Ground, effective_t1, effective_t2);
            }
        }

        MoveId::Freezedry => {
            // Ice move super effective vs Water
            let def_types = ctx.state.types[ctx.defender];

            if def_types[0] == Type::Water || def_types[1] == Type::Water {
                // Ice vs Water is normally 0.5x (2). We want 2x (8).
                // Multiply by 4.
                // Note: If target is Water/Water, we simply apply 2x multiplier logic.
                // If target is Water/Grass:
                // Normal Ice: Water(0.5) * Grass(2) = 1.0 (4)
                // Freeze-Dry: Water(2.0) * Grass(2) = 4.0 (16)
                // 4 * 4 = 16. Correct.
                ctx.effectiveness = (ctx.effectiveness as u16 * 4).min(255) as u8;
            }
        }

        _ => {}
    }
}
