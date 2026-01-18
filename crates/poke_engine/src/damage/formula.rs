//! Core damage formula and math utilities.
//!
//! This module contains the fundamental damage calculation math,
//! including Game Freak's specific rounding and overflow behaviors.

/// 16-bit overflow wrapping (simulates hardware behavior).
/// Values that exceed 65535 wrap around.
#[inline]
pub const fn of16(value: u32) -> u16 {
    (value & 0xFFFF) as u16
}

/// 32-bit overflow wrapping (simulates hardware behavior).
/// Values that exceed u32::MAX wrap around.
#[inline]
pub const fn of32(value: u64) -> u32 {
    (value & 0xFFFF_FFFF) as u32
}

/// Game Freak's rounding function ("pokeRound").
///
/// Rounds 0.5 down instead of up (banker's rounding toward zero).
/// This is critical for cartridge accuracy.
///
/// The fractional part > 0.5 rounds up, otherwise rounds down.
/// This differs from standard rounding where 0.5 rounds up.
#[inline]
pub fn pokeround(value: u32, divisor: u32) -> u32 {
    // Perform division and check if we should round up
    // We round up only if remainder > divisor/2 (strictly greater)
    let quotient = value / divisor;
    let remainder = value % divisor;
    let half = divisor / 2;
    
    // Round up only if remainder is strictly greater than half
    // (0.5 exactly rounds DOWN in Pokemon)
    if remainder > half {
        quotient + 1
    } else {
        quotient
    }
}

/// Apply a 4096-scale division with pokeRound.
///
/// This is the standard way to apply modifiers in Pokemon:
/// `pokeround(value * modifier / 4096)`
#[inline]
#[allow(dead_code)]
pub fn pokeround_4096(value: u32) -> u32 {
    pokeround(value, 4096)
}

/// Apply a 4096-scale modifier with proper pokeRound.
///
/// This performs: `pokeround(value * modifier / 4096)`
/// Game Freak rounds 0.5 DOWN, not up.
#[inline]
pub fn apply_modifier(value: u32, modifier: u16) -> u32 {
    if modifier == 4096 {
        return value;
    }
    let product = of32(value as u64 * modifier as u64);
    pokeround(product, 4096)
}

/// Apply a modifier and floor the result (no rounding).
///
/// Used for crit multiplier and some other cases where
/// the game uses simple floor division.
#[inline]
pub fn apply_modifier_floor(value: u32, modifier_num: u32, modifier_den: u32) -> u32 {
    of32(value as u64 * modifier_num as u64) / modifier_den
}

/// Chain multiple 4096-scale modifiers together.
///
/// Starts at 4096 (1.0x) and multiplies each modifier in sequence.
/// Each intermediate result uses pokeRound (0.5 rounds down).
///
/// Clamps the final result to valid bounds (0.1x to 32x approximately).
pub fn chain_mods(modifiers: &[u16]) -> u32 {
    let mut result: u32 = 4096;
    
    for &modifier in modifiers {
        if modifier != 4096 {
            let product = of32(result as u64 * modifier as u64);
            result = pokeround(product, 4096);
        }
    }
    
    // Clamp to valid range (based on reference implementation)
    result.clamp(1, 131072)
}

/// Calculate base damage before modifiers.
///
/// Formula: `floor((floor(2 * Level / 5 + 2) * BasePower * Attack / Defense) / 50) + 2`
///
/// Each intermediate step is truncated to match cartridge behavior.
///
/// # Arguments
/// * `level` - Attacker's level (1-100)
/// * `base_power` - Move's base power after BP modifiers
/// * `attack` - Effective attack stat (after boosts)
/// * `defense` - Effective defense stat (after boosts)
///
/// # Returns
/// Base damage value before random roll and other modifiers.
pub fn get_base_damage(level: u32, base_power: u32, attack: u32, defense: u32) -> u32 {
    // Avoid division by zero
    if defense == 0 {
        return 0;
    }
    
    // Level factor: floor(2 * level / 5 + 2)
    let level_factor = 2 * level / 5 + 2;
    
    // Main formula with truncation at each step
    // ((level_factor * base_power * attack) / defense) / 50 + 2
    let numerator = of32(level_factor as u64 * base_power as u64);
    let numerator = of32(numerator as u64 * attack as u64);
    let after_defense = numerator / defense;
    let after_50 = after_defense / 50;
    
    after_50 + 2
}

/// Apply the random damage roll.
///
/// The game generates a random value 85-100 and multiplies damage by that percentage.
/// Returns the damage for a specific roll index (0 = 85%, 15 = 100%).
#[inline]
#[allow(dead_code)]
pub fn apply_random_roll(base_damage: u32, roll_index: u8) -> u32 {
    let roll = 85 + (roll_index.min(15) as u32);
    of32(base_damage as u64 * roll as u64) / 100
}

/// Get all 16 possible damage values from random rolls.
#[allow(dead_code)]
pub fn get_all_rolls(base_damage: u32) -> [u32; 16] {
    let mut rolls = [0u32; 16];
    for i in 0..16 {
        rolls[i] = apply_random_roll(base_damage, i as u8);
    }
    rolls
}

/// Boost multiplier table.
///
/// Index 0 = -6, Index 6 = 0, Index 12 = +6
/// Each entry is (numerator, denominator).
const BOOST_TABLE: [(u32, u32); 13] = [
    (2, 8),  // -6: 2/8 = 0.25x
    (2, 7),  // -5: 2/7 ≈ 0.286x
    (2, 6),  // -4: 2/6 ≈ 0.333x
    (2, 5),  // -3: 2/5 = 0.4x
    (2, 4),  // -2: 2/4 = 0.5x
    (2, 3),  // -1: 2/3 ≈ 0.667x
    (2, 2),  //  0: 2/2 = 1.0x
    (3, 2),  // +1: 3/2 = 1.5x
    (4, 2),  // +2: 4/2 = 2.0x
    (5, 2),  // +3: 5/2 = 2.5x
    (6, 2),  // +4: 6/2 = 3.0x
    (7, 2),  // +5: 7/2 = 3.5x
    (8, 2),  // +6: 8/2 = 4.0x
];

/// Apply stat boost stage to a base stat.
///
/// # Arguments
/// * `base_stat` - The unmodified stat value
/// * `stage` - Boost stage from -6 to +6
///
/// # Returns
/// Modified stat value.
pub fn apply_boost(base_stat: u16, stage: i8) -> u16 {
    let stage = stage.clamp(-6, 6);
    let index = (stage + 6) as usize;
    let (num, den) = BOOST_TABLE[index];
    
    of16((base_stat as u32 * num) / den)
}

/// Accuracy/Evasion boost table (different from stat boosts).
///
/// Index 0 = -6, Index 6 = 0, Index 12 = +6
const ACC_EVA_TABLE: [(u32, u32); 13] = [
    (3, 9),  // -6: 33%
    (3, 8),  // -5: 38%
    (3, 7),  // -4: 43%
    (3, 6),  // -3: 50%
    (3, 5),  // -2: 60%
    (3, 4),  // -1: 75%
    (3, 3),  //  0: 100%
    (4, 3),  // +1: 133%
    (5, 3),  // +2: 167%
    (6, 3),  // +3: 200%
    (7, 3),  // +4: 233%
    (8, 3),  // +5: 267%
    (9, 3),  // +6: 300%
];

/// Apply accuracy/evasion boost stage.
#[allow(dead_code)]
pub fn apply_acc_eva_boost(base: u16, stage: i8) -> u16 {
    let stage = stage.clamp(-6, 6);
    let index = (stage + 6) as usize;
    let (num, den) = ACC_EVA_TABLE[index];
    
    of16((base as u32 * num) / den)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_of16() {
        assert_eq!(of16(100), 100);
        assert_eq!(of16(65535), 65535);
        assert_eq!(of16(65536), 0);      // Overflow wraps
        assert_eq!(of16(65537), 1);
    }
    
    #[test]
    fn test_of32() {
        assert_eq!(of32(100), 100);
        assert_eq!(of32(0xFFFF_FFFF), 0xFFFF_FFFF);
        assert_eq!(of32(0x1_0000_0000), 0);
    }
    
    #[test]
    fn test_apply_modifier() {
        // 1.0x modifier
        assert_eq!(apply_modifier(100, 4096), 100);
        
        // 1.5x modifier
        assert_eq!(apply_modifier(100, 6144), 150);
        
        // 0.5x modifier
        assert_eq!(apply_modifier(100, 2048), 50);
        
        // 2.0x modifier
        assert_eq!(apply_modifier(100, 8192), 200);
    }
    
    #[test]
    fn test_chain_mods() {
        // Single 1.5x
        assert_eq!(chain_mods(&[6144]), 6144);
        
        // 1.5x * 1.5x = 2.25x (9216 in 4096-scale)
        let result = chain_mods(&[6144, 6144]);
        assert_eq!(result, 9216);
        
        // 1.5x * 0.5x = 0.75x (3072 in 4096-scale)
        let result = chain_mods(&[6144, 2048]);
        assert_eq!(result, 3072);
    }
    
    #[test]
    fn test_base_damage() {
        // Level 50, 90 power, 100 attack, 100 defense
        // floor((floor(2 * 50 / 5 + 2) * 90 * 100) / 100) / 50 + 2
        // = floor((22 * 90 * 100) / 100) / 50 + 2
        // = floor(1980) / 50 + 2
        // = 39 + 2 = 41
        let damage = get_base_damage(50, 90, 100, 100);
        assert_eq!(damage, 41);
        
        // Level 100
        // floor((floor(2 * 100 / 5 + 2) * 90 * 100) / 100) / 50 + 2
        // = floor((42 * 90 * 100) / 100) / 50 + 2
        // = floor(3780) / 50 + 2
        // = 75 + 2 = 77
        let damage = get_base_damage(100, 90, 100, 100);
        assert_eq!(damage, 77);
    }
    
    #[test]
    fn test_random_rolls() {
        let base = 100;
        
        // 85% roll
        assert_eq!(apply_random_roll(base, 0), 85);
        
        // 100% roll
        assert_eq!(apply_random_roll(base, 15), 100);
        
        // All rolls
        let rolls = get_all_rolls(base);
        assert_eq!(rolls[0], 85);
        assert_eq!(rolls[15], 100);
        assert_eq!(rolls.len(), 16);
    }
    
    #[test]
    fn test_boost_application() {
        let base = 100;
        
        // No boost
        assert_eq!(apply_boost(base, 0), 100);
        
        // +1 = 1.5x
        assert_eq!(apply_boost(base, 1), 150);
        
        // +6 = 4x
        assert_eq!(apply_boost(base, 6), 400);
        
        // -1 = 0.667x
        assert_eq!(apply_boost(base, -1), 66);
        
        // -6 = 0.25x
        assert_eq!(apply_boost(base, -6), 25);
    }
    
    #[test]
    fn test_pokeround() {
        // pokeRound rounds 0.5 DOWN (Game Freak's rounding convention)
        // This differs from standard rounding where 0.5 rounds up
        
        // Exact 0.5 should round DOWN
        // 2048 / 4096 = 0.5 → 0
        assert_eq!(pokeround(2048, 4096), 0);
        
        // Just above 0.5 should round UP
        // 2049 / 4096 > 0.5 → 1
        assert_eq!(pokeround(2049, 4096), 1);
        
        // Standard cases
        assert_eq!(pokeround(4096, 4096), 1);   // 1.0
        assert_eq!(pokeround(6144, 4096), 1);   // 1.5 → rounds to 1 (0.5 rounds down)
        assert_eq!(pokeround(6145, 4096), 2);   // 1.5+ → rounds to 2
        assert_eq!(pokeround(8192, 4096), 2);   // 2.0
        
        // Test with smaller divisor
        assert_eq!(pokeround(5, 10), 0);   // 0.5 → 0
        assert_eq!(pokeround(6, 10), 1);   // 0.6 → 1
        assert_eq!(pokeround(15, 10), 1);  // 1.5 → 1
        assert_eq!(pokeround(16, 10), 2);  // 1.6 → 2
    }
    
    #[test]
    fn test_apply_modifier_floor() {
        // Crit uses floor(x * 1.5), not pokeRound
        assert_eq!(apply_modifier_floor(100, 3, 2), 150); // 100 * 1.5 = 150
        assert_eq!(apply_modifier_floor(101, 3, 2), 151); // 101 * 1.5 = 151.5 → 151
        assert_eq!(apply_modifier_floor(99, 3, 2), 148);  // 99 * 1.5 = 148.5 → 148
    }
}
