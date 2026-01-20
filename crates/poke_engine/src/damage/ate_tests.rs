use crate::state::{BattleState, TurnOrder};
use crate::damage::{DamageContext, Gen9, compute_base_power};
use crate::species::SpeciesId;
use crate::types::Type;
use crate::abilities::AbilityId;
use crate::moves::MoveId;

#[test]
fn test_pixilate() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Attacker: Sylveon with Pixilate
    let attacker = 0;
    state.species[attacker] = SpeciesId::from_str("sylveon").unwrap_or(SpeciesId(700));
    state.types[attacker] = [Type::Fairy, Type::Fairy];
    state.abilities[attacker] = AbilityId::Pixilate;
    state.stats[attacker][1] = 100; // Atk

    // Defender: Dragonite (Dragon/Flying)
    let defender = 6;
    state.species[defender] = SpeciesId::from_str("dragonite").unwrap_or(SpeciesId(149));
    state.types[defender] = [Type::Dragon, Type::Flying];
    state.stats[defender][2] = 100; // Def

    // Move: Tackle (Normal, BP 40)
    let move_id = MoveId::Tackle;

    let mut ctx = DamageContext::new(gen, &state, attacker, defender, move_id, false);

    // Check type change
    assert_eq!(ctx.move_type, Type::Fairy, "Pixilate should change Normal move to Fairy");

    // Check Effectiveness (Fairy vs Dragon/Flying) -> 2x * 1x = 2x
    assert_eq!(ctx.effectiveness, 8, "Fairy vs Dragon should be 2x");

    // Check Base Power Boost
    compute_base_power(&mut ctx);
    // Tackle BP 40.
    // Pixilate: 40 * 1.2 = 48.
    // 40 * 4915 / 4096 = 48.
    assert_eq!(ctx.base_power, 48, "Pixilate should boost converted move BP by 1.2x");
}

#[test]
fn test_pixilate_no_boost_on_fairy() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Attacker: Sylveon with Pixilate
    let attacker = 0;
    state.species[attacker] = SpeciesId::from_str("sylveon").unwrap_or(SpeciesId(700));
    state.types[attacker] = [Type::Fairy, Type::Fairy];
    state.abilities[attacker] = AbilityId::Pixilate;

    // Defender
    let defender = 6;
    state.types[defender] = [Type::Normal, Type::Normal];

    // Move: Moonblast (Fairy, BP 95)
    let move_id = MoveId::Moonblast;

    let mut ctx = DamageContext::new(gen, &state, attacker, defender, move_id, false);

    assert_eq!(ctx.move_type, Type::Fairy);

    compute_base_power(&mut ctx);
    assert_eq!(ctx.base_power, 95, "Pixilate should NOT boost already Fairy moves");
}

#[test]
fn test_analytic() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Attacker: Magnezone with Analytic
    let attacker = 0;
    state.species[attacker] = SpeciesId::from_str("magnezone").unwrap_or(SpeciesId(462));
    state.types[attacker] = [Type::Electric, Type::Steel];
    state.abilities[attacker] = AbilityId::Analytic;
    state.stats[attacker][3] = 100; // SpA

    // Defender
    let defender = 6;
    state.types[defender] = [Type::Normal, Type::Normal];
    state.stats[defender][4] = 100; // SpD

    let move_id = MoveId::Thunderbolt; // BP 90

    // Scenario 1: Attacker is Faster (Moves First) -> No Boost
    state.stats[attacker][5] = 100; // Spe
    state.stats[defender][5] = 50;  // Spe

    {
        let mut ctx = DamageContext::new(gen, &state, attacker, defender, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 90, "Analytic should NOT boost if moving first");
    }

    // Scenario 2: Attacker is Slower (Moves Last) -> Boost 1.3x
    state.stats[attacker][5] = 50;
    state.stats[defender][5] = 100;

    {
        let mut ctx = DamageContext::new(gen, &state, attacker, defender, move_id, false);
        compute_base_power(&mut ctx);
        // 90 * 1.3 = 117.
        // 90 * 5325 / 4096 = 117.
        assert_eq!(ctx.base_power, 117, "Analytic should boost if moving last");
    }

    // Scenario 3: Attacker is Slower but uses Priority Move -> First -> No Boost
    // Quick Attack (+1 Priority)
    let priority_move = MoveId::Quickattack;
    {
        let mut ctx = DamageContext::new(gen, &state, attacker, defender, priority_move, false);
        compute_base_power(&mut ctx);
        // Should use base power of Quick Attack (40) without boost
        assert_eq!(ctx.base_power, 40, "Analytic should NOT boost if moving first due to priority");
    }
}
