use crate::items::ItemId;
use crate::items::hooks::ItemHooks;

pub static ITEM_REGISTRY: [Option<ItemHooks>; ItemId::COUNT] = {
    let mut registry: [Option<ItemHooks>; ItemId::COUNT] = [None; ItemId::COUNT];

    // =========================================================================
    // Item Hook Registrations
    // =========================================================================
    // Example:
    // registry[ItemId::Lifeorb as usize] = Some(ItemHooks { ... });

    registry
};
