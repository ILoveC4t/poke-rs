//! Modular damage calculation pipelines.
//!
//! This module defines the `DamagePipeline` trait which allows each generation
//! to specify its own order of operations for final damage calculation.
//!
//! # Architecture
//!
//! The damage calculation has several phases:
//! 1. **Base Power** - Compute effective BP (Technician, Weight-based, etc.)
//! 2. **Effective Stats** - Apply boosts, item/ability stat mods
//! 3. **Base Damage** - The main formula: `(2*L/5+2) * BP * Atk / Def / 50 + 2`
//! 4. **Pre-Random Mods** - Spread, Weather, Crit (order varies by gen)
//! 5. **Final Damage** - Random roll, STAB, Type effectiveness, Burn, Screens
//!
//! Phase 5 is where generations differ the most, which is why we abstract it
//! into the `DamagePipeline` trait.
//!
//! # Library Usage
//!
//! Users can define custom generations by implementing both `GenMechanics` and
//! `DamagePipeline`:
//!
//! ```ignore
//! struct MyCustomGen;
//! struct MyCustomPipeline;
//!
//! impl DamagePipeline for MyCustomPipeline {
//!     fn compute_final_damage(...) -> [u16; 16] { ... }
//! }
//!
//! impl GenMechanics for MyCustomGen {
//!     const GEN: u8 = 0; // Custom
//!     fn pipeline(&self) -> &'static dyn DamagePipeline { &MY_PIPELINE }
//! }
//! ```

use crate::damage::formula::{apply_modifier, of32, pokeround};
use crate::damage::Modifier;
use crate::moves::MoveCategory;

// ============================================================================
// Pipeline Trait
// ============================================================================

/// Defines the order of operations for computing final damage rolls.
///
/// Different generations apply Random Roll, STAB, Type Effectiveness, Burn,
/// and Screens in different orders. This trait allows each generation to
/// define its own sequence.
///
/// The trait is object-safe (`&dyn DamagePipeline`) to support runtime
/// generation selection while keeping the API ergonomic.
pub trait DamagePipeline: Send + Sync {
    /// Compute all 16 final damage values from base damage.
    ///
    /// # Arguments
    /// * `base_damage` - Damage after BP, stats, spread, weather, crit
    /// * `effectiveness` - Type effectiveness multiplier (4 = 1x)
    /// * `has_stab` - Whether STAB applies
    /// * `stab_mod` - The STAB modifier to use (1.5x, 2.0x, etc.)
    /// * `is_crit` - Whether this is a critical hit
    /// * `is_burned` - Whether attacker is burned
    /// * `ignore_burn` - Whether burn damage reduction should be skipped (Guts/Facade)
    /// * `category` - Physical or Special
    /// * `has_screen` - Whether Reflect/Light Screen/Aurora Veil is active
    /// * `screen_mod` - The screen damage reduction modifier
    /// * `attacker_hooks` - Attacker's ability hooks (Tinted Lens, Sniper, etc.)
    /// * `defender_hooks` - Defender's ability hooks (Multiscale, Filter, etc.)
    /// * `item_final_mod` - Closure to apply item final modifiers
    /// * `ability_final_mod` - Closure to apply ability final modifiers
    ///
    /// # Returns
    /// Array of 16 damage values corresponding to random rolls 85-100%.
    fn compute_final_damage(
        &self,
        base_damage: u32,
        effectiveness: u8,
        has_stab: bool,
        stab_mod: Modifier,
        is_crit: bool,
        is_burned: bool,
        ignore_burn: bool,
        category: MoveCategory,
        has_screen: bool,
        screen_mod: Modifier,
        item_final_mod: &dyn Fn(u32) -> u32,
        ability_final_mod: &dyn Fn(u32) -> u32,
    ) -> [u16; 16];
}

// ============================================================================
// Shared Helpers
// ============================================================================

/// Apply random roll (85-100%) to damage.
#[inline]
pub fn apply_random_roll(damage: u32, roll_index: usize) -> u32 {
    let roll_percent = 85 + roll_index as u32;
    of32(damage as u64 * roll_percent as u64) / 100
}

/// Apply STAB modifier with 4096-scale pokeRound.
#[inline]
pub fn apply_stab_4096(damage: u32, stab_mod: Modifier) -> u32 {
    if stab_mod == Modifier::ONE {
        return damage;
    }
    let product = of32(damage as u64 * stab_mod.val() as u64);
    pokeround(product, 4096)
}

/// Apply STAB modifier with floor division (Gen 3-4).
#[inline]
pub fn apply_stab_floor(damage: u32, stab_mod: Modifier) -> u32 {
    if stab_mod == Modifier::ONE {
        return damage;
    }
    apply_modifier(damage, stab_mod)
}

/// Apply type effectiveness (always floor division).
#[inline]
pub fn apply_effectiveness(damage: u32, effectiveness: u8) -> u32 {
    of32(damage as u64 * effectiveness as u64) / 4
}

/// Apply burn damage reduction (halves physical damage).
#[inline]
pub fn apply_burn(damage: u32, is_burned: bool, ignore_burn: bool, category: MoveCategory) -> u32 {
    if is_burned && category == MoveCategory::Physical && !ignore_burn {
        damage / 2
    } else {
        damage
    }
}

/// Apply screen damage reduction.
#[inline]
pub fn apply_screen(damage: u32, is_crit: bool, has_screen: bool, screen_mod: Modifier) -> u32 {
    if !is_crit && has_screen {
        apply_modifier(damage, screen_mod)
    } else {
        damage
    }
}

/// Clamp final damage to valid range (min 1, max u16::MAX).
#[inline]
pub fn clamp_damage(damage: u32) -> u16 {
    damage.max(1).min(u16::MAX as u32) as u16
}

// ============================================================================
// Generation 5+ Pipeline (Modern Standard)
// ============================================================================

/// Gen 5+ damage order: Random → STAB → Effectiveness → Burn → Screen → Mods
///
/// This is the "modern" formula used by Gens 5, 6, 7, 8, and 9.
/// Uses 4096-scale pokeRound for STAB.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen5PlusPipeline;

impl DamagePipeline for Gen5PlusPipeline {
    fn compute_final_damage(
        &self,
        base_damage: u32,
        effectiveness: u8,
        has_stab: bool,
        stab_mod: Modifier,
        is_crit: bool,
        is_burned: bool,
        ignore_burn: bool,
        category: MoveCategory,
        has_screen: bool,
        screen_mod: Modifier,
        item_final_mod: &dyn Fn(u32) -> u32,
        ability_final_mod: &dyn Fn(u32) -> u32,
    ) -> [u16; 16] {
        let mut rolls = [0u16; 16];

        for i in 0..16 {
            // 1. Random roll (FIRST)
            let mut damage = apply_random_roll(base_damage, i);

            // 2. STAB (4096-scale)
            if has_stab {
                damage = apply_stab_4096(damage, stab_mod);
            }

            // 3. Type effectiveness
            damage = apply_effectiveness(damage, effectiveness);

            // 4. Burn (applies in final phase for Gen 5+)
            damage = apply_burn(damage, is_burned, ignore_burn, category);

            // 5. Screen
            damage = apply_screen(damage, is_crit, has_screen, screen_mod);

            // 6. Item final mods (Life Orb, Expert Belt)
            damage = item_final_mod(damage);

            // 7. Ability final mods (Tinted Lens, Multiscale, Filter)
            damage = ability_final_mod(damage);

            rolls[i] = clamp_damage(damage);
        }

        rolls
    }
}

// ============================================================================
// Generation 4 Pipeline (Hybrid)
// ============================================================================

/// Gen 4 damage order: Random → STAB → Effectiveness → Item → Ability
///
/// Gen 4 uses random-first like Gen 5+, but Burn/Screens are applied
/// earlier in the pipeline (before this function is called).
/// Uses 4096-scale pokeRound for STAB (Adaptability was introduced).
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen4Pipeline;

impl DamagePipeline for Gen4Pipeline {
    fn compute_final_damage(
        &self,
        base_damage: u32,
        effectiveness: u8,
        has_stab: bool,
        stab_mod: Modifier,
        _is_crit: bool,
        _is_burned: bool,
        _ignore_burn: bool,
        _category: MoveCategory,
        _has_screen: bool,
        _screen_mod: Modifier,
        item_final_mod: &dyn Fn(u32) -> u32,
        ability_final_mod: &dyn Fn(u32) -> u32,
    ) -> [u16; 16] {
        let mut rolls = [0u16; 16];

        for i in 0..16 {
            // 1. Random roll (FIRST, like Gen 5+)
            let mut damage = apply_random_roll(base_damage, i);

            // 2. STAB (4096-scale, Adaptability exists)
            if has_stab {
                damage = apply_stab_4096(damage, stab_mod);
            }

            // 3. Type effectiveness
            damage = apply_effectiveness(damage, effectiveness);

            // 4. Item final mods
            damage = item_final_mod(damage);

            // 5. Ability final mods
            damage = ability_final_mod(damage);

            // Note: Burn and Screens already applied before base_damage

            rolls[i] = clamp_damage(damage);
        }

        rolls
    }
}

// ============================================================================
// Generation 3 Pipeline (Classic)
// ============================================================================

/// Gen 3 damage order: STAB → Effectiveness → Item → Ability → Random (LAST)
///
/// Gen 3 applies the random roll at the very end, after all modifiers.
/// Uses floor division for STAB (no Adaptability).
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen3Pipeline;

impl DamagePipeline for Gen3Pipeline {
    fn compute_final_damage(
        &self,
        base_damage: u32,
        effectiveness: u8,
        has_stab: bool,
        stab_mod: Modifier,
        _is_crit: bool,
        _is_burned: bool,
        _ignore_burn: bool,
        _category: MoveCategory,
        _has_screen: bool,
        _screen_mod: Modifier,
        _item_final_mod: &dyn Fn(u32) -> u32,
        ability_final_mod: &dyn Fn(u32) -> u32,
    ) -> [u16; 16] {
        let mut rolls = [0u16; 16];

        // Steps 1-4 are applied once, then random roll generates 16 values
        let mut damage = base_damage;

        // 1. STAB (floor division, no Adaptability)
        if has_stab {
            damage = apply_stab_floor(damage, stab_mod);
        }

        // 2. Type effectiveness
        damage = apply_effectiveness(damage, effectiveness);

        // 3. Gen 3 has no item final mods (Life Orb didn't exist)

        // 4. Ability final mods
        damage = ability_final_mod(damage);

        // 5. Random roll (LAST)
        for i in 0..16 {
            let roll_damage = apply_random_roll(damage, i);
            rolls[i] = clamp_damage(roll_damage);
        }

        rolls
    }
}

// ============================================================================
// Static Pipeline Instances
// ============================================================================

/// Static instance for Gen 5+ pipeline.
pub static GEN5_PLUS_PIPELINE: Gen5PlusPipeline = Gen5PlusPipeline;

/// Static instance for Gen 4 pipeline.
pub static GEN4_PIPELINE: Gen4Pipeline = Gen4Pipeline;

/// Static instance for Gen 3 pipeline.
pub static GEN3_PIPELINE: Gen3Pipeline = Gen3Pipeline;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_random_roll() {
        // Roll 0 = 85%
        assert_eq!(apply_random_roll(100, 0), 85);
        // Roll 15 = 100%
        assert_eq!(apply_random_roll(100, 15), 100);
        // Roll 7 = 92%
        assert_eq!(apply_random_roll(100, 7), 92);
    }

    #[test]
    fn test_apply_effectiveness() {
        // 4 = 1x (neutral)
        assert_eq!(apply_effectiveness(100, 4), 100);
        // 8 = 2x (super effective)
        assert_eq!(apply_effectiveness(100, 8), 200);
        // 2 = 0.5x (not very effective)
        assert_eq!(apply_effectiveness(100, 2), 50);
        // 16 = 4x (double super effective)
        assert_eq!(apply_effectiveness(100, 16), 400);
    }

    #[test]
    fn test_gen3_random_last() {
        // In Gen 3, all 16 rolls should differ only by the random factor
        // since random is applied last
        let pipeline = Gen3Pipeline;
        let identity = |d| d;

        let rolls = pipeline.compute_final_damage(
            100,
            4,     // 1x effectiveness
            false, // no STAB
            Modifier::ONE,
            false, // not crit
            false, // not burned
            false, // don't ignore burn
            MoveCategory::Physical,
            false, // no screen
            Modifier::ONE,
            &identity,
            &identity,
        );

        // Roll 0 should be 85, Roll 15 should be 100
        assert_eq!(rolls[0], 85);
        assert_eq!(rolls[15], 100);
    }

    #[test]
    fn test_gen5_random_first() {
        // In Gen 5+, random is first, then modifiers
        let pipeline = Gen5PlusPipeline;
        let identity = |d| d;

        let rolls = pipeline.compute_final_damage(
            100,
            4,     // 1x effectiveness
            false, // no STAB
            Modifier::ONE,
            false, // not crit
            false, // not burned
            false, // don't ignore burn
            MoveCategory::Physical,
            false, // no screen
            Modifier::ONE,
            &identity,
            &identity,
        );

        // Should still be 85-100 since no modifiers applied
        assert_eq!(rolls[0], 85);
        assert_eq!(rolls[15], 100);
    }

    #[test]
    fn test_stab_difference() {
        let identity = |d| d;
        let stab_15x = Modifier::ONE_POINT_FIVE;

        // Gen 3 with STAB
        let gen3_rolls = Gen3Pipeline.compute_final_damage(
            100, 4, true, stab_15x, false, false, false,
            MoveCategory::Physical, false, Modifier::ONE, &identity, &identity,
        );

        // Gen 5 with STAB
        let gen5_rolls = Gen5PlusPipeline.compute_final_damage(
            100, 4, true, stab_15x, false, false, false,
            MoveCategory::Physical, false, Modifier::ONE, &identity, &identity,
        );

        // Both should have STAB applied, but order differs
        // Gen 3: 100 * 1.5 = 150 -> rolls 127-150
        // Gen 5: rolls 85-100 -> * 1.5 = 127-150 (approximately)
        assert!(gen3_rolls[0] > 100);
        assert!(gen5_rolls[0] > 100);
    }
}
