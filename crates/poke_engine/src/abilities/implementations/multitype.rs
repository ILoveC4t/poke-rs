//! Multitype ability implementation (Arceus).
//!
//! Multitype changes Arceus's type based on its held Plate item.
//! This is triggered on switch-in to ensure proper type-matching for STAB.

use crate::items::ItemId;
use crate::state::BattleState;
use crate::types::Type;

/// Maps a Plate item to its corresponding Type.
/// Returns None if the item is not a Plate.
pub fn plate_to_type(item: ItemId) -> Option<Type> {
    match item {
        ItemId::Flameplate => Some(Type::Fire),
        ItemId::Splashplate => Some(Type::Water),
        ItemId::Meadowplate => Some(Type::Grass),
        ItemId::Zapplate => Some(Type::Electric),
        ItemId::Icicleplate => Some(Type::Ice),
        ItemId::Fistplate => Some(Type::Fighting),
        ItemId::Toxicplate => Some(Type::Poison),
        ItemId::Earthplate => Some(Type::Ground),
        ItemId::Skyplate => Some(Type::Flying),
        ItemId::Mindplate => Some(Type::Psychic),
        ItemId::Insectplate => Some(Type::Bug),
        ItemId::Stoneplate => Some(Type::Rock),
        ItemId::Spookyplate => Some(Type::Ghost),
        ItemId::Dracoplate => Some(Type::Dragon),
        ItemId::Dreadplate => Some(Type::Dark),
        ItemId::Ironplate => Some(Type::Steel),
        ItemId::Pixieplate => Some(Type::Fairy),
        _ => None,
    }
}

/// Called on switch-in: if Arceus holds a Plate, change its type.
pub fn multitype_on_switch_in(state: &mut BattleState, idx: usize) {
    let item = state.items[idx];
    if let Some(plate_type) = plate_to_type(item) {
        // Arceus becomes pure [plate_type] when holding a Plate
        state.types[idx] = [plate_type, plate_type];
    }
}
