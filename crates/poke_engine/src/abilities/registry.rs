use crate::abilities::AbilityId;
use crate::abilities::hooks::AbilityHooks;
use crate::abilities::implementations::{weather_setters, priority, intimidate};

pub static ABILITY_REGISTRY: [Option<AbilityHooks>; AbilityId::COUNT] = {
    let mut registry: [Option<AbilityHooks>; AbilityId::COUNT] = [None; AbilityId::COUNT];

    // Weather / Terrain
    registry[AbilityId::Drizzle as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::drizzle),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Drought as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::drought),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sandstream as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::sand_stream),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Snowwarning as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::snow_warning),
        ..AbilityHooks::NONE
    });

    registry[AbilityId::Electricsurge as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::electric_surge),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Grassysurge as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::grassy_surge),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Mistysurge as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::misty_surge),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Psychicsurge as usize] = Some(AbilityHooks {
        on_switch_in: Some(weather_setters::psychic_surge),
        ..AbilityHooks::NONE
    });

    // Priority
    registry[AbilityId::Prankster as usize] = Some(AbilityHooks {
        on_modify_priority: Some(priority::prankster),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Galewings as usize] = Some(AbilityHooks {
        on_modify_priority: Some(priority::gale_wings),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Triage as usize] = Some(AbilityHooks {
        on_modify_priority: Some(priority::triage),
        ..AbilityHooks::NONE
    });

    // Intimidate
    registry[AbilityId::Intimidate as usize] = Some(AbilityHooks {
        on_switch_in: Some(intimidate::intimidate),
        ..AbilityHooks::NONE
    });

    registry
};
