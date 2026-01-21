use crate::items::ItemId;
use crate::items::hooks::ItemHooks;
use crate::items::implementations::*;

pub static ITEM_REGISTRY: [Option<ItemHooks>; ItemId::COUNT] = {
    let mut registry: [Option<ItemHooks>; ItemId::COUNT] = [None; ItemId::COUNT];

    // =========================================================================
    // Stat Modifiers (OnModifyAttack / OnModifyDefense)
    // =========================================================================

    registry[ItemId::Assaultvest as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_assault_vest),
        ..ItemHooks::NONE
    });

    registry[ItemId::Eviolite as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_eviolite),
        ..ItemHooks::NONE
    });

    registry[ItemId::Lightball as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_light_ball),
        ..ItemHooks::NONE
    });

    registry[ItemId::Choiceband as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_choice_band),
        ..ItemHooks::NONE
    });

    registry[ItemId::Choicespecs as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_choice_specs),
        ..ItemHooks::NONE
    });

    registry[ItemId::Deepseatooth as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_deep_sea_tooth),
        ..ItemHooks::NONE
    });

    registry[ItemId::Deepseascale as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_deep_sea_scale),
        ..ItemHooks::NONE
    });

    registry[ItemId::Souldew as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_soul_dew),
        on_modify_defense: Some(on_modify_defense_soul_dew),
        ..ItemHooks::NONE
    });

    registry[ItemId::Metalpowder as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_metal_powder),
        ..ItemHooks::NONE
    });

    // =========================================================================
    // Attacker Final Modifiers (OnAttackerFinalMod)
    // =========================================================================

    registry[ItemId::Lifeorb as usize] = Some(ItemHooks {
        on_attacker_final_mod: Some(on_attacker_final_mod_life_orb),
        ..ItemHooks::NONE
    });

    registry[ItemId::Expertbelt as usize] = Some(ItemHooks {
        on_attacker_final_mod: Some(on_attacker_final_mod_expert_belt),
        ..ItemHooks::NONE
    });

    // =========================================================================
    // Type-Boosting Items (OnModifyBasePower)
    // =========================================================================

    registry[ItemId::Charcoal as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_charcoal),
        ..ItemHooks::NONE
    });

    registry[ItemId::Mysticwater as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_mystic_water),
        ..ItemHooks::NONE
    });

    registry[ItemId::Miracleseed as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_miracle_seed),
        ..ItemHooks::NONE
    });

    registry[ItemId::Magnet as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_magnet),
        ..ItemHooks::NONE
    });

    registry[ItemId::Nevermeltice as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_never_melt_ice),
        ..ItemHooks::NONE
    });

    registry[ItemId::Blackbelt as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_black_belt),
        ..ItemHooks::NONE
    });

    registry[ItemId::Poisonbarb as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_poison_barb),
        ..ItemHooks::NONE
    });

    registry[ItemId::Softsand as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_soft_sand),
        ..ItemHooks::NONE
    });

    registry[ItemId::Sharpbeak as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_sharp_beak),
        ..ItemHooks::NONE
    });

    registry[ItemId::Twistedspoon as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_twisted_spoon),
        ..ItemHooks::NONE
    });

    registry[ItemId::Silverpowder as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_silver_powder),
        ..ItemHooks::NONE
    });

    registry[ItemId::Hardstone as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_hard_stone),
        ..ItemHooks::NONE
    });

    registry[ItemId::Spelltag as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_spell_tag),
        ..ItemHooks::NONE
    });

    registry[ItemId::Dragonfang as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_dragon_fang),
        ..ItemHooks::NONE
    });

    registry[ItemId::Blackglasses as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_black_glasses),
        ..ItemHooks::NONE
    });

    registry[ItemId::Metalcoat as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_metal_coat),
        ..ItemHooks::NONE
    });

    registry[ItemId::Silkscarf as usize] = Some(ItemHooks {
        on_modify_base_power: Some(on_modify_bp_silk_scarf),
        ..ItemHooks::NONE
    });

    // =========================================================================
    // Speed Modifiers (OnModifySpeed)
    // =========================================================================

    registry[ItemId::Choicescarf as usize] = Some(ItemHooks {
        on_modify_speed: Some(on_modify_speed_choice_scarf),
        ..ItemHooks::NONE
    });

    registry[ItemId::Ironball as usize] = Some(ItemHooks {
        on_modify_speed: Some(on_modify_speed_iron_ball),
        on_check_grounded: Some(on_check_grounded_iron_ball),
        ..ItemHooks::NONE
    });

    // =========================================================================
    // Grounding Modifiers (OnCheckGrounded)
    // =========================================================================

    registry[ItemId::Airballoon as usize] = Some(ItemHooks {
        on_check_grounded: Some(on_check_grounded_air_balloon),
        ..ItemHooks::NONE
    });

    // =========================================================================
    // Hazard Immunity (OnHazardImmunity)
    // =========================================================================
    registry[ItemId::Heavydutyboots as usize] = Some(ItemHooks {
        on_hazard_immunity: Some(on_hazard_immunity_heavy_duty_boots),
        ..ItemHooks::NONE
    });

    registry
};

