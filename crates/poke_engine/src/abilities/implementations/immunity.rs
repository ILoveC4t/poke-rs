//! Type immunity abilities.
//!
//! Called via `OnTypeImmunity` to check if a move is blocked by an ability.
//! Returns true if immune.

use crate::state::{BattleState, Status};
use crate::types::Type;

/// Levitate: Immune to Ground moves (if not grounded)
pub fn levitate(state: &BattleState, defender: usize, move_type: Type) -> bool {
    // Note: is_grounded(defender) returns false if Levitate is active AND not negated.
    // However, if Iron Ball / Gravity is active, is_grounded returns true.
    // So if grounded, Levitate fails.
    // If not grounded, Levitate applies (and grants immunity to Ground).
    move_type == Type::Ground && !state.is_grounded(defender)
}

/// Flash Fire: Immune to Fire moves
pub fn flash_fire(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Fire
}

/// Volt Absorb: Immune to Electric moves
pub fn volt_absorb(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Electric
}

/// Water Absorb: Immune to Water moves
pub fn water_absorb(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Water
}

/// Storm Drain: Immune to Water moves
pub fn storm_drain(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Water
}

/// Lightning Rod: Immune to Electric moves
pub fn lightning_rod(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Electric
}

/// Sap Sipper: Immune to Grass moves
pub fn sap_sipper(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Grass
}

/// Motor Drive: Immune to Electric moves
pub fn motor_drive(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Electric
}

/// Dry Skin: Immune to Water moves
pub fn dry_skin(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Water
}

/// Earth Eater: Immune to Ground moves
pub fn earth_eater(_state: &BattleState, _defender: usize, move_type: Type) -> bool {
    move_type == Type::Ground
}

/// Levitate: Grounding check (returns Some(false) = ungrounded)
pub fn levitate_grounding(_state: &BattleState, _entity: usize) -> Option<bool> {
    Some(false)
}

/// Magic Guard: Immune to hazard damage (Stealth Rock, Spikes)
pub fn magic_guard_hazard_immunity(
    _state: &BattleState,
    _entity: usize,
    hazard: crate::state::Hazard,
) -> bool {
    // Magic Guard prevents indirect damage, but not status or stat drops.
    // So it blocks SR and Spikes damage, but not Toxic Spikes (status) or Sticky Web (speed).
    match hazard {
        crate::state::Hazard::StealthRock | crate::state::Hazard::Spikes => true,
        crate::state::Hazard::ToxicSpikes | crate::state::Hazard::StickyWeb => false,
    }
}

/// Limber: Immune to Paralysis
pub fn limber(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::PARALYSIS
}

/// Insomnia / Vital Spirit: Immune to Sleep
pub fn insomnia(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::SLEEP
}

/// Immunity: Immune to Poison/Toxic
pub fn immunity(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::POISON || status == Status::TOXIC
}

/// Magma Armor: Immune to Freeze
pub fn magma_armor(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::FREEZE
}

/// Water Veil: Immune to Burn
pub fn water_veil(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::BURN
}

/// Pastel Veil: Immune to Poison/Toxic
pub fn pastel_veil(_state: &BattleState, _entity: usize, status: Status) -> bool {
    status == Status::POISON || status == Status::TOXIC
}
