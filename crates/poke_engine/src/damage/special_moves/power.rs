use crate::damage::context::DamageContext;
use crate::damage::generations::GenMechanics;
use crate::moves::{MoveFlags, MoveId};

/// Check if a move has variable base power (often 0 in data).
pub fn is_variable_power(move_id: MoveId) -> bool {
    // Frustration and Return are variable power but might lack the flag in old data
    if matches!(move_id, MoveId::Frustration | MoveId::Return) {
        return true;
    }
    move_id.data().flags.contains(MoveFlags::VARIABLE_POWER)
}

/// Modify base power based on special move logic (weight, HP, etc.)
///
/// This is called BEFORE ability modifiers (Technician, etc.).
/// Returns the modified base power.
pub fn modify_base_power<G: GenMechanics>(ctx: &DamageContext<'_, G>) -> u32 {
    // Logic migrated to MoveHooks.
    // This function remains as a placeholder or specific overrides not yet migrated.
    ctx.base_power as u32
}
