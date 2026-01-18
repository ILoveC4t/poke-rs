//! Damage modifier pipeline.
//!
//! This module contains functions that modify damage at various stages
//! of the calculation. Each function is a discrete step in the pipeline.

use super::context::DamageContext;
use super::formula::{apply_boost, apply_modifier, of16};
use super::generations::{GenMechanics, Weather, Terrain};
use crate::abilities::AbilityId;
use crate::items::ItemId;
use crate::moves::{MoveCategory, MoveFlags};

// ============================================================================
// Phase 1: Base Power Computation
// ============================================================================

/// Compute the effective base power after ability and item modifiers.
pub fn compute_base_power<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    let mut bp = ctx.base_power as u32;
    
    // Technician: 1.5x for moves with BP <= 60
    if ctx.attacker_ability == AbilityId::Technician && bp <= 60 {
        bp = bp * 3 / 2;
    }
    
    // TODO: Implement these base power modifiers
    // - Rivalry (+/- 25% based on gender)
    // - Reckless (1.2x for recoil moves)
    // - Iron Fist (1.2x for punch moves)
    // - Sheer Force (1.3x, disables secondary effects)
    // - Sand Force (1.3x for Rock/Ground/Steel in Sand)
    // - Analytic (1.3x if moving last)
    // - Tough Claws (1.3x for contact moves)
    // - Aerilate/Pixilate/Refrigerate/Galvanize (1.2x + type change)
    // - Steelworker (1.5x for Steel moves)
    // - Water Bubble (2x for Water moves)
    
    // TODO: Item-based BP modifiers
    // - Muscle Band (1.1x Physical)
    // - Wise Glasses (1.1x Special)
    // - Type-boosting items (1.2x)
    // - Plates (1.2x)
    // - Gems (1.5x, one-time)
    
    // TODO: Weight-based moves
    // - Grass Knot / Low Kick: BP based on target weight
    // - Heavy Slam / Heat Crash: BP based on weight ratio
    
    // TODO: HP-based moves
    // - Eruption / Water Spout: BP = 150 * currentHP / maxHP
    // - Flail / Reversal: Inverse HP scaling
    
    // TODO: Other variable BP moves
    // - Acrobatics (2x without item)
    // - Facade (2x when statused)
    // - Venoshock (2x vs poisoned)
    // - Hex (2x vs statused)
    // - Brine (2x below 50% HP)
    // - Assurance (2x if target was hit this turn)
    // - Payback (2x if target moved first)
    // - Avalanche / Revenge (2x if hit by target this turn)
    // - Stored Power / Power Trip (20 + 20 per boost)
    // - Punishment (inverse of target's boosts)
    // - Electro Ball (speed ratio)
    // - Gyro Ball (inverse speed ratio)
    // - Foul Play (uses target's Atk)
    
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
    let (atk_idx, def_idx) = ctx.get_stat_indices();
    
    let mut attack = ctx.state.stats[ctx.attacker][atk_idx];
    let mut defense = ctx.state.stats[ctx.defender][def_idx];
    
    // Get boost stages
    // Boost indices: 0=Atk, 1=Def, 2=SpA, 3=SpD, 4=Spe
    let atk_boost_idx = if atk_idx == 1 { 0 } else { 2 }; // Atk or SpA
    let def_boost_idx = if def_idx == 2 { 1 } else { 3 }; // Def or SpD
    
    let atk_boost = ctx.state.boosts[ctx.attacker][atk_boost_idx];
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
    
    // Ability modifiers for attack
    if ctx.gen.has_abilities() {
        // Hustle: 1.5x Attack (accuracy penalty handled elsewhere)
        if ctx.attacker_ability == AbilityId::Hustle && ctx.category == MoveCategory::Physical {
            attack = of16(attack as u32 * 3 / 2);
        }
        
        // TODO: More attack-modifying abilities
        // - Pure Power / Huge Power (2x Atk)
        // - Flower Gift (1.5x Atk in Sun)
        // - Guts (1.5x Atk when statused)
        // - Defeatist (0.5x when below 50% HP)
        // - Slow Start (0.5x for 5 turns)
        // - Stakeout (2x if target switches in)
        // - Gorilla Tactics (1.5x Atk, locked into one move)
        // - Solar Power (1.5x SpA in Sun)
        // - Plus/Minus (1.5x SpA with partner)
        
        // Ability modifiers for defense
        // - Marvel Scale (1.5x Def when statused)
        // - Fur Coat (2x Def)
        // - Grass Pelt (1.5x Def in Grassy Terrain)
        // - Ice Scales (2x SpD)
    }
    
    // Item modifiers
    let attacker_item = ctx.state.items[ctx.attacker];
    let _defender_item = ctx.state.items[ctx.defender];
    
    // Choice Band: 1.5x Atk
    if attacker_item == ItemId::Choiceband && ctx.category == MoveCategory::Physical {
        attack = of16(attack as u32 * 3 / 2);
    }
    
    // Choice Specs: 1.5x SpA
    if attacker_item == ItemId::Choicespecs && ctx.category == MoveCategory::Special {
        attack = of16(attack as u32 * 3 / 2);
    }
    
    // TODO: More item modifiers
    // - Assault Vest (1.5x SpD)
    // - Eviolite (1.5x Def/SpD if not fully evolved)
    // - Deep Sea Scale/Tooth (Clamperl)
    // - Metal Powder/Quick Powder (Ditto)
    // - Thick Club (Cubone/Marowak 2x Atk)
    // - Light Ball (Pikachu 2x Atk/SpA)
    
    (attack.max(1), defense.max(1))
}

// ============================================================================
// Phase 3: Pre-Random Modifiers
// ============================================================================

/// Apply spread move modifier (0.75x for hitting multiple targets).
pub fn apply_spread_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    if ctx.is_spread {
        ctx.apply_mod(3072); // 0.75x
    }
}

/// Apply weather modifier.
pub fn apply_weather_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    let weather = Weather::from_u8(ctx.state.weather);
    
    if let Some(modifier) = ctx.gen.weather_modifier(weather, ctx.move_type) {
        ctx.apply_mod(modifier);
    }
    
    // TODO: Handle weather immunity abilities
    // - Cloud Nine / Air Lock suppress weather effects
}

/// Apply terrain modifier.
#[allow(dead_code)]
pub fn apply_terrain_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    let terrain = Terrain::from_u8(ctx.state.terrain);
    
    // Terrain affects the user of the move if they're grounded
    if let Some(modifier) = ctx.gen.terrain_modifier(terrain, ctx.move_type, ctx.attacker_grounded) {
        ctx.apply_mod(modifier);
    }
}

/// Apply critical hit modifier.
pub fn apply_crit_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    if ctx.is_crit {
        ctx.apply_mod(ctx.gen.crit_multiplier());
    }
}

// ============================================================================
// Phase 4: Final Damage Computation
// ============================================================================

/// Compute final damage for all 16 random rolls.
///
/// This applies:
/// - Random roll (85-100%)
/// - STAB
/// - Type effectiveness
/// - Burn (for physical moves)
/// - Screens
/// - Final modifiers (Life Orb, etc.)
pub fn compute_final_damage<G: GenMechanics>(ctx: &DamageContext<'_, G>, base_damage: u32) -> [u16; 16] {
    let mut rolls = [0u16; 16];
    
    // Apply pre-random chain modifier to base damage
    let modified_base = apply_modifier(base_damage, ctx.chain_mod as u16);
    
    // Type immunity check
    if ctx.effectiveness == 0 {
        return rolls; // All zeros
    }
    
    for i in 0..16 {
        // Random roll (85-100%)
        let roll_percent = 85 + i as u32;
        let mut damage = (modified_base * roll_percent) / 100;
        
        // STAB
        if ctx.has_stab {
            let stab_mod = ctx.gen.stab_multiplier(ctx.has_adaptability, ctx.is_tera_stab);
            damage = apply_modifier(damage, stab_mod);
        }
        
        // Type effectiveness
        // effectiveness is in units where 4 = 1x
        // We multiply by effectiveness and divide by 4
        damage = damage * ctx.effectiveness as u32 / 4;
        
        // Burn (0.5x for physical, unless Guts/Facade)
        if ctx.is_burned() 
            && ctx.category == MoveCategory::Physical 
            && ctx.attacker_ability != AbilityId::Guts
            // TODO: Check if move is Facade
        {
            damage = apply_modifier(damage, ctx.gen.burn_modifier());
        }
        
        // Screens (Reflect/Light Screen/Aurora Veil)
        // 0.5x in singles, 0.67x in doubles
        if !ctx.is_crit && ctx.has_screen(ctx.category == MoveCategory::Physical) {
            ctx.apply_mod_to(&mut damage, 2048); // 0.5x for singles
            // TODO: 2732 (0.67x) for doubles
        }
        
        // Final modifiers
        // TODO: Life Orb (5324 = 1.3x)
        // TODO: Expert Belt (4915 = 1.2x for super effective)
        // TODO: Tinted Lens (8192 = 2x for not very effective)
        // TODO: Sniper (6144 = 1.5x for crits)
        // TODO: Solid Rock / Filter (3072 = 0.75x for super effective)
        // TODO: Prism Armor (3072 = 0.75x for super effective)
        // TODO: Multiscale / Shadow Shield (2048 = 0.5x at full HP)
        // TODO: Fluffy (2048 = 0.5x for contact, 8192 = 2x for Fire)
        // TODO: Friend Guard (3072 = 0.75x)
        // TODO: Neuroforce (5120 = 1.25x for super effective)
        
        // Minimum damage is 1 (unless immune)
        rolls[i] = damage.max(1).min(u16::MAX as u32) as u16;
    }
    
    rolls
}

impl<G: GenMechanics> DamageContext<'_, G> {
    /// Apply a modifier directly to a damage value (for post-random mods).
    fn apply_mod_to(&self, damage: &mut u32, modifier: u16) {
        *damage = apply_modifier(*damage, modifier);
    }
}

// ============================================================================
// Item Damage Modifiers (Helpers)
// ============================================================================

/// Check if an item is a type-boosting item for the given type.
#[allow(dead_code)]
fn get_type_boost_item_mod(item: ItemId, move_type: crate::types::Type) -> Option<u16> {
    use crate::types::Type;
    
    // Type-boosting items give 1.2x (4915 in 4096-scale)
    let matches = match (item, move_type) {
        (ItemId::Silkscarf, Type::Normal) => true,
        (ItemId::Blackbelt, Type::Fighting) => true,
        (ItemId::Sharpbeak, Type::Flying) => true,
        (ItemId::Poisonbarb, Type::Poison) => true,
        (ItemId::Softsand, Type::Ground) => true,
        (ItemId::Hardstone, Type::Rock) => true,
        (ItemId::Silverpowder, Type::Bug) => true,
        (ItemId::Spelltag, Type::Ghost) => true,
        (ItemId::Metalcoat, Type::Steel) => true,
        (ItemId::Charcoal, Type::Fire) => true,
        (ItemId::Mysticwater, Type::Water) => true,
        (ItemId::Miracleseed, Type::Grass) => true,
        (ItemId::Magnet, Type::Electric) => true,
        (ItemId::Twistedspoon, Type::Psychic) => true,
        (ItemId::Nevermeltice, Type::Ice) => true,
        (ItemId::Dragonfang, Type::Dragon) => true,
        (ItemId::Blackglasses, Type::Dark) => true,
        // No Fairy-type boosting item in core series
        _ => false,
    };
    
    if matches { Some(4915) } else { None } // 1.2x
}

/// Check if attacker has a contact-based ability modifier.
#[allow(dead_code)]
fn has_contact_ability_boost(ability: AbilityId, move_flags: MoveFlags) -> Option<u16> {
    if !move_flags.contains(MoveFlags::CONTACT) {
        return None;
    }
    
    match ability {
        AbilityId::Toughclaws => Some(5325), // 1.3x
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_boost_items() {
        use crate::types::Type;
        
        // Charcoal boosts Fire
        assert_eq!(get_type_boost_item_mod(ItemId::Charcoal, Type::Fire), Some(4915));
        
        // Charcoal doesn't boost Water
        assert_eq!(get_type_boost_item_mod(ItemId::Charcoal, Type::Water), None);
    }
}
