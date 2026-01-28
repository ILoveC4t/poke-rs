//! Correctness tests for Critical Hit mechanics in Gen 1 and 2.
//!
//! These tests verify that our engine correctly implements the specific
//! critical hit rules for early generations, which differ from modern mechanics:
//! - Gen 1: Crit ignores ALL stat modifications (positive and negative).
//! - Gen 1: Crit ignores Burn attack drop.
//! - Gen 2: Crit ignores ALL stat modifications (positive and negative).
//! - Gen 2: Crit ignores Burn attack drop.
//!
//! These tests are necessary because Smogon's damage calculator fixtures
//! for these cases appear to expect behavior that contradicts cartridge mechanics
//! (e.g. applying burn or screens on crit), likely due to defaults or Stadium mechanics.

use poke_engine::damage::calculate_damage;
use poke_engine::damage::generations::{Gen1, Gen2};
use poke_engine::moves::MoveId;
use poke_engine::species::SpeciesId;
use poke_engine::state::{BattleState, Status};
use poke_engine::types::Type;

#[test]
fn test_gen1_crit_positive_boost() {
    let mut state = BattleState::new();
    let gen = Gen1;

    // Attacker: Rattata (Normal)
    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.boosts[0][0] = 2; // +2 Atk (2.0x)

    // Defender: Rattata (Normal)
    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Gen 1 Crit should IGNORE the positive boost.
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    // Create unboosted state
    let mut state_unboosted = state;
    state_unboosted.boosts[0][0] = 0;
    let result_unboosted = calculate_damage(gen, &state_unboosted, 0, 6, move_id, true);

    println!("Gen 1 Crit +2 Atk Max Damage: {}", result.max);
    println!("Gen 1 Crit +0 Atk Max Damage: {}", result_unboosted.max);

    assert_eq!(
        result.max, result_unboosted.max,
        "Gen 1 Crit should ignore positive attack boosts"
    );
}

#[test]
fn test_gen1_crit_negative_boost() {
    let mut state = BattleState::new();
    let gen = Gen1;

    // Attacker: Rattata
    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.boosts[0][0] = -2; // -2 Atk (0.5x)

    // Defender: Rattata
    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Crit should IGNORE negative boost.
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    // Unboosted
    let mut state_unboosted = state;
    state_unboosted.boosts[0][0] = 0;
    let result_unboosted = calculate_damage(gen, &state_unboosted, 0, 6, move_id, true);

    println!("Gen 1 Crit -2 Atk Max Damage: {}", result.max);
    println!("Gen 1 Crit +0 Atk Max Damage: {}", result_unboosted.max);

    assert_eq!(
        result.max, result_unboosted.max,
        "Gen 1 Crit should ignore negative attack boosts"
    );
}

#[test]
fn test_gen1_non_crit_burn() {
    let mut state = BattleState::new();
    let gen = Gen1;

    // Attacker: Rattata
    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.status[0] = Status::BURN;

    // Defender: Rattata
    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle; // Physical

    // Non-Crit: Should halve attack (damage ~0.5x)
    let result = calculate_damage(gen, &state, 0, 6, move_id, false);

    // No Burn
    let mut state_no_burn = state;
    state_no_burn.status[0] = Status::NONE;
    let result_no_burn = calculate_damage(gen, &state_no_burn, 0, 6, move_id, false);

    println!("Gen 1 Burned Max Damage: {}", result.max);
    println!("Gen 1 Normal Max Damage: {}", result_no_burn.max);

    // Gen1 uses truncated integer division, so we allow small rounding tolerance
    assert!(
        result.max >= result_no_burn.max / 2 - 2 && result.max <= result_no_burn.max / 2 + 2,
        "Gen 1 Burn should halve damage (within ±2 for rounding)"
    );
}

#[test]
fn test_gen1_crit_burn() {
    let mut state = BattleState::new();
    let gen = Gen1;

    // Attacker: Rattata
    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.status[0] = Status::BURN;

    // Defender: Rattata
    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Crit: Should IGNORE burn (damage equal to unburned crit)
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    // No Burn Crit
    let mut state_no_burn = state;
    state_no_burn.status[0] = Status::NONE;
    let result_no_burn = calculate_damage(gen, &state_no_burn, 0, 6, move_id, true);

    println!("Gen 1 Crit Burned Max Damage: {}", result.max);
    println!("Gen 1 Crit Normal Max Damage: {}", result_no_burn.max);

    assert_eq!(
        result.max, result_no_burn.max,
        "Gen 1 Crit should ignore burn"
    );
}

#[test]
fn test_gen2_crit_burn() {
    let mut state = BattleState::new();
    let gen = Gen2;

    // Attacker: Rattata
    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.status[0] = Status::BURN;

    // Defender: Rattata
    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Crit: Should IGNORE burn
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    // No Burn Crit
    let mut state_no_burn = state;
    state_no_burn.status[0] = Status::NONE;
    let result_no_burn = calculate_damage(gen, &state_no_burn, 0, 6, move_id, true);

    println!("Gen 2 Crit Burned Max Damage: {}", result.max);
    println!("Gen 2 Crit Normal Max Damage: {}", result_no_burn.max);

    assert_eq!(
        result.max, result_no_burn.max,
        "Gen 2 Crit should ignore burn"
    );
}

#[test]
fn test_gen2_crit_positive_boost() {
    let mut state = BattleState::new();
    let gen = Gen2;

    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.boosts[0][0] = 2; // +2 Atk

    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Gen 2 Crit should IGNORE positive boost
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    let mut state_unboosted = state;
    state_unboosted.boosts[0][0] = 0;
    let result_unboosted = calculate_damage(gen, &state_unboosted, 0, 6, move_id, true);

    println!("Gen 2 Crit +2 Atk Max Damage: {}", result.max);
    println!("Gen 2 Crit +0 Atk Max Damage: {}", result_unboosted.max);

    assert_eq!(
        result.max, result_unboosted.max,
        "Gen 2 Crit should ignore positive attack boosts"
    );
}

#[test]
fn test_gen2_crit_negative_boost() {
    let mut state = BattleState::new();
    let gen = Gen2;

    state.species[0] = SpeciesId::from_str("rattata").unwrap();
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.boosts[0][0] = -2; // -2 Atk

    state.species[6] = SpeciesId::from_str("rattata").unwrap();
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][2] = 100; // Def

    let move_id = MoveId::Tackle;

    // Gen 2 Crit should IGNORE negative boost
    let result = calculate_damage(gen, &state, 0, 6, move_id, true);

    let mut state_unboosted = state;
    state_unboosted.boosts[0][0] = 0;
    let result_unboosted = calculate_damage(gen, &state_unboosted, 0, 6, move_id, true);

    assert_eq!(
        result.max, result_unboosted.max,
        "Gen 2 Crit should ignore negative attack boosts"
    );
}
