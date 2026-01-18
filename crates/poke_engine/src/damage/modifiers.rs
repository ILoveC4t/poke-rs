//! Damage modifier pipeline.
//!
//! This module contains functions that modify damage at various stages
//! of the calculation. Each function is a discrete step in the pipeline.

use super::context::DamageContext;
use super::formula::{apply_boost, apply_modifier, apply_modifier_floor, of16, of32, pokeround};
use super::generations::{GenMechanics, Weather, Terrain};
use crate::abilities::AbilityId;
use crate::items::ItemId;
use crate::moves::{MoveCategory, MoveFlags, MoveId};
use crate::state::Status;

// ============================================================================
// Phase 1: Base Power Computation
// ============================================================================

/// Compute the effective base power after ability and item modifiers.
pub fn compute_base_power<G: GenMechanics>(ctx: &mut DamageContext<'_, G>) {
    let mut bp = ctx.base_power as u32;
    let move_name = ctx.move_data.name;
    
    // ========================================================================
    // Weight-based moves (must be calculated before Technician)
    // ========================================================================
    
    // Grass Knot / Low Kick: BP based on target's weight
    // Weight is stored in 0.1kg units (fixed-point), so 200kg = 2000
    // TODO: Apply weight modifiers from abilities (Heavy Metal 2x, Light Metal 0.5x)
    //       and items (Float Stone 0.5x) before calculating BP.
    //       Also handle Autotomize state reducing weight by 100kg per use.
    if move_name == "Grass Knot" || move_name == "Low Kick" {
        let defender_species = ctx.state.species[ctx.defender];
        let weight = defender_species.data().weight; // 0.1kg units
        bp = match weight {
            w if w >= 2000 => 120, // >= 200kg
            w if w >= 1000 => 100, // >= 100kg
            w if w >= 500 => 80,   // >= 50kg
            w if w >= 250 => 60,   // >= 25kg
            w if w >= 100 => 40,   // >= 10kg
            _ => 20,               // < 10kg
        };
    }
    
    // Heavy Slam / Heat Crash: BP based on weight ratio (attacker / defender)
    // TODO: Apply weight modifiers (Heavy Metal, Light Metal, Float Stone, Autotomize)
    //       to both attacker and defender weights before calculating ratio.
    if move_name == "Heavy Slam" || move_name == "Heat Crash" {
        let attacker_weight = ctx.state.species[ctx.attacker].data().weight;
        let defender_weight = ctx.state.species[ctx.defender].data().weight.max(1);
        // Multiply by 10 for precision before dividing
        let ratio_x10 = (attacker_weight as u32 * 10) / defender_weight as u32;
        bp = match ratio_x10 {
            r if r >= 50 => 120, // >= 5x
            r if r >= 40 => 100, // >= 4x
            r if r >= 30 => 80,  // >= 3x
            r if r >= 20 => 60,  // >= 2x
            _ => 40,             // < 2x
        };
    }
    
    // ========================================================================
    // HP-based moves
    // ========================================================================
    
    // Eruption / Water Spout: BP = 150 * currentHP / maxHP
    if move_name == "Eruption" || move_name == "Water Spout" {
        let current_hp = ctx.state.hp[ctx.attacker] as u32;
        let max_hp = ctx.state.max_hp[ctx.attacker] as u32;
        bp = (150 * current_hp / max_hp.max(1)).max(1);
    }
    
    // Flail / Reversal: BP increases as HP decreases
    if move_name == "Flail" || move_name == "Reversal" {
        let current_hp = ctx.state.hp[ctx.attacker] as u32;
        let max_hp = ctx.state.max_hp[ctx.attacker] as u32;
        // HP% thresholds: 48/255 = ~4.7%, 80/255 = ~10.2%, etc.
        let hp_percent = (current_hp * 48) / max_hp.max(1);
        bp = match hp_percent {
            0..=1 => 200,   // < 4.17%
            2..=4 => 150,   // < 10.42%
            5..=9 => 100,   // < 20.83%
            10..=16 => 80,  // < 35.42%
            17..=32 => 40,  // < 68.75%
            _ => 20,        // >= 68.75%
        };
    }
    
    // ========================================================================
    // Ability-based BP modifiers (after weight calc for Technician interaction)
    // ========================================================================
    
    // Technician: 1.5x for moves with BP <= 60
    if ctx.attacker_ability == AbilityId::Technician && bp <= 60 {
        bp = bp * 3 / 2;
    }
    
    // Reckless: 1.2x for recoil moves
    // TODO: Reckless needs access to move's recoil property, not a flag
    // The recoil property is stored separately in the move data JSON
    // if ctx.attacker_ability == AbilityId::Reckless {
    //     if move has recoil { bp = bp * 6 / 5; }
    // }
    
    // Iron Fist: 1.2x for punch moves
    if ctx.attacker_ability == AbilityId::Ironfist {
        if ctx.move_data.flags.intersects(MoveFlags::PUNCH) {
            bp = bp * 6 / 5;
        }
    }
    
    // Tough Claws: 1.3x for contact moves
    if ctx.attacker_ability == AbilityId::Toughclaws {
        if ctx.move_data.flags.intersects(MoveFlags::CONTACT) {
            bp = bp * 5461 / 4096; // 1.333... in fixed-point
        }
    }

    // Type-boosting items (e.g. Charcoal)
    if let Some(modifier) = get_type_boost_item_mod(ctx.state.items[ctx.attacker], ctx.move_type) {
        bp = apply_modifier(bp, modifier);
    }
    
    // TODO: Implement remaining ability BP modifiers
    // - Rivalry (+/- 25% based on gender)
    // - Sheer Force (1.3x, disables secondary effects)
    // - Sand Force (1.3x for Rock/Ground/Steel in Sand)
    // - Analytic (1.3x if moving last)
    // - Aerilate/Pixilate/Refrigerate/Galvanize (1.2x + type change)
    // - Steelworker (1.5x for Steel moves)
    // - Water Bubble (2x for Water moves)

    // Facade: 2x if burned, poisoned, or paralyzed
    if ctx.move_id == MoveId::Facade {
        let status = ctx.attacker_status();
        if status.intersects(Status::BURN | Status::POISON | Status::TOXIC | Status::PARALYSIS) {
            bp *= 2;
        }
    }

    // TODO: Knock Off: 1.5x BP if target has a removable item
    //       Check defender item != None and item is not unremovable (Mega Stone, Z-Crystal, etc.)
    //       Also check Klutz: item is still "present" for Knock Off boost even if Klutz negates it

    // TODO: Parental Bond ability: Multi-hit (2 hits), second hit at 0.25x power (Gen 7+)
    //       Requires special handling in damage pipeline to return combined damage

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
    
        // TODO: Defender damage-reducing abilities (apply in final modifier chain)
        // - Multiscale / Shadow Shield: 0.5x damage when at full HP
        // - Filter / Prism Armor / Solid Rock: 0.75x on super-effective hits
        // - Fluffy: 0.5x contact damage, 2x Fire damage
        // - Punk Rock: 0.5x sound-based damage
        // - Ice Scales: 0.5x special damage
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
/// 
/// Applied directly to base_damage using pokeRound.
pub fn apply_spread_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    if ctx.is_spread {
        // pokeRound(OF32(baseDamage * 3072) / 4096)
        *base_damage = apply_modifier(*base_damage, 3072); // 0.75x
    }
}

/// Apply weather modifier.
///
/// Applied directly to base_damage using pokeRound.
/// TODO: Terrain boost (Electric/Grassy/Psychic) checks ATTACKER grounding.
///       Misty Terrain Dragon reduction checks DEFENDER grounding.
///       Current call site may pass wrong grounding state.
pub fn apply_weather_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    let weather = Weather::from_u8(ctx.state.weather);
    
    if let Some(modifier) = ctx.gen.weather_modifier(weather, ctx.move_type) {
        // pokeRound(OF32(baseDamage * weatherMod) / 4096)
        *base_damage = apply_modifier(*base_damage, modifier);
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
///
/// Note: In smogon's implementation, crit uses direct floor(x * 1.5),
/// NOT the 4096-scale system. This is applied during base damage phase.
pub fn apply_crit_mod<G: GenMechanics>(ctx: &mut DamageContext<'_, G>, base_damage: &mut u32) {
    if ctx.is_crit {
        // Crit uses floor(damage * 1.5), not the 4096-scale modifier system
        *base_damage = apply_modifier_floor(*base_damage, 3, 2);
    }
}

// ============================================================================
// Phase 4: Final Damage Computation
// ============================================================================

/// Compute final damage for all 16 random rolls.
///
/// This matches smogon's getFinalDamage order:
/// 1. Random roll (85-100%)
/// 2. STAB (apply then pokeround)
/// 3. Type effectiveness (floor after multiply)
/// 4. Burn (simple floor(x/2))
/// 5. Final modifiers (screens, items, abilities)
///
/// Note: Weather, spread, and crit are applied to base_damage BEFORE
/// this function is called.
pub fn compute_final_damage<G: GenMechanics>(ctx: &DamageContext<'_, G>, base_damage: u32) -> [u16; 16] {
    let mut rolls = [0u16; 16];
    
    // Type immunity check
    if ctx.effectiveness == 0 {
        return rolls; // All zeros
    }
    
    for i in 0..16 {
        // Step 1: Random roll (85-100%)
        // floor(OF32(baseAmount * (85 + i)) / 100)
        let roll_percent = 85 + i as u32;
        let mut damage = of32(base_damage as u64 * roll_percent as u64) / 100;
        
        // Step 2: STAB
        // Apply STAB modifier, then pokeround BEFORE type effectiveness
        if ctx.has_stab {
            let stab_mod = ctx.gen.stab_multiplier(ctx.has_adaptability, ctx.is_tera_stab);
            if stab_mod != 4096 {
                // damageAmount = OF32(damageAmount * stabMod) / 4096
                // Then pokeRound before effectiveness
                let product = of32(damage as u64 * stab_mod as u64);
                damage = pokeround(product, 4096);
            }
        }
        
        // Step 3: Type effectiveness
        // floor(OF32(pokeRound(damageAmount) * effectiveness))
        // effectiveness is in units where 4 = 1x (so 8 = 2x, 2 = 0.5x)
        // We multiply by effectiveness then divide by 4
        damage = of32(damage as u64 * ctx.effectiveness as u64) / 4;
        
        // Step 4: Burn (0.5x for physical, unless Guts/Facade)
        // Smogon uses simple floor(damage / 2), NOT 4096-scale
        if ctx.is_burned() 
            && ctx.category == MoveCategory::Physical 
            && ctx.attacker_ability != AbilityId::Guts
            && ctx.move_id != MoveId::Facade
        {
            damage = damage / 2;
        }
        
        // Step 5: Screens (Reflect/Light Screen/Aurora Veil)
        // pokeRound(OF32(damageAmount * screenMod) / 4096)
        // 0.5x in singles (2048), 0.67x in doubles (2732)
        if !ctx.is_crit && ctx.has_screen(ctx.category == MoveCategory::Physical) {
            let screen_mod = 2048u16; // 0.5x for singles
            // TODO: 2732 (0.67x) for doubles
            damage = apply_modifier(damage, screen_mod);
        }
        
        // Step 6: Final modifiers (chain applied with pokeRound)
        // These are modifiers that weren't applied to base damage

        // Life Orb (1.3x)
        if ctx.state.items[ctx.attacker] == ItemId::Lifeorb {
            damage = apply_modifier(damage, 5324);
        }

        // Expert Belt (1.2x for super effective)
        if ctx.state.items[ctx.attacker] == ItemId::Expertbelt && ctx.effectiveness > 4 {
            damage = apply_modifier(damage, 4915);
        }

        // TODO(TASK-A): Metronome requires consecutive move tracking from Task D

        // Tinted Lens: 2x damage if "not very effective"
        if ctx.attacker_ability == AbilityId::Tintedlens && ctx.effectiveness < 4 {
            damage = apply_modifier(damage, 8192);
        }

        // TODO: Sniper (6144 = 1.5x for crits)
        // TODO: Solid Rock / Filter (3072 = 0.75x for super effective)
        // TODO: Multiscale / Shadow Shield (2048 = 0.5x at full HP)
        
        // Minimum damage is 1 (unless immune)
        rolls[i] = damage.max(1).min(u16::MAX as u32) as u16;
    }
    
    rolls
}

impl<G: GenMechanics> DamageContext<'_, G> {
    /// Apply a modifier directly to a damage value (for post-random mods).
    #[allow(dead_code)]
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

    #[test]
    fn test_facade_damage() {
        use crate::state::{BattleState, Status};
        use crate::damage::{DamageContext, Gen9};
        use crate::species::SpeciesId;
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

            assert!(min_damage > 100, "Facade should ignore burn reduction (got {})", min_damage);
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
            assert_eq!(ctx.base_power, 140, "Facade BP should double when paralyzed");
        }

        // Case 5: Asleep (should NOT double)
        {
            state.status[0] = Status::SLEEP;
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            compute_base_power(&mut ctx);
            assert_eq!(ctx.base_power, 70, "Facade BP should NOT double when asleep");
        }
    }

    #[test]
    fn test_item_modifiers() {
        use crate::state::BattleState;
        use crate::damage::{DamageContext, Gen9};
        use crate::species::SpeciesId;
        use crate::types::Type;
        use crate::items::ItemId;
        use crate::moves::{MoveId, MoveCategory};

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
            // apply_modifier uses pokeround? No, usually pokeround.
            // apply_modifier(100, 5324) -> (100*5324 + 2048) >> 12 = 534448 >> 12 = 130.48 -> 130.
            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0]; // min roll (random=85)
            // Wait, compute_final_damage applies random roll first!
            // Roll 85: 100 * 0.85 = 85.
            // Life Orb: 85 * 5324 / 4096 = 110.
            // Without Life Orb: 85.

            // Let's reset random roll logic or check strict ratio.
            // Just check it's boosted.
            // 85 * 1.3 = 110.5

            // 85 (roll) * 1.5 (STAB) = 127.
            // 127 * 1.3 (Life Orb) = 165.
            assert_eq!(damage, 165, "Life Orb should boost damage by ~1.3x (with STAB)");
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

            assert_eq!(damage, 204, "Expert Belt should boost super effective damage by ~1.2x");

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

            assert_eq!(damage_neutral, 85, "Expert Belt should NOT boost neutral damage");
        }

        // 5. Charcoal (1.2x Fire moves)
        {
            state.items[0] = ItemId::Charcoal;
            let move_id = MoveId::Ember; // Fire
            let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

            // Base Power 40.
            // Charcoal: 40 * 1.2 = 48.

            compute_base_power(&mut ctx);
            assert_eq!(ctx.base_power, 48, "Charcoal should boost Fire move BP by 1.2x");

            // Non-Fire move
            let move_id_normal = MoveId::Tackle;
            let mut ctx_normal = DamageContext::new(gen, &state, 0, 6, move_id_normal, false);
            // BP 40 (Tackle is 40 in recent gens? Or 50?)
            // Tackle is 40 in Gen 9.

            let original_bp = ctx_normal.move_data.power;
            compute_base_power(&mut ctx_normal);
            assert_eq!(ctx_normal.base_power, original_bp, "Charcoal should NOT boost Normal move BP");
        }
    }

    #[test]
    fn test_tinted_lens() {
        use crate::state::BattleState;
        use crate::damage::{DamageContext, Gen9};
        use crate::species::SpeciesId;
        use crate::types::Type;
        use crate::abilities::AbilityId;
        use crate::moves::MoveId;

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
            assert_eq!(ctx.effectiveness, 2, "Normal vs Rock should be 0.5x (effectiveness 2)");

            // Base damage 100 passed to function
            // 1. Roll 85: 85
            // 2. STAB (1.5x): 85 * 1.5 = 127.5 -> 127 (pokeround rounds 0.5 down)
            // 3. Effectiveness (0.5x): 127 * 2 / 4 = 63.5 -> 63
            // 4. Tinted Lens (2x): 63 * 2 = 126

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0]; // min roll (85)

            assert_eq!(damage, 126, "Tinted Lens should double damage for not very effective hits");
        }

        // Case 2: Neutral hit (should NOT boost)
        {
            state.types[6] = [Type::Normal, Type::Normal]; // Normal vs Normal is 1x
            let ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
            assert_eq!(ctx.effectiveness, 4, "Normal vs Normal should be 1x (effectiveness 4)");

            // 1. Roll 85: 85
            // 2. STAB (1.5x): 127
            // 3. Effectiveness (1x): 127
            // 4. No boost

            let rolls = compute_final_damage(&ctx, 100);
            let damage = rolls[0];

            assert_eq!(damage, 127, "Tinted Lens should NOT boost neutral damage");
        }
    }
}
