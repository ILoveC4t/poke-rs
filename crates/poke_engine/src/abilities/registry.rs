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
    registry[AbilityId::Reckless as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::reckless),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Steelworker as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::steelworker),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Waterbubble as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::water_bubble),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Punkrock as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::punk_rock),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Rivalry as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::rivalry),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sheerforce as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::sheer_force),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sandforce as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::sand_force),
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
    registry[AbilityId::Sniper as usize] = Some(AbilityHooks {
        on_attacker_final_mod: Some(final_modifiers::sniper),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Defender Final Modifiers (OnDefenderFinalMod)
    // =========================================================================
    registry[AbilityId::Multiscale as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::multiscale),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Shadowshield as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::multiscale),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Filter as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::filter),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Fluffy as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::fluffy),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Solidrock as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::filter),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Prismarmor as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::filter),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Type Immunity (OnTypeImmunity)
    // =========================================================================
    registry[AbilityId::Levitate as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::levitate),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Flashfire as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::flash_fire),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Voltabsorb as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::volt_absorb),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Waterabsorb as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::water_absorb),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Stormdrain as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::storm_drain),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Lightningrod as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::lightning_rod),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sapsipper as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::sap_sipper),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Motordrive as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::motor_drive),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Dryskin as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::dry_skin),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Eartheater as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::earth_eater),
        ..AbilityHooks::NONE
    });

    registry
};
