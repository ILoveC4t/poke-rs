//! Tests that verify correct Multitype implementation.
//!
//! These tests verify that our engine correctly implements Multitype's type change
//! and STAB calculation, which differs from smogon's damage-calc behavior.
//!
//! Smogon's calc does NOT apply Multitype's type change when using base "Arceus",
//! so their fixtures expect no STAB. However, in actual games, Arceus + Plate
//! does change type and DOES get STAB on Judgment.

use poke_engine::abilities::AbilityId;
use poke_engine::damage::{calculate_damage, generations::{Gen4, Gen5, Gen6, Gen7}};
use poke_engine::entities::PokemonConfig;
use poke_engine::items::ItemId;
use poke_engine::moves::MoveId;
use poke_engine::species::SpeciesId;
use poke_engine::state::BattleState;
use poke_engine::types::Type;

#[test]
fn test_multitype_changes_arceus_type() {
    let mut state = BattleState::default();
    
    // Arceus with Meadow Plate should become Grass type
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .item(ItemId::Meadowplate)
        .spawn(&mut state, 0, 0);
    
    let attacker = 0;
    assert_eq!(state.types[attacker], [Type::Grass, Type::Grass],
        "Multitype should change Arceus to Grass type when holding Meadow Plate");
}

#[test]
fn test_multitype_no_plate_stays_normal() {
    let mut state = BattleState::default();
    
    // Arceus without a Plate should stay Normal type
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .spawn(&mut state, 0, 0);
    
    let attacker = 0;
    assert_eq!(state.types[attacker], [Type::Normal, Type::Normal],
        "Arceus without a Plate should stay Normal type");
}

#[test]
fn test_arceus_judgment_stab_gen4() {
    let mut state = BattleState::default();
    
    // Arceus with Meadow Plate and Multitype
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .item(ItemId::Meadowplate)
        .spawn(&mut state, 0, 0);
    
    // Blastoise (Water type, weak to Grass)
    PokemonConfig::new(SpeciesId::from_str("blastoise").unwrap())
        .level(100)
        .ability(AbilityId::Torrent)
        .spawn(&mut state, 1, 0);
    
    let attacker = 0;
    let defender = BattleState::entity_index(1, 0);
    
    // Verify type was changed
    assert_eq!(state.types[attacker], [Type::Grass, Type::Grass]);
    
    // Calculate damage with STAB
    let result = calculate_damage(Gen4, &state, attacker, defender, MoveId::Judgment, false);
    
    // Expected damage WITH STAB: ~290-344 (smogon expects 194-230 without STAB)
    assert!(result.rolls[0] >= 285 && result.rolls[0] <= 295,
        "Gen4: First damage roll should be ~290 with STAB, got {}", result.rolls[0]);
    assert!(result.rolls[15] >= 340 && result.rolls[15] <= 350,
        "Gen4: Last damage roll should be ~344 with STAB, got {}", result.rolls[15]);
}

#[test]
fn test_arceus_judgment_stab_gen5() {
    let mut state = BattleState::default();
    
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .item(ItemId::Meadowplate)
        .spawn(&mut state, 0, 0);
    
    PokemonConfig::new(SpeciesId::from_str("blastoise").unwrap())
        .level(100)
        .ability(AbilityId::Torrent)
        .spawn(&mut state, 1, 0);
    
    let attacker = 0;
    let defender = BattleState::entity_index(1, 0);
    
    assert_eq!(state.types[attacker], [Type::Grass, Type::Grass]);
    
    let result = calculate_damage(Gen5, &state, attacker, defender, MoveId::Judgment, false);
    
    assert!(result.rolls[0] >= 285 && result.rolls[0] <= 295,
        "Gen5: First damage roll should be ~290 with STAB, got {}", result.rolls[0]);
}

#[test]
fn test_arceus_judgment_stab_gen6() {
    let mut state = BattleState::default();
    
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .item(ItemId::Meadowplate)
        .spawn(&mut state, 0, 0);
    
    PokemonConfig::new(SpeciesId::from_str("blastoise").unwrap())
        .level(100)
        .ability(AbilityId::Torrent)
        .spawn(&mut state, 1, 0);
    
    let attacker = 0;
    let defender = BattleState::entity_index(1, 0);
    
    assert_eq!(state.types[attacker], [Type::Grass, Type::Grass]);
    
    let result = calculate_damage(Gen6, &state, attacker, defender, MoveId::Judgment, false);
    
    assert!(result.rolls[0] >= 285 && result.rolls[0] <= 295,
        "Gen6: First damage roll should be ~290 with STAB, got {}", result.rolls[0]);
}

#[test]
fn test_arceus_judgment_stab_gen7() {
    let mut state = BattleState::default();
    
    PokemonConfig::new(SpeciesId::from_str("arceus").unwrap())
        .level(100)
        .ability(AbilityId::Multitype)
        .item(ItemId::Meadowplate)
        .spawn(&mut state, 0, 0);
    
    PokemonConfig::new(SpeciesId::from_str("blastoise").unwrap())
        .level(100)
        .ability(AbilityId::Torrent)
        .spawn(&mut state, 1, 0);
    
    let attacker = 0;
    let defender = BattleState::entity_index(1, 0);
    
    assert_eq!(state.types[attacker], [Type::Grass, Type::Grass]);
    
    let result = calculate_damage(Gen7, &state, attacker, defender, MoveId::Judgment, false);
    
    assert!(result.rolls[0] >= 285 && result.rolls[0] <= 295,
        "Gen7: First damage roll should be ~290 with STAB, got {}", result.rolls[0]);
}
