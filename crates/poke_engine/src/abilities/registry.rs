use crate::abilities::AbilityId;
use crate::abilities::hooks::AbilityHooks;
use crate::abilities::implementations::{
    weather_setters, priority, intimidate,
    damage_modifiers, stat_modifiers, final_modifiers, immunity,
};

pub static ABILITY_REGISTRY: [Option<AbilityHooks>; AbilityId::COUNT] = {
    let mut registry: [Option<AbilityHooks>; AbilityId::COUNT] = [None; AbilityId::COUNT];

    // =========================================================================
    // Weather / Terrain Setters
    // =========================================================================
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

    // =========================================================================
    // Priority Modifiers
    // =========================================================================
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

    // =========================================================================
    // Switch-in Effects
    // =========================================================================
    registry[AbilityId::Intimidate as usize] = Some(AbilityHooks {
        on_switch_in: Some(intimidate::intimidate),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Base Power Modifiers (OnModifyBasePower)
    // =========================================================================
    registry[AbilityId::Technician as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::technician),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Ironfist as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::iron_fist),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Toughclaws as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::tough_claws),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Stat Modifiers (OnModifyAttack / OnModifyDefense)
    // =========================================================================
    registry[AbilityId::Hustle as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::hustle),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Attacker Final Modifiers (OnAttackerFinalMod)
    // =========================================================================
    registry[AbilityId::Tintedlens as usize] = Some(AbilityHooks {
        on_attacker_final_mod: Some(final_modifiers::tinted_lens),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Defender Final Modifiers (OnDefenderFinalMod)
    // =========================================================================
    registry[AbilityId::Multiscale as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::multiscale),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Type Immunity (OnTypeImmunity)
    // =========================================================================
    registry[AbilityId::Levitate as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::levitate),
        ..AbilityHooks::NONE
    });

    registry
};
