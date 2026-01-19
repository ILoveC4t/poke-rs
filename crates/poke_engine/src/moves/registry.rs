//! Move hook registry.
//!
//! Static registry mapping MoveId to MoveHooks for conditional move logic.

use crate::moves::MoveId;
use super::hooks::MoveHooks;
use super::implementations::*;

pub static MOVE_REGISTRY: [Option<MoveHooks>; MoveId::COUNT] = {
    let mut registry: [Option<MoveHooks>; MoveId::COUNT] = [None; MoveId::COUNT];

    // =========================================================================
    // Conditional Base Power Moves (OnBasePowerCondition + multiplier)
    // =========================================================================

    // Knock Off: 1.5x if target has a removable item
    registry[MoveId::Knockoff as usize] = Some(MoveHooks {
        on_base_power_condition: Some(knockoff_condition),
        conditional_multiplier: 6144, // 1.5x (6144/4096)
        ..MoveHooks::NONE
    });

    // Venoshock: 2x if target is poisoned
    registry[MoveId::Venoshock as usize] = Some(MoveHooks {
        on_base_power_condition: Some(venoshock_condition),
        conditional_multiplier: 8192, // 2x
        ..MoveHooks::NONE
    });

    // Hex: 2x if target has any status
    registry[MoveId::Hex as usize] = Some(MoveHooks {
        on_base_power_condition: Some(hex_condition),
        conditional_multiplier: 8192, // 2x
        ..MoveHooks::NONE
    });

    // Brine: 2x if target is at or below 50% HP
    registry[MoveId::Brine as usize] = Some(MoveHooks {
        on_base_power_condition: Some(brine_condition),
        conditional_multiplier: 8192, // 2x
        ..MoveHooks::NONE
    });

    registry
};
