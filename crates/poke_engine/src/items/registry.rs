use crate::items::ItemId;
use crate::items::hooks::ItemHooks;
use crate::items::implementations::*;

pub static ITEM_REGISTRY: [Option<ItemHooks>; ItemId::COUNT] = {
    let mut registry: [Option<ItemHooks>; ItemId::COUNT] = [None; ItemId::COUNT];

    // =========================================================================
    // Item Hook Registrations
    // =========================================================================

    registry[ItemId::Assaultvest as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_assault_vest),
        ..ItemHooks::NONE
    });

    registry[ItemId::Eviolite as usize] = Some(ItemHooks {
        on_modify_defense: Some(on_modify_defense_eviolite),
        ..ItemHooks::NONE
    });

    registry[ItemId::Thickclub as usize] = Some(ItemHooks {
        on_modify_attack: Some(on_modify_attack_thick_club),
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

    registry
};
