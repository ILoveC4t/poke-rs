use super::*;
use crate::damage::{DamageContext, Gen9};
use crate::moves::MoveId;
use crate::species::SpeciesId;
use crate::state::BattleState;
use crate::state::Status;
use crate::types::Type;

#[test]
fn test_fixed_damage_moves() {
    let mut state = BattleState::new();
    // Attacker: Level 50
    state.level[0] = 50;
    state.hp[0] = 100;
    state.max_hp[0] = 100;

    // Defender: Ghost type
    state.types[6] = [Type::Ghost, Type::Ghost];
    state.hp[6] = 100;
    state.max_hp[6] = 100;

    // Seismic Toss vs Ghost (Immune)
    let move_id = MoveId::Seismictoss;
    assert_eq!(fixed::get_fixed_damage(move_id, &state, 0, 6), Some(0));

    // Seismic Toss vs Normal (Level damage)
    state.types[6] = [Type::Normal, Type::Normal];
    assert_eq!(fixed::get_fixed_damage(move_id, &state, 0, 6), Some(50));

    // Super Fang vs Ghost (Immune)
    let move_id = MoveId::Superfang;
    state.types[6] = [Type::Ghost, Type::Ghost];
    assert_eq!(fixed::get_fixed_damage(move_id, &state, 0, 6), Some(0));

    // Super Fang vs Normal (50% HP)
    state.types[6] = [Type::Normal, Type::Normal];
    assert_eq!(fixed::get_fixed_damage(move_id, &state, 0, 6), Some(50));

    // Dragon Rage vs Fairy (Immune)
    let move_id = MoveId::Dragonrage;
    state.types[6] = [Type::Fairy, Type::Fairy];
    assert_eq!(fixed::get_fixed_damage(move_id, &state, 0, 6), Some(0));
}

#[test]
fn test_base_power_modifiers() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Eruption: HP-based
    state.species[0] = SpeciesId::from_str("typhlosion").unwrap_or_else(|| SpeciesId(157));
    state.hp[0] = 150; // Full HP
    state.max_hp[0] = 150;

    let move_id = MoveId::Eruption;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 150);

    // Eruption: 1 HP
    state.hp[0] = 1;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 1);

    // Facade: Burned
    state.hp[0] = 100;
    state.status[0] = Status::BURN;
    let move_id = MoveId::Facade;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

    // Default Facade BP is 70
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 140);
}

#[test]
fn test_grass_knot() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Snorlax - Heavy (460.0 kg = 4600 units)
    // Manually set weight since power.rs now uses state.weight
    state.weight[6] = 4600;

    let move_id = MoveId::Grassknot;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);

    // 460kg >= 200kg -> 120 BP
    assert_eq!(bp, 120, "Grass Knot vs 460kg target should be 120 BP");
}

#[test]
fn test_reckless_ability() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Attacker has Reckless
    use crate::abilities::AbilityId;
    state.abilities[0] = AbilityId::Reckless;

    // Double-Edge (Recoil move) - BP 120
    let move_id = MoveId::Doubleedge;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    // Move flags must be set. `ctx.move_data` is from `move_id.data()`.
    // Double-Edge data should have recoil flag if build.rs worked.

    // Initialize BP
    ctx.base_power = 120;

    // Call compute_base_power (public function)
    crate::damage::modifiers::compute_base_power(&mut ctx);

    let expected = (120u32 * 4915 / 4096) as u16; // ~144
    assert_eq!(
        ctx.base_power, expected,
        "Reckless should boost recoil moves by 1.2x"
    );

    // Tackle (No recoil) - BP 40
    let move_id = MoveId::Tackle;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    ctx.base_power = 40;
    crate::damage::modifiers::compute_base_power(&mut ctx);
    assert_eq!(
        ctx.base_power, 40,
        "Reckless should not boost non-recoil moves"
    );
}

#[test]
fn test_weight_modifiers() {
    let mut state = BattleState::new();
    let gen = Gen9;

    // Test Case 1: Heavy Metal (x2)
    // Base 100kg (1000 units) -> 200kg (2000 units) -> Grass Knot BP 120 (>=200kg)
    state.weight[6] = 1000;
    state.abilities[6] = crate::abilities::AbilityId::Heavymetal;

    let move_id = MoveId::Grassknot;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);

    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 120, "Heavy Metal should double weight to 200kg");

    // Test Case 2: Light Metal (x0.5)
    // Base 100kg (1000 units) -> 50kg (500 units) -> Grass Knot BP 80 (>=50kg)
    state.abilities[6] = crate::abilities::AbilityId::Lightmetal;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 80, "Light Metal should halve weight to 50kg");

    // Test Case 3: Float Stone (x0.5)
    // Base 100kg -> 50kg -> BP 80
    state.abilities[6] = crate::abilities::AbilityId::Noability;
    state.items[6] = crate::items::ItemId::Floatstone;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(bp, 80, "Float Stone should halve weight to 50kg");

    // Test Case 4: Light Metal + Float Stone (x0.25)
    // Base 100kg -> 25kg -> BP 60 (>=25kg)
    state.abilities[6] = crate::abilities::AbilityId::Lightmetal;
    state.items[6] = crate::items::ItemId::Floatstone;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(
        bp, 60,
        "Light Metal + Float Stone should quarter weight to 25kg"
    );

    // Test Case 5: Heavy Slam
    // Attacker: 100kg + Heavy Metal -> 200kg
    // Defender: 50kg
    // Ratio: 200/50 = 4. BP = 100 (>=4x)

    state.weight[0] = 1000;
    state.abilities[0] = crate::abilities::AbilityId::Heavymetal;
    state.items[0] = crate::items::ItemId::default();

    state.weight[6] = 500;
    state.abilities[6] = crate::abilities::AbilityId::Noability;
    state.items[6] = crate::items::ItemId::default();

    let move_id = MoveId::Heavyslam;
    let mut ctx = DamageContext::new(gen, &state, 0, 6, move_id, false);
    let bp = power::modify_base_power(&mut ctx);
    assert_eq!(
        bp, 100,
        "Heavy Slam should account for attacker Heavy Metal"
    );
}
