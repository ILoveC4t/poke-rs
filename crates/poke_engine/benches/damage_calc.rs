//! Benchmarks for damage calculation pipeline.
//!
//! Target: >1M calculations/sec for AI rollout viability.
//!
//! Run with:
//!   cargo bench --package poke_engine --bench damage_calc

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use poke_engine::{
    calculate_damage, BattleState, Gen9, MoveId, PokemonConfig,
};

/// Set up a typical singles battle scenario
fn setup_singles_battle() -> (BattleState, MoveId) {
    let mut state = BattleState::new();
    
    // Attacker: Garchomp with typical competitive set
    if let Some(config) = PokemonConfig::from_str("garchomp") {
        config
            .level(50)
            .evs([0, 252, 0, 0, 4, 252])
            .spawn(&mut state, 0, 0);
    }
    
    // Defender: Tyranitar
    if let Some(config) = PokemonConfig::from_str("tyranitar") {
        config
            .level(50)
            .evs([252, 0, 128, 0, 128, 0])
            .spawn(&mut state, 1, 0);
    }
    
    let earthquake = MoveId::from_str("earthquake").expect("earthquake exists");
    
    (state, earthquake)
}

/// Set up a VGC doubles scenario (4 active Pokémon)
#[allow(dead_code)]
fn setup_doubles_battle() -> BattleState {
    let mut state = BattleState::new();
    
    // Player 1
    let pokemon_p1 = ["flutter mane", "iron hands", "amoonguss", "landorus"];
    for (slot, species) in pokemon_p1.iter().enumerate() {
        if let Some(config) = PokemonConfig::from_str(species) {
            config.level(50).spawn(&mut state, 0, slot);
        }
    }
    
    // Player 2
    let pokemon_p2 = ["rillaboom", "incineroar", "urshifu", "tornadus"];
    for (slot, species) in pokemon_p2.iter().enumerate() {
        if let Some(config) = PokemonConfig::from_str(species) {
            config.level(50).spawn(&mut state, 1, slot);
        }
    }
    
    state
}

fn bench_single_damage_calc(c: &mut Criterion) {
    let (state, move_id) = setup_singles_battle();
    
    c.bench_function("damage_calc_single", |b| {
        b.iter(|| {
            calculate_damage(
                black_box(Gen9),
                black_box(&state),
                black_box(0),
                black_box(6),
                black_box(move_id),
                black_box(false),
            )
        })
    });
}

fn bench_damage_calc_with_crit(c: &mut Criterion) {
    let (state, move_id) = setup_singles_battle();
    
    let mut group = c.benchmark_group("damage_calc_crit");
    
    group.bench_function("non_crit", |b| {
        b.iter(|| {
            calculate_damage(Gen9, &state, 0, 6, move_id, black_box(false))
        })
    });
    
    group.bench_function("crit", |b| {
        b.iter(|| {
            calculate_damage(Gen9, &state, 0, 6, move_id, black_box(true))
        })
    });
    
    group.finish();
}

fn bench_damage_calc_throughput(c: &mut Criterion) {
    let (state, move_id) = setup_singles_battle();
    
    let mut group = c.benchmark_group("damage_calc_throughput");
    
    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    for _ in 0..size {
                        let _ = calculate_damage(Gen9, &state, 0, 6, move_id, false);
                    }
                })
            },
        );
    }
    
    group.finish();
}

fn bench_multiple_moves(c: &mut Criterion) {
    let mut state = BattleState::new();
    
    // Pikachu with full moveset
    if let Some(config) = PokemonConfig::from_str("pikachu") {
        config
            .level(50)
            .spawn(&mut state, 0, 0);
    }
    
    // Bulbasaur as defender
    if let Some(config) = PokemonConfig::from_str("bulbasaur") {
        config.level(50).spawn(&mut state, 1, 0);
    }
    
    let moves: Vec<MoveId> = ["thunderbolt", "voltswitch", "irontail", "quickattack"]
        .iter()
        .filter_map(|m| MoveId::from_str(m))
        .collect();
    
    c.bench_function("damage_calc_4moves", |b| {
        b.iter(|| {
            for &move_id in &moves {
                let _ = calculate_damage(Gen9, &state, 0, 6, move_id, false);
            }
        })
    });
}

fn bench_ai_rollout_simulation(c: &mut Criterion) {
    // Simulate an AI evaluating many board states
    let (base_state, move_id) = setup_singles_battle();
    
    c.bench_function("ai_rollout_100_states", |b| {
        b.iter(|| {
            let mut state = base_state;
            for _ in 0..100 {
                // Calculate damage
                let result = calculate_damage(Gen9, &state, 0, 6, move_id, false);
                // Simulate applying damage (mutating state)
                state.hp[6] = state.hp[6].saturating_sub(result.rolls[8]);
                // Reset for next iteration
                state.hp[6] = state.max_hp[6];
            }
        })
    });
}

criterion_group!(
    benches,
    bench_single_damage_calc,
    bench_damage_calc_with_crit,
    bench_damage_calc_throughput,
    bench_multiple_moves,
    bench_ai_rollout_simulation,
);

criterion_main!(benches);
