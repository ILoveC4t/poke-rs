use super::*;
use crate::moves::MoveId;
use crate::state::BattleState;

#[test]
fn test_multitype_plate_to_type() {
    use super::implementations::multitype::plate_to_type;
    use crate::items::ItemId;
    use crate::types::Type;

    assert_eq!(plate_to_type(ItemId::Meadowplate), Some(Type::Grass));
    assert_eq!(plate_to_type(ItemId::Flameplate), Some(Type::Fire));
    assert_eq!(plate_to_type(ItemId::Splashplate), Some(Type::Water));
    assert_eq!(plate_to_type(ItemId::None), None);
}

#[test]
fn test_registry_lookup() {
    // Drizzle should have a hook
    let drizzle = AbilityId::Drizzle;
    let hook = ABILITY_REGISTRY[drizzle as usize];
    assert!(hook.is_some());
    assert!(hook.unwrap().on_switch_in.is_some());

    // Noability should be None
    let no_ability = AbilityId::Noability;
    let hook = ABILITY_REGISTRY[no_ability as usize];
    assert!(hook.is_none());
}

#[test]
fn test_drizzle_hook() {
    let mut state = BattleState::new();
    let idx = 0;

    // Initial state
    assert_eq!(state.weather, 0);

    // Call hook manually (simulating engine call)
    let drizzle = AbilityId::Drizzle;
    if let Some(hook) = ABILITY_REGISTRY[drizzle as usize] {
        if let Some(on_switch_in) = hook.on_switch_in {
            on_switch_in(&mut state, idx);
        }
    }

    assert_eq!(state.weather, Weather::Rain as u8);
    assert_eq!(state.weather_turns, 5);
}

#[test]
fn test_prankster_hook() {
    let state = BattleState::new();
    let idx = 0;

    // "thunderwave" is status, "thunderbolt" is special.
    let thunder_wave = MoveId::from_str("thunderwave").expect("thunderwave exists");
    let thunderbolt = MoveId::from_str("thunderbolt").expect("thunderbolt exists");

    let prankster = AbilityId::Prankster;
    let hook = ABILITY_REGISTRY[prankster as usize].unwrap();
    let on_modify_priority = hook.on_modify_priority.unwrap();

    // Status move -> +1
    let p1 = on_modify_priority(&state, idx, thunder_wave, 0);
    assert_eq!(p1, 1);

    // Damage move -> +0
    let p2 = on_modify_priority(&state, idx, thunderbolt, 0);
    assert_eq!(p2, 0);
}
