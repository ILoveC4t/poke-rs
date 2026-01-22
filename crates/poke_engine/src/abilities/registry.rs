use crate::abilities::AbilityId;
use crate::abilities::hooks::AbilityHooks;
use crate::abilities::implementations::{
    weather_setters, priority, intimidate,
    damage_modifiers, stat_modifiers, final_modifiers, immunity, speed, status,
    type_changers,
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
    registry[AbilityId::Strongjaw as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::strong_jaw),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Megalauncher as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::mega_launcher),
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
    // Note: Guts is registered in the Stat Modifiers section below

    registry[AbilityId::Sheerforce as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::sheer_force),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sandforce as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::sand_force),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Analytic as usize] = Some(AbilityHooks {
        on_modify_base_power: Some(damage_modifiers::analytic),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Aerilate as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::aerilate),
        on_modify_base_power: Some(damage_modifiers::ate_boost),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Pixilate as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::pixilate),
        on_modify_base_power: Some(damage_modifiers::ate_boost),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Refrigerate as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::refrigerate),
        on_modify_base_power: Some(damage_modifiers::ate_boost),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Galvanize as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::galvanize),
        on_modify_base_power: Some(damage_modifiers::ate_boost),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Type Changers (OnModifyType only - no boost)
    // =========================================================================
    registry[AbilityId::Normalize as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::normalize),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Liquidvoice as usize] = Some(AbilityHooks {
        on_modify_type: Some(type_changers::liquid_voice),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Stat Modifiers (OnModifyAttack / OnModifyDefense)
    // =========================================================================
    registry[AbilityId::Hustle as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::hustle),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Purepower as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::huge_power),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Hugepower as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::huge_power),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Guts as usize] = Some(AbilityHooks {
        on_modify_attack: Some(status::on_modify_attack_guts),
        on_ignore_status_damage_reduction: Some(status::on_ignore_status_damage_reduction_guts),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Gorillatactics as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::gorilla_tactics),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Defeatist as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::defeatist),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Protosynthesis as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::protosynthesis),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Quarkdrive as usize] = Some(AbilityHooks {
        on_modify_attack: Some(stat_modifiers::quark_drive),
        ..AbilityHooks::NONE
    });

    registry[AbilityId::Furcoat as usize] = Some(AbilityHooks {
        on_modify_defense: Some(stat_modifiers::fur_coat),
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
    registry[AbilityId::Neuroforce as usize] = Some(AbilityHooks {
        on_attacker_final_mod: Some(final_modifiers::neuroforce),
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
    registry[AbilityId::Icescales as usize] = Some(AbilityHooks {
        on_defender_final_mod: Some(final_modifiers::ice_scales),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Type Immunity (OnTypeImmunity)
    // =========================================================================
    registry[AbilityId::Levitate as usize] = Some(AbilityHooks {
        on_type_immunity: Some(immunity::levitate),
        on_check_grounded: Some(immunity::levitate_grounding),
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

    registry[AbilityId::Magicguard as usize] = Some(AbilityHooks {
        on_hazard_immunity: Some(immunity::magic_guard_hazard_immunity),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Speed Modifiers (OnModifySpeed)
    // =========================================================================
    registry[AbilityId::Chlorophyll as usize] = Some(AbilityHooks {
        on_modify_speed: Some(speed::chlorophyll),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Swiftswim as usize] = Some(AbilityHooks {
        on_modify_speed: Some(speed::swift_swim),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Sandrush as usize] = Some(AbilityHooks {
        on_modify_speed: Some(speed::sand_rush),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Slushrush as usize] = Some(AbilityHooks {
        on_modify_speed: Some(speed::slush_rush),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Surgesurfer as usize] = Some(AbilityHooks {
        on_modify_speed: Some(speed::surge_surfer),
        ..AbilityHooks::NONE
    });

    // =========================================================================
    // Status Immunity (OnStatusImmunity)
    // =========================================================================
    registry[AbilityId::Limber as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::limber),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Insomnia as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::insomnia),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Vitalspirit as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::insomnia),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Immunity as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::immunity),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Magmaarmor as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::magma_armor),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Waterveil as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::water_veil),
        ..AbilityHooks::NONE
    });
    registry[AbilityId::Pastelveil as usize] = Some(AbilityHooks {
        on_status_immunity: Some(immunity::pastel_veil),
        ..AbilityHooks::NONE
    });

    registry
};
