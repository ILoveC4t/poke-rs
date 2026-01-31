use poke_engine::moves::{MoveId, MoveFlags};
fn main() {
    let move_id = MoveId::from_str("ragingbull").unwrap();
    let data = move_id.data();
    println!("Move: {:?}", move_id);
    println!("Flags: {:?}", data.flags);
    println!("Has BREAKS_SCREENS: {}", data.flags.contains(MoveFlags::BREAKS_SCREENS));
}
