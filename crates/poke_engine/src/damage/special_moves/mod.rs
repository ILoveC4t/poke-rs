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

        // Weather Ball migrated to MoveHooks (OnModifyType, OnModifyBasePower)

        // Flying Press/Thousand Arrows/Freeze-Dry migrated to OnModifyEffectiveness
        MoveId::Flyingpress => {}
        MoveId::Thousandarrows => {}
        MoveId::Freezedry => {}

        _ => {}
    }
}
