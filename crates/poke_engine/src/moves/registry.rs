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

    // Facade: 2x if statused
    registry[MoveId::Facade as usize] = Some(MoveHooks {
        on_base_power_condition: Some(facade_condition),
        on_ignore_status_damage_reduction: Some(on_ignore_status_damage_reduction_facade),
        conditional_multiplier: 8192, // 2x
        ..MoveHooks::NONE
    });


    // =========================================================================
    // Special Moves
    // =========================================================================

    registry[MoveId::Weatherball as usize] = Some(MoveHooks {
        on_modify_type: Some(on_modify_type_weather_ball),
        on_modify_base_power: Some(on_modify_base_power_weather_ball),
        on_modify_final_damage: Some(on_modify_final_damage_weather_ball),
        ..MoveHooks::NONE
    });

    registry[MoveId::Freezedry as usize] = Some(MoveHooks {
        on_modify_effectiveness: Some(freeze_dry_effectiveness),
        ..MoveHooks::NONE
    });

    registry[MoveId::Flyingpress as usize] = Some(MoveHooks {
        on_modify_effectiveness: Some(flying_press_effectiveness),
        ..MoveHooks::NONE
    });

    registry[MoveId::Thousandarrows as usize] = Some(MoveHooks {
        on_modify_effectiveness: Some(thousand_arrows_effectiveness),
        ..MoveHooks::NONE
    });

    // =========================================================================
    // Variable Power Moves (Weight, HP, etc.)
    // =========================================================================

    registry[MoveId::Grassknot as usize] = Some(MoveHooks {
        on_modify_base_power: Some(grass_knot_power),
        ..MoveHooks::NONE
    });
    registry[MoveId::Lowkick as usize] = Some(MoveHooks {
        on_modify_base_power: Some(grass_knot_power),
        ..MoveHooks::NONE
    });

    registry[MoveId::Heavyslam as usize] = Some(MoveHooks {
        on_modify_base_power: Some(heavy_slam_power),
        ..MoveHooks::NONE
    });
    registry[MoveId::Heatcrash as usize] = Some(MoveHooks {
        on_modify_base_power: Some(heavy_slam_power),
        ..MoveHooks::NONE
    });

    registry[MoveId::Eruption as usize] = Some(MoveHooks {
        on_modify_base_power: Some(eruption_power),
        ..MoveHooks::NONE
    });
    registry[MoveId::Waterspout as usize] = Some(MoveHooks {
        on_modify_base_power: Some(eruption_power),
        ..MoveHooks::NONE
    });

    registry[MoveId::Flail as usize] = Some(MoveHooks {
        on_modify_base_power: Some(flail_power),
        ..MoveHooks::NONE
    });
    registry[MoveId::Reversal as usize] = Some(MoveHooks {
        on_modify_base_power: Some(flail_power),
        ..MoveHooks::NONE
    });

    registry[MoveId::Return as usize] = Some(MoveHooks {
        on_modify_base_power: Some(return_power),
        ..MoveHooks::NONE
    });
    registry[MoveId::Frustration as usize] = Some(MoveHooks {
        on_modify_base_power: Some(frustration_power),
        ..MoveHooks::NONE
    });

    // =========================================================================
    // Type-Changing Moves (Judgment, Techno Blast, Multi-Attack)
    // =========================================================================

    registry[MoveId::Judgment as usize] = Some(MoveHooks {
        on_modify_type: Some(on_modify_type_judgment),
        ..MoveHooks::NONE
    });

    registry[MoveId::Ragingbull as usize] = Some(MoveHooks {
        on_modify_type: Some(on_modify_type_raging_bull),
        ..MoveHooks::NONE
    });

    registry
};
