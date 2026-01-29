use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poke_engine::damage::{calculate_damage, Gen9};
use poke_engine::entities::PokemonConfig;
use poke_engine::moves::MoveId;
use poke_engine::state::BattleState;
const BATCH_INNER: usize = 1_000;

fn create_benchmark_state() -> (BattleState, MoveId) {
    let mut state = BattleState::new();

    // Spawn attacker (Pikachu) at index 0
    if let Some(config) = PokemonConfig::from_str("pikachu") {
        config.level(100).spawn(&mut state, 0, 0);
    }

    // Spawn defender (Snorlax) at index 6 (side 1, slot 0)
    if let Some(config) = PokemonConfig::from_str("snorlax") {
        config.level(100).spawn(&mut state, 1, 0);
    }

    let move_id = MoveId::from_str("thunderbolt").expect("Move exists");
    (state, move_id)
}

fn benchmark_state_size() {
    // Report actual size (not part of the noisy timing suite)
    println!(
        "BattleState size: {} bytes",
        std::mem::size_of::<BattleState>()
    );
    println!(
        "Fits in L1 cache line (64B): {}",
        std::mem::size_of::<BattleState>() <= 64
    );
}

fn benchmark_state_clone(c: &mut Criterion) {
    let (state, _) = create_benchmark_state();

    c.bench_function("BattleState clone", |b| {
        b.iter(|| {
            for _ in 0..(BATCH_INNER) {
                let _ = black_box(&state).clone();
            }
        })
    });
}

fn benchmark_damage_calc(c: &mut Criterion) {
    let (state, move_id) = create_benchmark_state();

    c.bench_function("calculate_damage", |b| {
        b.iter(|| {
            for _ in 0..(BATCH_INNER / 2) {
                calculate_damage(
                    Gen9,
                    black_box(&state),
                    0, // attacker
                    6, // defender (player 1, slot 0)
                    move_id,
                    false,
                );
            }
        })
    });
}

// `benchmark_state_size` prints static size info and is omitted from the noisy timing group.
criterion_group!(benches, benchmark_state_clone, benchmark_damage_calc);
criterion_main!(benches);
