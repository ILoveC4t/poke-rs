use poke_engine::{MoveId, SpeciesId, get_move_data, get_species_data, get_type_effectiveness, Type};

fn main() {
    println!("Pokemon Engine Data Test");

    // Test Move: Pound
    // We assume Pound exists. If not, compilation will fail (which is good test).
    let move_id = MoveId::Pound;
    let move_data = get_move_data(move_id);
    println!("Move: {} ({:?})", move_data.name, move_id);
    println!("  Type: {:?}", move_data.type_);
    println!("  Power: {}", move_data.power);
    println!("  Category: {:?}", move_data.category);
    println!("  Flags: {:?}", move_data.flags);

    // Test Species: Bulbasaur
    let species_id = SpeciesId::Bulbasaur;
    let species_data = get_species_data(species_id);
    println!("Species: {} ({:?})", species_data.name, species_id);
    println!("  Types: {:?}, {:?}", species_data.types[0], species_data.types[1]);
    println!("  Base Stats: {:?}", species_data.base_stats);
    println!("  Abilities: {:?}", species_data.abilities);

    // Test Type Chart
    let eff = get_type_effectiveness(Type::Water, Type::Fire);
    println!("Water vs Fire: {}", eff);

    let eff2 = get_type_effectiveness(Type::Electric, Type::Ground);
    println!("Electric vs Ground: {}", eff2);
}
