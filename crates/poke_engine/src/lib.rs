pub mod core_data;

mod db {
    #![allow(dead_code)] // Generated code might have unused variants
    include!(concat!(env!("OUT_DIR"), "/generated_db.rs"));
}

pub use db::{MoveId, SpeciesId};
pub use core_data::*;

// We can expose the raw arrays if we want, or just accessors.
// Exposing raw arrays allows O(1) access for the user too.
pub use db::MOVES;
pub use db::SPECIES;
pub use db::TYPE_CHART;

pub fn get_move_data(id: MoveId) -> &'static MoveData {
    &MOVES[id as usize]
}

pub fn get_species_data(id: SpeciesId) -> &'static SpeciesData {
    &SPECIES[id as usize]
}

pub fn get_type_effectiveness(attacker: Type, defender: Type) -> f32 {
    let atk_idx = attacker as usize;
    let def_idx = defender as usize;

    // Safety check for Unknown types or logic errors
    if atk_idx >= 19 || def_idx >= 19 {
        return 1.0;
    }

    TYPE_CHART[atk_idx][def_idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_data_access() {
        let m = get_move_data(MoveId::Pound);
        assert_eq!(m.name, "Pound");
        assert_eq!(m.type_, Type::Normal);
        assert!(m.flags.contains(MoveFlags::CONTACT));
    }

    #[test]
    fn test_species_data_access() {
        let s = get_species_data(SpeciesId::Bulbasaur);
        assert_eq!(s.name, "Bulbasaur");
        assert!(s.types.contains(&Type::Grass));
    }

    #[test]
    fn test_type_chart() {
        assert_eq!(get_type_effectiveness(Type::Water, Type::Fire), 2.0);
        assert_eq!(get_type_effectiveness(Type::Normal, Type::Ghost), 0.0);
    }
}
