//! Move hook implementations.
//!
//! Contains the condition check functions for moves with conditional power boosts.

use crate::state::{BattleState, Status};
use crate::moves::Move;
use crate::types::Type;
use crate::damage::generations::Weather;
use crate::abilities::{AbilityId, AbilityFlags};
use crate::items::ItemId;

// ============================================================================
// Knock Off: 1.5x if target has removable item
// ============================================================================

pub fn knockoff_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    let item = state.items[defender];
    if item == crate::items::ItemId::None {
        return false;
    }
    let item_data = item.data();
    !item_data.is_unremovable
}

// ============================================================================
// Venoshock: 2x if target is poisoned
// ============================================================================

pub fn venoshock_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    state.status[defender].intersects(Status::POISON | Status::TOXIC)
}

// ============================================================================
// Hex: 2x if target has any major status condition
// ============================================================================

pub fn hex_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    state.status[defender] != Status::NONE
}

// ============================================================================
// Brine: 2x if target is at or below 50% HP
// ============================================================================

pub fn brine_condition(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
) -> bool {
    let hp = state.hp[defender];
    let max_hp = state.max_hp[defender];
    hp * 2 <= max_hp
}

// ============================================================================
// Weather Ball: Type changes and 2x power in weather
// ============================================================================

pub fn on_modify_type_weather_ball(
    state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    base_type: Type,
) -> Type {
    let weather = Weather::from_u8(state.weather);
    match weather {
        Weather::Sun | Weather::HarshSun => Type::Fire,
        Weather::Rain | Weather::HeavyRain => Type::Water,
        Weather::Sand => Type::Rock,
        Weather::Hail | Weather::Snow => Type::Ice,
        _ => base_type,
    }
}

pub fn on_modify_base_power_weather_ball(
    state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    bp: u16,
) -> u16 {
    // Gen 3: Damage is doubled after crit, not BP doubled
    if state.generation == 3 {
        return bp;
    }
    
    // Gen 4+: Double BP in weather
    let weather = Weather::from_u8(state.weather);
    match weather {
        Weather::Sun | Weather::HarshSun |
        Weather::Rain | Weather::HeavyRain |
        Weather::Sand |
        Weather::Hail | Weather::Snow => bp * 2,
        _ => bp,
    }
}

/// Gen 3 Weather Ball: Doubles damage after crit instead of BP.
pub fn on_modify_final_damage_weather_ball(
    state: &BattleState,
    _attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    damage: u32,
) -> u32 {
    // Only applies in Gen 3
    if state.generation != 3 {
        return damage;
    }
    
    let weather = Weather::from_u8(state.weather);
    match weather {
        Weather::Sun | Weather::HarshSun |
        Weather::Rain | Weather::HeavyRain |
        Weather::Sand |
        Weather::Hail | Weather::Snow => damage * 2,
        _ => damage,
    }
}

// ============================================================================
// Freeze-Dry: Super effective vs Water
// ============================================================================

pub fn freeze_dry_effectiveness(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
    effectiveness: u8,
    _type_chart: &dyn Fn(Type, Type) -> u8,
) -> u8 {
    let def_types = state.types[defender];
    if def_types[0] == Type::Water || def_types[1] == Type::Water {
        // Normally 0.5x (2) -> 2x (8) requires * 4.
        // If Water/Water: 0.5x -> 2x.
        // If Water/Grass: 0.5 * 2 = 1x -> 2 * 2 = 4x. (4 -> 16).
        return (effectiveness as u16 * 4).min(255) as u8;
    }
    effectiveness
}

// ============================================================================
// Flying Press: Dual Fighting/Flying effectiveness
// ============================================================================

pub fn flying_press_effectiveness(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
    effectiveness: u8,
    type_chart: &dyn Fn(Type, Type) -> u8,
) -> u8 {
    let def_types = state.types[defender];
    
    // effectiveness passed in is Fighting vs Target.
    // We need to calculate Flying vs Target.
    let flying_eff_1 = type_chart(Type::Flying, def_types[0]);
    let flying_eff_2 = if def_types[1] != def_types[0] {
        type_chart(Type::Flying, def_types[1])
    } else {
        4 // 1x
    };
    
    // Flying total: (e1 * e2) / 4
    let flying_total = (flying_eff_1 as u16 * flying_eff_2 as u16) / 4;
    
    // Combined: (Fighting * Flying) / 4
    ((effectiveness as u16 * flying_total) / 4) as u8
}

// ============================================================================
// Thousand Arrows: Hits Flying neutral 
// ============================================================================

pub fn thousand_arrows_effectiveness(
    state: &BattleState,
    _attacker: usize,
    defender: usize,
    _move_data: &'static Move,
    effectiveness: u8,
    type_chart: &dyn Fn(Type, Type) -> u8,
) -> u8 {
    let def_types = state.types[defender];
    let t1 = def_types[0];
    let t2 = if def_types[1] != def_types[0] { Some(def_types[1]) } else { None };

    if t1 == Type::Flying || t2 == Some(Type::Flying) {
         // Re-calculate with Flying treated as Neutral (Normal)
         let eff1 = if t1 == Type::Flying { 
             4 // 1x
         } else {
             type_chart(Type::Ground, t1)
         };
         
         let eff2 = if let Some(t) = t2 {
             if t == Type::Flying {
                 4 // 1x
             } else {
                 type_chart(Type::Ground, t)
             }
         } else {
             4 // 1x
         };
         
         return ((eff1 as u16 * eff2 as u16) / 4) as u8;
    }
    effectiveness
}

// ============================================================================
// Facade: 2x if burned, poisoned, or paralyzed
// ============================================================================

pub fn facade_condition(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
) -> bool {
    let status = state.status[attacker];
    status.intersects(Status::BURN | Status::POISON | Status::TOXIC | Status::PARALYSIS)
}

pub fn on_ignore_status_damage_reduction_facade(
    _state: &BattleState,
    _attacker: usize,
    status: Status,
) -> bool {
    status == Status::BURN
}

// ============================================================================
// Weight-Based Moves Implementation
// ============================================================================

/// Calculate effective weight of an entity, applying modifiers.
/// If attacker has Mold Breaker, weight-modifying abilities are bypassed.
fn get_modified_weight(
    state: &BattleState, 
    entity_idx: usize, 
    entity_ability: AbilityId,
    attacker_ability: AbilityId,
) -> u32 {
    let mut weight = state.weight[entity_idx] as u32;
    if weight == 0 {
        weight = state.species[entity_idx].data().weight as u32;
    }

    // Check if attacker has Mold Breaker (bypasses target's abilities)
    let has_mold_breaker = attacker_ability.flags().contains(AbilityFlags::MOLD_BREAKER);

    // Apply ability modifiers (unless bypassed by Mold Breaker)
    if !has_mold_breaker {
        if entity_ability == AbilityId::Heavymetal {
            weight *= 2;
        } else if entity_ability == AbilityId::Lightmetal {
            weight /= 2;
        }
    }

    // Apply item modifiers (items are NOT bypassed by Mold Breaker)
    if state.items[entity_idx] == ItemId::Floatstone {
        weight /= 2;
    }

    weight.max(1)
}

// Grass Knot / Low Kick: BP based on target's weight
pub fn grass_knot_power(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let attacker_ability = state.abilities[attacker];
    let defender_ability = state.abilities[defender];
    
    let weight = get_modified_weight(state, defender, defender_ability, attacker_ability);
    match weight {
        w if w >= 2000 => 120, // >= 200kg
        w if w >= 1000 => 100, // >= 100kg
        w if w >= 500 => 80,   // >= 50kg
        w if w >= 250 => 60,   // >= 25kg
        w if w >= 100 => 40,   // >= 10kg
        _ => 20,               // < 10kg
    }
}

// Heavy Slam / Heat Crash: BP based on weight ratio
pub fn heavy_slam_power(
    state: &BattleState,
    attacker: usize,
    defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let attacker_ability = state.abilities[attacker];
    let defender_ability = state.abilities[defender];

    let attacker_weight = get_modified_weight(state, attacker, attacker_ability, AbilityId::Noability);
    let defender_weight = get_modified_weight(state, defender, defender_ability, attacker_ability);
    
    let ratio_x10 = (attacker_weight * 10) / defender_weight;
    match ratio_x10 {
        r if r >= 50 => 120, // >= 5x
        r if r >= 40 => 100, // >= 4x
        r if r >= 30 => 80,  // >= 3x
        r if r >= 20 => 60,  // >= 2x
        _ => 40,             // < 2x
    }
}

// Eruption / Water Spout: HP-based
pub fn eruption_power(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let current_hp = state.hp[attacker] as u32;
    let max_hp = state.max_hp[attacker] as u32;
    (150 * current_hp / max_hp.max(1)).max(1) as u16
}

// Flail / Reversal: Low HP based
pub fn flail_power(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let current_hp = state.hp[attacker] as u32;
    let max_hp = state.max_hp[attacker] as u32;
    let hp_percent = (current_hp * 48) / max_hp.max(1);
    match hp_percent {
        0..=1 => 200,   // < 4.17%
        2..=4 => 150,   // < 10.42%
        5..=9 => 100,   // < 20.83%
        10..=16 => 80,  // < 35.42%
        17..=32 => 40,  // < 68.75%
        _ => 20,        // >= 68.75%
    }
}
// Return: BP = Happiness / 2.5 (Max 102)
pub fn return_power(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let happiness = state.happiness[attacker] as u16;
    let power = happiness as u32 * 10 / 25; // x 0.4
    power.max(1) as u16
}

// Frustration: BP = (255 - Happiness) / 2.5 (Max 102)
pub fn frustration_power(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    _bp: u16,
) -> u16 {
    let happiness = state.happiness[attacker] as u16;
    let power = (255 - happiness as u32) * 10 / 25; // x 0.4
    power.max(1) as u16
}

// ============================================================================
// Judgment / Techno Blast / Multi-Attack: Type changes based on held item
// ============================================================================

/// Maps a Plate item to its corresponding Type.
fn plate_to_type(item: ItemId) -> Option<Type> {
    match item {
        ItemId::Flameplate => Some(Type::Fire),
        ItemId::Splashplate => Some(Type::Water),
        ItemId::Meadowplate => Some(Type::Grass),
        ItemId::Zapplate => Some(Type::Electric),
        ItemId::Icicleplate => Some(Type::Ice),
        ItemId::Fistplate => Some(Type::Fighting),
        ItemId::Toxicplate => Some(Type::Poison),
        ItemId::Earthplate => Some(Type::Ground),
        ItemId::Skyplate => Some(Type::Flying),
        ItemId::Mindplate => Some(Type::Psychic),
        ItemId::Insectplate => Some(Type::Bug),
        ItemId::Stoneplate => Some(Type::Rock),
        ItemId::Spookyplate => Some(Type::Ghost),
        ItemId::Dracoplate => Some(Type::Dragon),
        ItemId::Dreadplate => Some(Type::Dark),
        ItemId::Ironplate => Some(Type::Steel),
        ItemId::Pixieplate => Some(Type::Fairy),
        _ => None,
    }
}

/// Judgment: Type changes based on held Plate.
pub fn on_modify_type_judgment(
    state: &BattleState,
    attacker: usize,
    _defender: usize,
    _move_data: &'static Move,
    base_type: Type,
) -> Type {
    let item = state.items[attacker];
    plate_to_type(item).unwrap_or(base_type)
}

