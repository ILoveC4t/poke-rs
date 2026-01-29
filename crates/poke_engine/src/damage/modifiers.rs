//! Damage modifier pipeline.
//!
//! This module contains functions that modify damage at various stages
//! of the calculation. Each function is a discrete step in the pipeline.

use super::context::DamageContext;
use super::formula::{apply_boost, apply_modifier, apply_modifier_floor, of16, of32, pokeround};
use super::generations::{GenMechanics, Terrain, Weather};
use super::Modifier;
use crate::abilities::{AbilityId, ABILITY_REGISTRY};
use crate::items::{ItemId, ITEM_REGISTRY};
use crate::modifier;
use crate::moves::{MoveCategory, MoveFlags, MoveId, MOVE_REGISTRY};
use crate::state::{BattleState, Status};

// ============================================================================
// Stat Indices
// ============================================================================

/// Stat array indices for BattleState.stats
const STAT_INDEX_HP: usize = 0;
const STAT_INDEX_ATTACK: usize = 1;
const STAT_INDEX_DEFENSE: usize = 2;
const STAT_INDEX_SP_ATTACK: usize = 3;
const STAT_INDEX_SP_DEFENSE: usize = 4;
const STAT_INDEX_SPEED: usize = 5;

/// Boost array indices for BattleState.boosts
const BOOST_INDEX_ATTACK: usize = 0;
const BOOST_INDEX_DEFENSE: usize = 1;
const BOOST_INDEX_SP_ATTACK: usize = 2;
const BOOST_INDEX_SP_DEFENSE: usize = 3;
const BOOST_INDEX_SPEED: usize = 4;

// ============================================================================
// Screen-Breaking Move Detection
// ============================================================================

/// Screen-breaking moves ignore Reflect/Light Screen/Aurora Veil.
/// These moves break screens after dealing damage, but the damage itself is not reduced.
fn is_screen_breaker(move_id: MoveId) -> bool {
    move_id.data().flags.contains(MoveFlags::BREAKS_SCREENS)
}

/// Check if attacker has Mold Breaker or variants (Teravolt, Turboblaze).
/// These abilities bypass the target's defensive abilities.
fn has_mold_breaker(ability: AbilityId) -> bool {
    ability
        .flags()
        .contains(crate::abilities::AbilityFlags::MOLD_BREAKER)
}

// ============================================================================
// Item Hook Helpers
// ============================================================================

/// Call the OnModifyBasePower hook for the attacker's item, if registered.
fn call_item_base_power_hook<G: GenMechanics>(ctx: &DamageContext<'_, G>, bp: u16) -> u16 {
    let attacker_item = ctx.state.items[ctx.attacker];
    if let Some(Some(hooks)) = ITEM_REGISTRY.get(attacker_item as usize) {
        if let Some(hook) = hooks.on_modify_base_power {
            return hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.move_data,
                ctx.move_type,
                bp,
            );
        }
    }
    bp
}

/// Apply item final modifiers (attacker items like Life Orb, Expert Belt).
fn apply_item_final_mods<G: GenMechanics>(ctx: &DamageContext<'_, G>, mut damage: u32) -> u32 {
    let attacker_item = ctx.state.items[ctx.attacker];
    if let Some(Some(hooks)) = ITEM_REGISTRY.get(attacker_item as usize) {
        if let Some(hook) = hooks.on_attacker_final_mod {
            damage = hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.effectiveness,
                ctx.is_crit,
                damage,
            );
        }
    }
    damage
}

// ============================================================================
// Move Hook Helpers
// ============================================================================

/// Apply move hook conditional modifiers (Knock Off, Venoshock, Hex, Brine, etc.).
fn call_move_base_power_hook<G: GenMechanics>(ctx: &DamageContext<'_, G>, bp: u16) -> u16 {
    if let Some(Some(hooks)) = MOVE_REGISTRY.get(ctx.move_id as usize) {
        // Check conditional multiplier
        if let Some(condition) = hooks.on_base_power_condition {
            if condition(ctx.state, ctx.attacker, ctx.defender, ctx.move_data) {
                let multiplier = super::Modifier::new(hooks.conditional_multiplier);
                return apply_modifier(bp as u32, multiplier).max(1) as u16;
            }
        }
        // Check custom base power modifier
        if let Some(hook) = hooks.on_modify_base_power {
            return hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.move_data,
                ctx.move_type,
                bp,
            );
        }
    }
    bp
}

// ============================================================================
// Ability Hook Helpers
// ============================================================================

/// Call the OnModifyBasePower hook for the attacker's ability, if registered.
fn call_base_power_hook<G: GenMechanics>(ctx: &DamageContext<'_, G>, bp: u16) -> u16 {
    if let Some(Some(hooks)) = ABILITY_REGISTRY.get(ctx.attacker_ability as usize) {
        if let Some(hook) = hooks.on_modify_base_power {
            return hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.move_data,
                ctx.move_type,
                bp,
            );
        }
    }
    bp
}

/// Call the OnModifyAttack hook for the attacker's ability, if registered.
fn call_attack_hook<G: GenMechanics>(ctx: &DamageContext<'_, G>, attack: u16) -> u16 {
    if let Some(Some(hooks)) = ABILITY_REGISTRY.get(ctx.attacker_ability as usize) {
        if let Some(hook) = hooks.on_modify_attack {
            return hook(ctx.state, ctx.attacker, ctx.move_id, ctx.category, attack);
        }
    }
    attack
}

/// Call the OnModifyDefense hook for the defender's ability, if registered.
/// Bypassed by Mold Breaker, Teravolt, and Turboblaze.
fn call_defense_hook<G: GenMechanics>(ctx: &DamageContext<'_, G>, defense: u16) -> u16 {
    // Mold Breaker bypasses defender's defensive ability hooks
    if has_mold_breaker(ctx.attacker_ability) {
        return defense;
    }

    let defender_ability = ctx.state.abilities[ctx.defender];
    if let Some(Some(hooks)) = ABILITY_REGISTRY.get(defender_ability as usize) {
        if let Some(hook) = hooks.on_modify_defense {
            return hook(ctx.state, ctx.defender, ctx.attacker, ctx.category, defense);
        }
    }
    defense
}

/// Check if status damage reduction should be ignored (Guts, Facade).
fn should_ignore_status_damage_reduction<G: GenMechanics>(
    ctx: &DamageContext<'_, G>,
    status: Status,
) -> bool {
    // Check ability (Guts)
    if let Some(Some(hooks)) = ABILITY_REGISTRY.get(ctx.attacker_ability as usize) {
        if let Some(hook) = hooks.on_ignore_status_damage_reduction {
            if hook(ctx.state, ctx.attacker, status) {
                return true;
            }
        }
    }

    // Check move (Facade)
    if let Some(Some(hooks)) = MOVE_REGISTRY.get(ctx.move_id as usize) {
        if let Some(hook) = hooks.on_ignore_status_damage_reduction {
            if hook(ctx.state, ctx.attacker, status) {
                return true;
            }
        }
    }

    false
}
/// Order: attacker mods first, then defender mods (per Smogon order).
fn apply_final_mods<G: GenMechanics>(
    ctx: &DamageContext<'_, G>,
    mut damage: u32,
    attacker_hooks: Option<&crate::abilities::AbilityHooks>,
    defender_hooks: Option<&crate::abilities::AbilityHooks>,
) -> u32 {
    // Attacker's ability (Tinted Lens, Sniper)
    if let Some(hooks) = attacker_hooks {
        if let Some(hook) = hooks.on_attacker_final_mod {
            damage = hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.effectiveness,
                ctx.is_crit,
                damage,
            );
        }
    }

    // Defender's ability (Multiscale, Filter, Fluffy)
    // Bypassed by Mold Breaker, Teravolt, and Turboblaze
    if !has_mold_breaker(ctx.attacker_ability) {
        if let Some(hooks) = defender_hooks {
            if let Some(hook) = hooks.on_defender_final_mod {
                let is_contact = ctx.move_data.flags.contains(MoveFlags::CONTACT);
                damage = hook(
                    ctx.state,
                    ctx.attacker,
                    ctx.defender,
                    ctx.effectiveness,
                    ctx.move_type,
                    ctx.category,
                    is_contact,
                    damage,
                );
            }
        }
    }

    damage
}

// ============================================================================
// Phase 1: Base Power Computation
// ============================================================================

/// Compute the effective base power after ability and item modifiers.
pub fn compute_base_power<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    // 1. Apply special move overrides (Weight, HP, Status based)
    // This replaces the old inline logic for Grass Knot, Eruption, Facade, etc.
    let mut bp = super::special_moves::modify_base_power(ctx);

    // ========================================================================
    // Move-based BP modifiers via hook system
    // ========================================================================

    // Knock Off (1.5x if target has removable item, Gen 6+ only)
    // Venoshock (2x if poisoned), Hex (2x if statused), Brine (2x if below 50% HP)
    // Variable Power Moves (Low Kick, Grass Knot, etc.) must be calculated FIRST
    // so that Ability modifiers (Technician) see the correct base power.
    if ctx.gen.generation() >= 6 || ctx.move_id != MoveId::Knockoff {
        bp = call_move_base_power_hook(ctx, bp as u16) as u32;
    }

    // ========================================================================
    // Ability-based BP modifiers via hook system
    // ========================================================================

    // Call registered OnModifyBasePower hook if available
    bp = call_base_power_hook(ctx, bp as u16) as u32;

    // ========================================================================
    // Item-based BP modifiers via hook system
    // ========================================================================

    bp = call_item_base_power_hook(ctx, bp as u16) as u32;

    // ========================================================================
    // Terrain-based BP modifiers (Gen 6+)
    // ========================================================================
    // Smogon applies terrain as a base power modifier (bpMods), not damage.
    // Electric/Grassy/Psychic boost matching types, Misty halves Dragon.
    // Grassy Terrain also halves Earthquake/Bulldoze/Magnitude.
    apply_terrain_mod_bp(ctx, &mut bp);

    // TODO: Parental Bond ability: Multi-hit (2 hits), second hit at 0.25x power (Gen 7+)
    //       Requires special handling in damage pipeline to return combined damage

    // TODO: Other conditional power moves that weren't in modify_base_power yet:
    // - Assurance (2x if target was hit this turn)
    // - Payback (2x if target moved first)
    // - Avalanche / Revenge (2x if hit by target this turn)
    // - Stored Power / Power Trip (20 + 20 per boost)
    // - Punishment (inverse of target's boosts)
    // - Electro Ball (speed ratio)
    // - Gyro Ball (inverse speed ratio)
    // - Foul Play (uses target's Atk)

    // Note: Weather is applied to base damage (not BP) in formula.rs

    ctx.base_power = bp.min(u16::MAX as u32) as u16;
}

// ============================================================================
// Phase 2: Effective Stats
// ============================================================================

/// Compute effective attack and defense stats.
///
/// This accounts for:
/// - Stat boosts (or ignoring them for crits)
/// - Abilities that modify stats
/// - Items that modify stats
///
/// Returns (attack, defense).
pub fn compute_effective_stats<G: GenMechanics>(ctx: &DamageContext<'_, G>) -> (u16, u16) {
    let (mut atk_idx, mut def_idx) = ctx.get_stat_indices();

    // ========================================================================
    // Special Move Logic: Stat Swaps (Body Press, Psyshock, Foul Play)
    // ========================================================================

    // Body Press: Use Defense as Attack
    if ctx.move_id == MoveId::Bodypress {
        atk_idx = STAT_INDEX_DEFENSE;
    }

    // Psyshock / Psystrike / Secret Sword: Use Defense as target Defense (even if special)
    if matches!(
        ctx.move_id,
        MoveId::Psyshock | MoveId::Psystrike | MoveId::Secretsword
    ) {
        def_idx = STAT_INDEX_DEFENSE;
    }

    // Foul Play: Use Target's Attack
    let use_target_atk = ctx.move_id == MoveId::Foulplay;
    let atk_source_idx = if use_target_atk {
        ctx.defender
    } else {
        ctx.attacker
    };

    let mut attack = ctx.state.stats[atk_source_idx][atk_idx];
    let mut defense = ctx.state.stats[ctx.defender][def_idx];

    // Get boost stages
    // Map stat index to boost index
    let atk_boost_idx = match atk_idx {
        STAT_INDEX_ATTACK => BOOST_INDEX_ATTACK,
        STAT_INDEX_DEFENSE => BOOST_INDEX_DEFENSE,
        STAT_INDEX_SP_ATTACK => BOOST_INDEX_SP_ATTACK,
        _ => BOOST_INDEX_ATTACK, // Fallback
    };

    let def_boost_idx = if def_idx == STAT_INDEX_DEFENSE {
        BOOST_INDEX_DEFENSE
    } else {
        BOOST_INDEX_SP_DEFENSE
    };

    let atk_boost = ctx.state.boosts[atk_source_idx][atk_boost_idx];
    let def_boost = ctx.state.boosts[ctx.defender][def_boost_idx];

    // Critical hit rules:
    // - Ignore attacker's negative offensive boosts
    // - Ignore defender's positive defensive boosts
    if ctx.is_crit {
        if atk_boost > 0 {
            attack = apply_boost(attack, atk_boost);
        }
        // Ignore positive defense boosts (use base)
        if def_boost < 0 {
            defense = apply_boost(defense, def_boost);
        }
    } else {
        attack = apply_boost(attack, atk_boost);
        defense = apply_boost(defense, def_boost);
    }

    // Gen 3-4: Explosion and Self-Destruct halve the target's Defense
    // This was removed in Gen 5+
    if G::GEN <= 4 && matches!(ctx.move_id, MoveId::Explosion | MoveId::Selfdestruct) {
        defense = defense / 2;
    }

    // Ability modifiers for attack (via hook system)
    if ctx.gen.has_abilities() {
        attack = call_attack_hook(ctx, attack);
        defense = call_defense_hook(ctx, defense);
    }

    // Item modifiers
    let attacker_item = ctx.state.items[ctx.attacker];
    let defender_item = ctx.state.items[ctx.defender];

    // Attacker item attack modifiers
    if let Some(Some(hooks)) = ITEM_REGISTRY.get(attacker_item as usize) {
        if let Some(hook) = hooks.on_modify_attack {
            attack = hook(ctx.state, ctx.attacker, ctx.category, attack);
        }
    }

    // Defender item defense modifiers
    if let Some(Some(hooks)) = ITEM_REGISTRY.get(defender_item as usize) {
        if let Some(hook) = hooks.on_modify_defense {
            defense = hook(ctx.state, ctx.defender, ctx.attacker, ctx.category, defense);
        }
    }

    (attack.max(1), defense.max(1))
}

// ============================================================================
// Phase 3: Pre-Random Modifiers
// ============================================================================

/// Apply spread move modifier (0.75x for hitting multiple targets).
///
/// Applied directly to base_damage using pokeRound.
pub fn apply_spread_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    if ctx.is_spread {
        // pokeRound(OF32(baseDamage * 3072) / 4096)
        *base_damage = apply_modifier(*base_damage, modifier!(0.75)); // 0.75x
    }
}

/// Check if weather is suppressed by any active ability (Cloud Nine, Air Lock).
fn is_weather_suppressed(state: &BattleState) -> bool {
    // Check both active Pokémon (Singles assumption for now)
    for &idx in &state.active {
        let ability = state.abilities[idx as usize];
        if ability
            .flags()
            .contains(crate::abilities::AbilityFlags::SUPPRESSES_WEATHER)
        {
            return true;
        }
    }
    false
}

/// Apply weather modifier to base damage.
///
/// Weather boost (1.5x for Fire in Sun, Water in Rain) is applied to base damage
/// in all generations. This matches Smogon's implementation.
pub fn apply_weather_mod_damage<G: GenMechanics>(
    ctx: &mut DamageContext<'_, G>,
    base_damage: &mut u32,
) {
    // Check suppression
    if is_weather_suppressed(ctx.state) {
        return;
    }

    let weather = Weather::from_u8(ctx.state.weather);
    if let Some(modifier) = ctx.gen.weather_modifier(weather, ctx.move_type) {
        *base_damage = apply_modifier(*base_damage, modifier);
    }
}

/// Apply weather modifier to base power (Gen 5+).
pub fn apply_weather_mod_bp<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, bp: &mut u32) {
    if ctx.gen.generation() < 5 {
        return; // Applied to damage in Gen 2-4
    }

    // Check suppression
    if is_weather_suppressed(ctx.state) {
        return;
    }

    let weather = Weather::from_u8(ctx.state.weather);
    if let Some(modifier) = ctx.gen.weather_modifier(weather, ctx.move_type) {
        *bp = apply_modifier(*bp, modifier);
    }
}

/// Apply terrain modifier to base power (Gen 6+).
///
/// Smogon applies terrain as a base power modifier (bpMods), not a final damage modifier.
/// This affects:
/// - Electric/Grassy/Psychic Terrain: boost matching types when attacker is grounded
/// - Misty Terrain: halves Dragon moves when defender is grounded
/// - Grassy Terrain: halves Earthquake/Bulldoze/Magnitude when defender is grounded
pub fn apply_terrain_mod_bp<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, bp: &mut u32) {
    let terrain = Terrain::from_u8(ctx.state.terrain);

    if let Some(modifier) = ctx.gen.terrain_modifier(
        terrain,
        ctx.move_id,
        ctx.move_type,
        ctx.attacker_grounded,
        ctx.defender_grounded,
    ) {
        *bp = apply_modifier(*bp, modifier);
    }
}

/// Apply terrain modifier (DEPRECATED - kept for reference).
/// This should NOT be used; terrain is applied to base power, not damage.
#[allow(dead_code)]
pub fn apply_terrain_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    let terrain = Terrain::from_u8(ctx.state.terrain);

    // Terrain affects:
    // 1. Attacker damage boost (Electric/Grass/Psychic) if attacker grounded
    // 2. Damage reduction (Misty/Grassy) if defender grounded
    if let Some(modifier) = ctx.gen.terrain_modifier(
        terrain,
        ctx.move_id,
        ctx.move_type,
        ctx.attacker_grounded,
        ctx.defender_grounded,
    ) {
        *base_damage = apply_modifier(*base_damage, modifier);
    }
}

/// Apply burn modifier (Gen 3-4 only, applied before screens/+2/crit).
///
/// Gen 3-4: Burn is applied early in the damage pipeline.
/// Gen 5+: Burn is applied after random/STAB/effectiveness in compute_final_damage.
pub fn apply_burn_mod_early<G: GenMechanics>(ctx: &DamageContext<'_, G>, base_damage: &mut u32) {
    // Only for Gen 3-4 (uses_4096_scale_modifiers returns false)
    if ctx.gen.uses_4096_scale_modifiers() {
        return;
    }

    if ctx.is_burned()
        && ctx.category == MoveCategory::Physical
        && !should_ignore_status_damage_reduction(ctx, Status::BURN)
    {
        *base_damage = *base_damage / 2;
    }
}

/// Apply screen modifier (Gen 3-4 only, applied before weather/+2/crit).
///
/// Gen 3-4: Screens are applied early in the damage pipeline.
/// Gen 5+: Screens are applied after random/STAB/effectiveness in compute_final_damage.
pub fn apply_screen_mod_early<G: GenMechanics>(ctx: &DamageContext<'_, G>, base_damage: &mut u32) {
    // Only for Gen 3-4 (uses_4096_scale_modifiers returns false)
    if ctx.gen.uses_4096_scale_modifiers() {
        return;
    }

    if ctx.is_crit || is_screen_breaker(ctx.move_id) {
        return;
    }

    if ctx.has_screen(ctx.category == MoveCategory::Physical) {
        // Gen 3-4: simple floor(damage * 0.5) for singles, floor(damage * 2/3) for doubles
        if ctx.state.is_doubles() {
            *base_damage = *base_damage * 2 / 3;
        } else {
            *base_damage = *base_damage / 2;
        }
    }
}

/// Apply critical hit modifier.
///
/// Note: In smogon's implementation, crit uses direct floor(x * 1.5),
/// NOT the 4096-scale system. This is applied during base damage phase.
/// Gen 3: crit doubles (floor(x * 2))
/// Gen 4-5: crit doubles (floor(x * 2))
/// Gen 6+: crit is 1.5x (floor(x * 1.5))
pub fn apply_crit_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    if ctx.is_crit {
        let crit_mult = ctx.gen.crit_multiplier();
        // Use floor division for crit, not 4096-scale
        if crit_mult == Modifier::DOUBLE {
            *base_damage = *base_damage * 2;
        } else {
            // Gen 6+: 1.5x
            *base_damage = apply_modifier_floor(*base_damage, 3, 2);
        }
    }
}

/// Apply move-specific final damage modifiers (after crit, before random rolls).
///
/// This is used for gen-specific mechanics like Gen 3 Weather Ball damage doubling.
pub fn apply_move_final_damage_mod<G: GenMechanics>(
    ctx: &DamageContext<'_, G>,
    base_damage: &mut u32,
) {
    if let Some(Some(hooks)) = MOVE_REGISTRY.get(ctx.move_id as usize) {
        if let Some(hook) = hooks.on_modify_final_damage {
            *base_damage = hook(
                ctx.state,
                ctx.attacker,
                ctx.defender,
                ctx.move_data,
                *base_damage,
            );
        }
    }
}

// ============================================================================
// Phase 4: Final Damage Computation
// ============================================================================

/// Compute final damage for all 16 random rolls.
///
/// This function delegates to the generation's pipeline for the actual order
/// of operations. See `crate::damage::pipeline` for implementation details.
///
/// The ordering differs by generation:
///
/// Gen 3: STAB → effectiveness → ability → random (LAST)
/// Gen 4: random → STAB → effectiveness → item → ability
/// Gen 5+: random → STAB → effectiveness → burn → screens → item → ability
///
/// Note: Weather, spread, and crit are applied to base_damage BEFORE
/// this function is called.
pub fn compute_final_damage<G: GenMechanics>(
    ctx: &DamageContext<'_, G>,
    base_damage: u32,
) -> [u16; 16] {
    // Type immunity check
    if ctx.effectiveness == 0 {
        return [0u16; 16];
    }

    // Get ability hooks for final modifiers
    let attacker_hooks = ABILITY_REGISTRY
        .get(ctx.attacker_ability as usize)
        .and_then(|a| a.as_ref());
    let defender_hooks = ABILITY_REGISTRY
        .get(ctx.defender_ability as usize)
        .and_then(|a| a.as_ref());

    // Prepare pipeline inputs
    let stab_mod = if ctx.has_stab {
        ctx.gen
            .stab_multiplier(ctx.has_adaptability, ctx.is_tera_stab)
    } else {
        Modifier::ONE
    };

    let is_burned = ctx.is_burned();
    let ignore_burn = should_ignore_status_damage_reduction(ctx, Status::BURN);

    let has_screen = !ctx.is_crit
        && !is_screen_breaker(ctx.move_id)
        && ctx.has_screen(ctx.category == MoveCategory::Physical);
    let screen_mod = if has_screen {
        Modifier::new(ctx.state.get_screen_modifier(ctx.defender, ctx.category))
    } else {
        Modifier::ONE
    };

    // Create closures for item and ability final mods
    // These capture the context and hooks needed
    let item_final_mod = |damage: u32| -> u32 { apply_item_final_mods(ctx, damage) };

    let ability_final_mod =
        |damage: u32| -> u32 { apply_final_mods(ctx, damage, attacker_hooks, defender_hooks) };

    // Delegate to the generation's pipeline
    ctx.gen.pipeline().compute_final_damage(
        base_damage,
        ctx.effectiveness,
        ctx.has_stab,
        stab_mod,
        ctx.is_crit,
        is_burned,
        ignore_burn,
        ctx.category,
        has_screen,
        screen_mod,
        &item_final_mod,
        &ability_final_mod,
    )
}

impl<G: GenMechanics> DamageContext<'_, G> {
    /// Apply a modifier directly to a damage value (for post-random mods).
    #[allow(dead_code)]
    fn apply_mod_to(&self, damage: &mut u32, modifier: Modifier) {
        *damage = apply_modifier(*damage, modifier);
    }
}

// ============================================================================
// Item Damage Modifiers (Helpers)
// ============================================================================

/// Check if an item is a type-boosting item for the given type.
#[allow(dead_code)]
// get_type_boost_item_mod removed (migrated to item hooks)

/// Check if attacker has a contact-based ability modifier.
#[allow(dead_code)]
fn has_contact_ability_boost(ability: AbilityId, move_flags: MoveFlags) -> Option<Modifier> {
    if !move_flags.contains(MoveFlags::CONTACT) {
        return None;
    }

    match ability {
        AbilityId::Toughclaws => Some(Modifier::ONE_POINT_THREE), // 1.3x
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        damage::{DamageContext, Gen9},
        items::ItemId,
        moves::{MoveCategory, MoveId},
        species::SpeciesId,
        state::{BattleState, Status},
        types::Type,
    };

    #[test]
    fn test_stat_modifying_items() {
        let mut state = BattleState::new();
        let gen = Gen9;

        // Attacker
        state.stats[0][1] = 100; // Atk
        state.stats[0][3] = 100; // SpA

        // Defender
        state.stats[6][2] = 100; // Def
        state.stats[6][4] = 100; // SpD

        // 1. Assault Vest (1.5x SpD)
        state.items[6] = ItemId::Assaultvest;
        let special_move = MoveId::Surf; // Special
        let ctx = DamageContext::new(gen, &state, 0, 6, special_move, false);
        let (_, def) = compute_effective_stats(&ctx);
        assert_eq!(def, 150, "Assault Vest should boost Sp. Defense by 1.5x");

        // 2. Eviolite (1.5x Def/SpD for pre-evo)
        state.items[6] = ItemId::Eviolite;
        state.species[6] = SpeciesId::from_str("chansey").unwrap(); // Can evolve
        let physical_move = MoveId::Tackle; // Physical
        let ctx_phys = DamageContext::new(gen, &state, 0, 6, physical_move, false);
        let (_, def_phys) = compute_effective_stats(&ctx_phys);
        assert_eq!(
            def_phys, 150,
            "Eviolite should boost Defense by 1.5x for Chansey"
        );

        let ctx_spec = DamageContext::new(gen, &state, 0, 6, special_move, false);
        let (_, def_spec) = compute_effective_stats(&ctx_spec);
        assert_eq!(
            def_spec, 150,
            "Eviolite should boost Sp. Defense by 1.5x for Chansey"
        );

        // 3. Thick Club (2x Atk for Cubone/Marowak)
        // NOTE: Thick Club item is filtered out in build.rs as nonstandard,
        // but the implementation and test are maintained for completeness
        state.items[0] = ItemId::None; // Use None since Thickclub is not available
        state.species[0] = SpeciesId::from_str("cubone").unwrap();
        // Skipping actual test since ItemId::Thickclub doesn't exist
        // let ctx_club = DamageContext::new(gen, &state, 0, 6, physical_move, false);
        // let (atk_club, _) = compute_effective_stats(&ctx_club);
        // assert_eq!(atk_club, 200, "Thick Club should double Attack for Cubone");

        // 4. Light Ball (2x Atk/SpA for Pikachu)
        state.items[0] = ItemId::Lightball;
        state.species[0] = SpeciesId::from_str("pikachu").unwrap();
        let ctx_light_phys = DamageContext::new(gen, &state, 0, 6, physical_move, false);
        let (atk_light_phys, _) = compute_effective_stats(&ctx_light_phys);
        assert_eq!(
            atk_light_phys, 200,
            "Light Ball should double Attack for Pikachu"
        );

        let ctx_light_spec = DamageContext::new(gen, &state, 0, 6, special_move, false);
        let (atk_light_spec, _) = compute_effective_stats(&ctx_light_spec);
        assert_eq!(
            atk_light_spec, 200,
            "Light Ball should double Sp. Attack for Pikachu"
        );
    }

    #[test]
    fn test_facade_damage() {
        use crate::damage::{DamageContext, Gen9};
        use crate::species::SpeciesId;
        use crate::state::{BattleState, Status};
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup attacker (index 0) and defender (index 6)
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100; // 100 Atk

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100; // 100 Def

        let move_id = MoveId::Facade;

        // Case 1: No status
        {
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(ctx.base_power, 70, "Facade BP should be 70 without status");
        }

        // Case 2: Burned
        {
            state.status[0] = Status::BURN;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(ctx.base_power, 140, "Facade BP should double when burned");

            // Verify burn reduction is ignored
            let rolls = compute_final_damage(&ctx, 100);
            let min_damage = rolls[0];

            assert!(
                min_damage > 100,
                "Facade should ignore burn reduction (got {})",
                min_damage
            );
        }

        // Case 3: Poisoned
        {
            state.status[0] = Status::POISON;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(ctx.base_power, 140, "Facade BP should double when poisoned");
        }

        // Case 4: Paralyzed
        {
            state.status[0] = Status::PARALYSIS;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 140,
                "Facade BP should double when paralyzed"
            );
        }

        // Case 5: Asleep (should NOT double)
        {
            state.status[0] = Status::SLEEP;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 70,
                "Facade BP should NOT double when asleep"
            );
        }
    }

    #[test]
    fn test_item_modifiers() {
        use crate::damage::{DamageContext, Gen9};
        use crate::items::ItemId;
        use crate::moves::{MoveCategory, MoveId};
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100, SpA 100, Def 100, SpD 100
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100; // Atk
        state.stats[0][3] = 100; // SpA

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100; // Def
        state.stats[6][4] = 100; // SpD

        // 1. Choice Band (1.5x Atk)
        {
            state.items[0] = ItemId::Choiceband;
            let move_id = MoveId::Tackle; // Physical
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            let (atk, _) = compute_effective_stats(&ctx);
            // 100 * 1.5 = 150
            assert_eq!(atk, 150, "Choice Band should boost Attack by 1.5x");
        }

        // 2. Choice Specs (1.5x SpA)
        {
            state.items[0] = ItemId::Choicespecs;
            // Use a Special move (e.g. Swift or similar). Assuming Ember exists and is special.
            // Or just mock the category if possible, but DamageContext derives it from MoveId.
            // MoveId::Swift is usually Special.
            let move_id = MoveId::Swift;
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            // Verify category is Special (just in case)
            if ctx.category == MoveCategory::Special {
                let (atk, _) = compute_effective_stats(&ctx);
                assert_eq!(atk, 150, "Choice Specs should boost Sp. Attack by 1.5x");
            }
        }

        // 3. Life Orb (1.3x damage)
        {
            state.items[0] = ItemId::Lifeorb;
            let move_id = MoveId::Tackle;
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            // Base damage 100.
            // Life Orb: 100 * 5324 / 4096 = 129.98 -> 129 or 130
            // apply_modifier(100, 5324) -> (100*5324 + 2048) >> 12 = 534448 >> 12 = 130.48 -> 130.
            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0]; // min roll (random=85)
                                   // Expected: min roll (85) with STAB and Life Orb applied.
            assert_eq!(
                damage, 165,
                "Life Orb should boost damage by ~1.3x (with STAB)"
            );
        }

        // 4. Expert Belt (1.2x on Super Effective)
        {
            state.items[0] = ItemId::Expertbelt;
            let move_id = MoveId::Karatechop; // Fighting type
                                              // Target is Normal (weak to Fighting)

            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert!(ctx.effectiveness > 4, "Move should be super effective");

            // Random roll 85.
            // SE (2x): 85 * 2 = 170.
            // Expert Belt (1.2x): 170 * 4915 / 4096 = 204.

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(
                damage, 204,
                "Expert Belt should boost super effective damage by ~1.2x"
            );

            // Neutral hit check
            state.types[6] = [Type::Fighting, Type::Fighting]; // Resists? No, Fighting vs Fighting is neutral?
                                                               // Fighting vs Poison is 0.5x. Fighting vs Flying is 0.5x.
                                                               // Fighting vs Bug is 0.5x.
                                                               // Fighting vs Fighting is 1x.

            let ctx_neutral = DamageContext::new(gen, &state, 0, 6, move_id, false);
            // 1x effectiveness.
            // Random 85.
            // No boost: 85.

            let rolls_neutral = compute_final_damage(&ctx_neutral, 100);
            let damage_neutral = rolls_neutral[0];

            assert_eq!(
                damage_neutral, 85,
                "Expert Belt should NOT boost neutral damage"
            );
        }

        // 5. Charcoal (1.2x Fire moves)
        {
            state.items[0] = ItemId::Charcoal;
            let move_id = MoveId::Ember; // Fire
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            // Base Power 40.
            // Charcoal: 40 * 1.2 = 48.

            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 48,
                "Charcoal should boost Fire move BP by 1.2x"
            );

            // Non-Fire move
            let move_id_normal = MoveId::Tackle;
            let mut ctx_normal = DamageContext::new(gen, &state, 0, 6, move_id_normal, false);
            // BP 40 (Tackle is 40 in recent gens? Or 50?)
            // Tackle is 40 in Gen 9.

            let original_bp = ctx_normal.move_data.power;
            compute_base_power(&mut ctx_normal);
            assert_eq!(
                ctx_normal.base_power, original_bp,
                "Charcoal should NOT boost Normal move BP"
            );
        }
    }

    #[test]
    fn test_tinted_lens() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100, Def 100
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100; // Atk

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Rock, Type::Rock]; // Rock resists Normal
        state.stats[6][2] = 100; // Def

        state.abilities[0] = AbilityId::Tintedlens;

        let move_id = MoveId::Tackle; // Normal type

        // Case 1: Not very effective (0.5x)
        {
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert_eq!(
                ctx.effectiveness, 2,
                "Normal vs Rock should be 0.5x (effectiveness 2)"
            );

            // Base damage 100 passed to function
            // 1. Roll 85: 85
            // 2. STAB (1.5x): 85 * 1.5 = 127.5 -> 127 (pokeround rounds 0.5 down)
            // 3. Effectiveness (0.5x): 127 * 2 / 4 = 63.5 -> 63
            // 4. Tinted Lens (2x): 63 * 2 = 126

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0]; // min roll (85)

            assert_eq!(
                damage, 126,
                "Tinted Lens should double damage for not very effective hits"
            );
        }

        // Case 2: Neutral hit (should NOT boost)
        {
            state.types[6] = [Type::Normal, Type::Normal]; // Normal vs Normal is 1x
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert_eq!(
                ctx.effectiveness, 4,
                "Normal vs Normal should be 1x (effectiveness 4)"
            );

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (1x): 127
            // 4. No boost

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(damage, 127, "Tinted Lens should NOT boost neutral damage");
        }

        // Case 3: Doubly Not Very Effective (0.25x)
        {
            state.types[6] = [Type::Rock, Type::Steel]; // Normal vs Rock/Steel is 0.5 * 0.5 = 0.25x
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert_eq!(
                ctx.effectiveness, 1,
                "Normal vs Rock/Steel should be 0.25x (effectiveness 1)"
            );

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (0.25x): 127 * 1 / 4 = 31.75 -> 31
            // 4. Tinted Lens (2x): 31 * 2 = 62

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(
                damage, 62,
                "Tinted Lens should double damage for 0.25x effective hits"
            );
        }

        // Case 4: Super Effective (2x) (should NOT boost)
        {
            let fighting_move = MoveId::Karatechop; // Fighting type
                                                    // Target is Rock/Steel (4x weak to Fighting)

            let ctx = DamageContext::new(gen, &state, 0, 6, fighting_move, false);
            // Fighting vs Rock (2x) * Fighting vs Steel (2x) = 4x (effectiveness 16)
            assert_eq!(
                ctx.effectiveness, 16,
                "Fighting vs Rock/Steel should be 4x (effectiveness 16)"
            );

            // 1. Roll 85: 85
            // 2. No STAB (Rattata is Normal): 85
            // 3. Effectiveness (4x): 85 * 16 / 4 = 340
            // 4. No boost from Tinted Lens (effectiveness >= 4)

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(
                damage, 340,
                "Tinted Lens should NOT boost super effective damage"
            );
        }
    }

    #[test]
    fn test_sniper() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100, Def 100
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100; // Atk
        state.abilities[0] = AbilityId::Sniper;

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100; // Def

        let move_id = MoveId::Tackle; // Normal

        // Case 1: Critical Hit (should boost)
        {
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, true); // is_crit = true

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (1x): 127
            // 4. Sniper (1.5x): 127 * 6144 / 4096 = 190.5 -> 190

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0]; // min roll 85

            assert_eq!(damage, 190, "Sniper should boost crit damage by 1.5x");
        }

        // Case 2: No Crit (should NOT boost)
        {
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false); // is_crit = false
            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(damage, 127, "Sniper should NOT boost non-crit damage");
        }
    }

    #[test]
    fn test_screens_doubles() {
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::{BattleFormat, BattleState};
        use crate::types::Type;

        let mut state = BattleState::new();
        state.format = BattleFormat::Doubles;
        let gen = Gen9;

        // Setup: Atk 100, Def 100
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100; // Atk

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100; // Def

        // Reflect active
        state.side_conditions[1].reflect_turns = 5;

        let move_id = MoveId::Tackle; // Physical

        let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

        // 1. Roll 85: 85
        // 2. STAB (1.5x): 127
        // 3. Effectiveness (1x): 127
        // 4. Screens (Doubles: 0.67x): 127 * 2732 / 4096 = 84.71 -> 85 (pokeround)

        let rolls = compute_final_damage(&ctx, 100);
        let damage = rolls[0];

        assert_eq!(
            damage, 85,
            "Screens in doubles should reduce damage by 0.67x"
        );

        // Singles comparison
        let mut state_singles = state; // Copy
        state_singles.format = BattleFormat::Singles;
        let ctx_singles = DamageContext::new(gen, &state_singles, 0, 6, move_id, false);

        // Screens (Singles: 0.5x): 127 * 2048 / 4096 = 63.5 -> 63 (pokeround: round half down)

        let rolls_singles = compute_final_damage(&ctx_singles, 100);
        let damage_singles = rolls_singles[0];

        assert_eq!(
            damage_singles, 63,
            "Screens in singles should reduce damage by 0.5x"
        );
    }

    #[test]
    fn test_filter() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100, Def 100
        state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[0] = [Type::Fighting, Type::Fighting]; // Fighting for SE vs Normal
        state.stats[0][1] = 100; // Atk

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100; // Def
        state.abilities[6] = AbilityId::Filter;

        let move_id = MoveId::Karatechop; // Fighting type

        // Case 1: Super Effective (2x) -> Filter (0.75x)
        {
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert_eq!(
                ctx.effectiveness, 8,
                "Fighting vs Normal should be 2x (effectiveness 8)"
            );

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (2x): 127 * 8 / 4 = 254
            // 4. Filter (0.75x): 254 * 3072 / 4096 = 190.5 -> 190

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(damage, 190, "Filter should reduce SE damage by 0.75x");
        }

        // Case 2: Neutral (1x) -> No Filter
        {
            state.types[0] = [Type::Normal, Type::Normal];
            let move_id_normal = MoveId::Tackle;
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id_normal, false);
            assert_eq!(
                ctx.effectiveness, 4,
                "Normal vs Normal should be 1x (effectiveness 4)"
            );

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (1x): 127
            // 4. No Filter

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(damage, 127, "Filter should NOT reduce neutral damage");
        }
    }

    #[test]
    fn test_rivalry() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::entities::Gender;
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100
        state.species[0] = SpeciesId::from_str("nidorino").unwrap_or(SpeciesId(33)); // Male
        state.types[0] = [Type::Poison, Type::Poison];
        state.stats[0][1] = 100;
        state.abilities[0] = AbilityId::Rivalry;
        state.gender[0] = Gender::Male;

        state.species[6] = SpeciesId::from_str("nidorino").unwrap_or(SpeciesId(33));
        state.types[6] = [Type::Poison, Type::Poison];
        state.stats[6][2] = 100;
        state.gender[6] = Gender::Male;

        let move_id = MoveId::Tackle;

        // Case 1: Same Gender (Male vs Male) -> 1.25x
        {
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            // Tackle BP 40. 40 * 1.25 = 50.
            // 40 * 5120 / 4096 = 50.
            assert_eq!(
                ctx.base_power, 50,
                "Rivalry should boost same gender BP by 1.25x"
            );
        }

        // Case 2: Opposite Gender (Male vs Female) -> 0.75x
        {
            state.gender[6] = Gender::Female;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            // 40 * 0.75 = 30.
            // 40 * 3072 / 4096 = 30.
            assert_eq!(
                ctx.base_power, 30,
                "Rivalry should reduce opposite gender BP by 0.75x"
            );
        }

        // Case 3: Genderless (Male vs Genderless) -> 1x
        {
            state.gender[6] = Gender::Genderless;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 40,
                "Rivalry should not affect genderless targets"
            );
        }
    }

    #[test]
    fn test_sheer_force() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100
        state.species[0] = SpeciesId::from_str("tauros").unwrap_or(SpeciesId(128));
        state.types[0] = [Type::Normal, Type::Normal];
        state.stats[0][1] = 100;
        state.abilities[0] = AbilityId::Sheerforce;

        state.species[6] = SpeciesId::from_str("tauros").unwrap_or(SpeciesId(128));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100;

        // Case 1: Move with secondary effect (Thunderbolt) -> 1.3x
        {
            let move_id = MoveId::Thunderbolt; // BP 90, has 10% para
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            // Note: Verify Thunderbolt has secondary effect flag generated by build.rs
            // If this fails, build.rs logic might be wrong or Thunderbolt data missing secondary

            compute_base_power(&mut ctx);
            // 90 * 1.3 = 117.
            // 90 * 5325 / 4096 = 117.004... -> 117.
            assert_eq!(
                ctx.base_power, 117,
                "Sheer Force should boost move with secondary effect by 1.3x"
            );
        }

        // Case 2: Move without secondary effect (Earthquake) -> 1x
        {
            let move_id = MoveId::Earthquake; // BP 100, no secondary
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 100,
                "Sheer Force should not boost move without secondary effect"
            );
        }
    }

    #[test]
    fn test_sand_force() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::BattleState;
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100
        state.species[0] = SpeciesId::from_str("probopass").unwrap_or(SpeciesId(476));
        state.types[0] = [Type::Rock, Type::Steel];
        state.stats[0][1] = 100;
        state.abilities[0] = AbilityId::Sandforce;

        state.species[6] = SpeciesId::from_str("probopass").unwrap_or(SpeciesId(476));
        state.types[6] = [Type::Rock, Type::Steel];
        state.stats[6][2] = 100;

        // Case 1: Sandstorm + Rock Move -> 1.3x
        {
            state.weather = 3; // Sand
            let move_id = MoveId::Rockthrow; // BP 50, Rock
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            compute_base_power(&mut ctx);
            // 50 * 1.3 = 65.
            assert_eq!(
                ctx.base_power, 65,
                "Sand Force should boost Rock moves in Sand"
            );
        }

        // Case 2: No Weather -> 1x
        {
            state.weather = 0;
            let move_id = MoveId::Rockthrow;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 50,
                "Sand Force should not boost without Sand"
            );
        }

        // Case 3: Sandstorm + Non-boosted Type (e.g. Normal) -> 1x
        {
            state.weather = 3;
            let move_id = MoveId::Tackle; // Normal
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            compute_base_power(&mut ctx);
            assert_eq!(
                ctx.base_power, 40,
                "Sand Force should not boost Normal moves"
            );
        }
    }
    #[test]
    fn test_guts_burn_ignore() {
        use crate::abilities::AbilityId;
        use crate::damage::{DamageContext, Gen9};
        use crate::moves::MoveId;
        use crate::species::SpeciesId;
        use crate::state::{BattleState, Status};
        use crate::types::Type;

        let mut state = BattleState::new();
        let gen = Gen9;

        // Setup: Atk 100
        state.species[0] = SpeciesId::from_str("machamp").unwrap_or(SpeciesId(68));
        state.types[0] = [Type::Fighting, Type::Fighting];
        state.stats[0][1] = 100;
        state.abilities[0] = AbilityId::Guts;

        state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
        state.types[6] = [Type::Normal, Type::Normal];
        state.stats[6][2] = 100;

        // Burn the attacker
        state.status[0] = Status::BURN;

        let move_id = MoveId::Karatechop; // Physical, Fighting
        let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

        // We check effective stats first to ensure Guts is working.
        // Guts boosts Atk by 1.5x when statused.
        let (atk, _) = compute_effective_stats(&ctx);
        assert_eq!(atk, 150, "Guts should boost Attack by 1.5x when burned");

        // Now final damage loop.
        // Burn halving should NOT happen.
        // Input base damage 100.
        // STAB (1.5x) -> 127
        // SE (2x) -> 254
        // No Burn reduction.

        let rolls = compute_final_damage(&ctx, 100);
        let min_damage = rolls[0];

        // If burn reduction applied: 254 / 2 = 127
        // If burn reduction ignored: 254
        assert_eq!(
            min_damage, 254,
            "Guts should ignore burn reduction (got {})",
            min_damage
        );
    }
}
