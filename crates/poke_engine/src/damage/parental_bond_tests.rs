use crate::state::BattleState;
use crate::damage::{calculate_damage, Gen9};
use crate::species::SpeciesId;
use crate::abilities::AbilityId;
use crate::moves::MoveId;
use crate::types::Type;
use crate::damage::formula::apply_modifier;
use crate::damage::Modifier;

#[test]
fn test_parental_bond_standard_move() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Kangaskhan with Parental Bond
    // Using a species that can have it (Mega Kangaskhan usually, but we can force ability)
    state.species[0] = SpeciesId::from_str("kangaskhan").unwrap();
    state.abilities[0] = AbilityId::Parentalbond;
    state.types[0] = [Type::Normal, Type::Normal];
    state.stats[0][1] = 100; // Atk
    state.level[0] = 50;

    // Defender
    state.species[6] = SpeciesId::from_str("mew").unwrap();
    state.types[6] = [Type::Psychic, Type::Psychic];
    state.stats[6][2] = 100; // Def
    state.level[6] = 50;

    // Move: Tackle (40 BP, Normal)
    let move_id = MoveId::Tackle;

    // 1. Calculate with Parental Bond
    let result_pb = calculate_damage(gen, &state, 0, 6, move_id, false);

    // 2. Calculate without Parental Bond (Remove ability)
    state.abilities[0] = AbilityId::Noability;
    let result_normal = calculate_damage(gen, &state, 0, 6, move_id, false);

    // Check min damage
    let damage_normal = result_normal.min;
    let damage_pb = result_pb.min;

    // Expected: damage_normal + floor(damage_normal * 0.25)
    // Note: This assumes the random roll is the same (min roll) which is 85.
    // However, the second hit in PB is an independent roll.
    // For "min" calculation in our implementation, we sum (min1 + min2).
    // So min2 is calculated from base_damage using 85 roll, then modified.

    // We can verify roughly 1.25x
    // 40 BP -> Damage X.
    // PB: X + floor(X * 0.25).

    // Let's compute expected value manually using helper
    let expected_hit2 = apply_modifier(damage_normal as u32, Modifier::new(1024)) as u16; // 0.25x
    let expected_total = damage_normal + expected_hit2;

    assert_eq!(damage_pb, expected_total,
        "Parental Bond should add 25% damage (Normal: {}, PB: {}, Expected: {})",
        damage_normal, damage_pb, expected_total);
}

#[test]
fn test_parental_bond_fixed_damage() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Kangaskhan with Parental Bond
    state.species[0] = SpeciesId::from_str("kangaskhan").unwrap();
    state.abilities[0] = AbilityId::Parentalbond;
    state.level[0] = 100;

    // Defender
    state.species[6] = SpeciesId::from_str("mew").unwrap();
    state.types[6] = [Type::Psychic, Type::Psychic];
    state.hp[6] = 300; // Enough HP

    // Move: Seismic Toss (Fixed damage = Level)
    let move_id = MoveId::Seismictoss;

    // 1. Calculate with Parental Bond
    let result_pb = calculate_damage(gen, &state, 0, 6, move_id, false);

    // Expected: 100 + floor(100 * 0.25) = 125
    assert_eq!(result_pb.min, 125, "Seismic Toss should deal 125 damage with Parental Bond");

    // 2. Calculate without Parental Bond
    state.abilities[0] = AbilityId::Noability;
    let result_normal = calculate_damage(gen, &state, 0, 6, move_id, false);

    assert_eq!(result_normal.min, 100, "Seismic Toss should deal 100 damage without Parental Bond");
}

#[test]
fn test_parental_bond_exclusions() {
    let mut state = BattleState::new();
    let gen = Gen9;

    state.species[0] = SpeciesId::from_str("kangaskhan").unwrap();
    state.abilities[0] = AbilityId::Parentalbond;
    state.stats[0][1] = 100;

    state.species[6] = SpeciesId::from_str("mew").unwrap();
    state.stats[6][2] = 100;

    // 1. Multi-Hit Move: Double Kick
    // Should NOT receive PB boost (Double Kick hits 2 times normally, PB doesn't add a 3rd or boost them)
    // Our implementation: returns damage for ONE hit (standard calc) OR returns sum?
    // Wait, for Multi-Hit moves, `calculate_damage` normally returns damage for ONE hit.
    // If PB is excluded, it returns damage for ONE hit.
    // If PB applied, it would return combined damage.
    // We expect it to be excluded, so it returns damage for 1 hit.

    let double_kick = MoveId::Doublekick;

    // Check flags (sanity check, although we can't easily check flags in test without accessing Move data)
    // Assuming codegen worked and MultiHit flag is set.

    let result_mk = calculate_damage(gen, &state, 0, 6, double_kick, false);

    // Without ability
    state.abilities[0] = AbilityId::Noability;
    let result_mk_normal = calculate_damage(gen, &state, 0, 6, double_kick, false);

    assert_eq!(result_mk.min, result_mk_normal.min, "Multi-Hit moves should not be affected by Parental Bond");

    // 2. Spread Move: Earthquake (Target: allAdjacent)
    state.abilities[0] = AbilityId::Parentalbond;
    let earthquake = MoveId::Earthquake;

    let result_eq = calculate_damage(gen, &state, 0, 6, earthquake, false);

    state.abilities[0] = AbilityId::Noability;
    let result_eq_normal = calculate_damage(gen, &state, 0, 6, earthquake, false);

    assert_eq!(result_eq.min, result_eq_normal.min, "Spread moves should not be affected by Parental Bond");
}
