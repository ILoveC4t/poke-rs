use super::weather::{Terrain, Weather};
use crate::moves::{Move, MoveCategory, MoveId};
use crate::state::BattleState;
use crate::state::Hazard;
use crate::types::Type;

/// Called when a Pokemon switches in (after hazards)
pub type OnSwitchIn = fn(state: &mut BattleState, switched_idx: usize);

/// Called during turn ordering to modify move priority
pub type OnModifyPriority =
    fn(state: &BattleState, attacker: usize, move_id: MoveId, base_priority: i8) -> i8;

/// Called immediately before a move is executed
pub type OnBeforeMove = fn(state: &mut BattleState, attacker: usize, move_id: MoveId);

/// Called during damage calculation to modify damage (legacy, prefer new hooks)
pub type OnModifyDamage =
    fn(state: &BattleState, attacker: usize, defender: usize, damage: u16) -> u16;

/// Called after damage has been dealt
pub type OnAfterDamage = fn(state: &mut BattleState, attacker: usize, defender: usize, damage: u16);

/// Called when a stat boost is applied to modify the stage change
pub type OnStatChange = fn(change: i8) -> i8;

/// Called during base power calculation (Technician, Iron Fist, Tough Claws, etc.)
/// Uses &MoveData for full access to flags, type, category, and other properties.
/// `move_type` is the dynamically-resolved type (after type-changing effects like Judgment, Weather Ball).
pub type OnModifyBasePower = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    move_data: &Move,
    move_type: Type,
    bp: u16,
) -> u16;

/// Called during stat calculation to modify attack stat (Hustle, Pure Power, etc.)
pub type OnModifyAttack = fn(
    state: &BattleState,
    attacker: usize,
    move_id: crate::moves::MoveId,
    category: MoveCategory,
    attack: u16,
) -> u16;

/// Called during stat calculation to modify defense stat (Marvel Scale, etc.)
pub type OnModifyDefense = fn(
    state: &BattleState,
    defender: usize,
    attacker: usize,
    category: MoveCategory,
    defense: u16,
) -> u16;

/// Attacker's post-damage modifier (Tinted Lens, Sniper, etc.)
/// Applied BEFORE defender modifiers in the damage chain.
pub type OnAttackerFinalMod = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    effectiveness: u8,
    is_crit: bool,
    damage: u32,
) -> u32;

/// Defender's damage reduction (Multiscale, Filter, Fluffy, Ice Scales, etc.)
/// Applied AFTER attacker modifiers in the damage chain.
pub type OnDefenderFinalMod = fn(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    effectiveness: u8,
    move_type: Type,
    category: MoveCategory,
    is_contact: bool,
    damage: u32,
) -> u32;

/// Called to check type immunity (Levitate, Flash Fire, etc.)
/// Returns true if the ability grants immunity.
pub type OnTypeImmunity = fn(state: &BattleState, defender: usize, move_type: Type) -> bool;

/// Called during speed calculation to modify effective speed.
/// Used by Chlorophyll, Swift Swim, Sand Rush, Slush Rush, Surge Surfer.
pub type OnModifySpeed = fn(state: &BattleState, entity: usize, speed: u16) -> u16;

/// Called to check if an entity is grounded (can override default logic)
/// Returns Some(true/false) to override, None to use default calculation
pub type OnCheckGrounded = fn(state: &BattleState, entity: usize) -> Option<bool>;

/// Called to check hazard immunity (Magic Guard, Heavy-Duty Boots, etc.)
/// Returns true if the entity is immune to entry hazards matches
pub type OnHazardImmunity = fn(state: &BattleState, entity: usize, hazard: Hazard) -> bool;

/// Called to check if status damage reduction should be ignored (e.g. Guts ignoring Burn attack drop)
pub type OnIgnoreStatusDamageReduction =
    fn(state: &BattleState, entity: usize, status: crate::state::Status) -> bool;

/// Called when checking for status immunity.
/// Returns true if the Pokémon is immune to the status.
pub type OnStatusImmunity =
    fn(state: &BattleState, entity: usize, status: crate::state::Status) -> bool;

// ============================================================================
// AbilityHooks Struct
// ============================================================================

#[derive(Clone, Copy, Default)]
pub struct AbilityHooks {
    // Existing hooks
    pub on_switch_in: Option<OnSwitchIn>,
    pub on_modify_priority: Option<OnModifyPriority>,
    pub on_before_move: Option<OnBeforeMove>,
    pub on_modify_damage: Option<OnModifyDamage>,
    pub on_after_damage: Option<OnAfterDamage>,
    pub on_stat_change: Option<OnStatChange>,
    // New damage-phase hooks
    pub on_modify_base_power: Option<OnModifyBasePower>,
    pub on_modify_attack: Option<OnModifyAttack>,
    pub on_modify_defense: Option<OnModifyDefense>,
    pub on_attacker_final_mod: Option<OnAttackerFinalMod>,
    pub on_defender_final_mod: Option<OnDefenderFinalMod>,
    pub on_type_immunity: Option<OnTypeImmunity>,
    // Speed/grounding hooks
    pub on_modify_speed: Option<OnModifySpeed>,
    pub on_check_grounded: Option<OnCheckGrounded>,
    pub on_hazard_immunity: Option<OnHazardImmunity>,
    pub on_ignore_status_damage_reduction: Option<OnIgnoreStatusDamageReduction>,
    pub on_status_immunity: Option<OnStatusImmunity>,
}

impl AbilityHooks {
    /// Empty hooks (default)
    pub const NONE: Self = Self {
        on_switch_in: None,
        on_modify_priority: None,
        on_before_move: None,
        on_modify_damage: None,
        on_after_damage: None,
        on_stat_change: None,
        on_modify_base_power: None,
        on_modify_attack: None,
        on_modify_defense: None,
        on_attacker_final_mod: None,
        on_defender_final_mod: None,
        on_type_immunity: None,
        on_modify_speed: None,
        on_check_grounded: None,
        on_hazard_immunity: None,
        on_ignore_status_damage_reduction: None,
        on_status_immunity: None,
    };

    /// Helper to set weather
    pub fn set_weather(state: &mut BattleState, weather: Weather, turns: u8) {
        state.weather = weather as u8;
        state.weather_turns = turns;
    }

    /// Helper to set terrain
    pub fn set_terrain(state: &mut BattleState, terrain: Terrain, turns: u8) {
        state.terrain = terrain as u8;
        state.terrain_turns = turns;
    }
}
