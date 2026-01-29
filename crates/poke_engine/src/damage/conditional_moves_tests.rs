use crate::damage::{compute_base_power, DamageContext, Gen9};
use crate::moves::MoveId;
use crate::species::SpeciesId;
use crate::state::{BattleState, Status};
use crate::types::Type;

#[test]
fn test_venoshock() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Setup: Atk 100, Def 100
    state.species[0] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
    state.types[0] = [Type::Poison, Type::Poison];
    state.stats[0][3] = 100; // SpA

    state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][4] = 100; // SpD

    let move_id = MoveId::Venoshock; // Poison, 65 BP

    // Case 1: No Status (1x)
    {
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        // Base power should be 65
        assert_eq!(
            ctx.base_power, 65,
            "Venoshock BP should be 65 without status"
        );
    }

    // Case 2: Poisoned (2x)
    {
        state.status[6] = Status::POISON;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        // 65 * 2 = 130
        assert_eq!(
            ctx.base_power, 130,
            "Venoshock BP should double when target is poisoned"
        );
    }

    // Case 3: Toxic (2x)
    {
        state.status[6] = Status::TOXIC;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(
            ctx.base_power, 130,
            "Venoshock BP should double when target is badly poisoned"
        );
    }

    // Case 4: Other status (1x)
    {
        state.status[6] = Status::BURN;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(
            ctx.base_power, 65,
            "Venoshock BP should NOT double for other status"
        );
    }
}

#[test]
fn test_hex() {
    let mut state = BattleState::new();
    let gen = Gen9;

    state.species[0] = SpeciesId::from_str("gengar").unwrap_or(SpeciesId(94));
    state.types[0] = [Type::Ghost, Type::Poison];
    state.stats[0][3] = 100; // SpA

    state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][4] = 100; // SpD

    let move_id = MoveId::Hex; // Ghost, 65 BP

    // Case 1: No Status (1x)
    {
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 65, "Hex BP should be 65 without status");
    }

    // Case 2: Burned (2x)
    {
        state.status[6] = Status::BURN;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(
            ctx.base_power, 130,
            "Hex BP should double when target is burned"
        );
    }

    // Case 3: Sleep (2x)
    {
        state.status[6] = Status::SLEEP;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(
            ctx.base_power, 130,
            "Hex BP should double when target is asleep"
        );
    }
}

#[test]
fn test_brine() {
    let mut state = BattleState::new();
    let gen = Gen9;

    state.species[0] = SpeciesId::from_str("squirtle").unwrap_or(SpeciesId(7));
    state.types[0] = [Type::Water, Type::Water];
    state.stats[0][3] = 100; // SpA

    state.species[6] = SpeciesId::from_str("rattata").unwrap_or(SpeciesId(19));
    state.types[6] = [Type::Normal, Type::Normal];
    state.stats[6][4] = 100; // SpD
    state.max_hp[6] = 100;
    state.hp[6] = 100;

    let move_id = MoveId::Brine; // Water, 65 BP

    // Case 1: Full HP (1x)
    {
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 65, "Brine BP should be 65 at full HP");
    }

    // Case 2: 51% HP (1x)
    {
        state.hp[6] = 51;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 65, "Brine BP should be 65 at 51% HP");
    }

    // Case 3: 50% HP (2x)
    {
        state.hp[6] = 50;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 130, "Brine BP should double at 50% HP");
    }

    // Case 4: 1 HP (2x)
    {
        state.hp[6] = 1;
        let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
        compute_base_power(&mut ctx);
        assert_eq!(ctx.base_power, 130, "Brine BP should double at 1 HP");
    }
}
