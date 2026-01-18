//! Benchmarks for BattleState cloning performance.
//!
//! The `Copy` trait on `BattleState` is critical for AI search trees.
//! This benchmark validates that state cloning is effectively free.
//!
//! Run with:
//!   cargo bench --package poke_engine --bench state_clone

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use poke_engine::{BattleState, PokemonConfig};

/// Set up a fully populated battle state (12 Pokémon, all fields populated)
fn setup_full_battle() -> BattleState {
    let mut state = BattleState::new();
    
    // Player 1 team (typical VGC team)
    let team1 = [
        "flutter mane", "iron hands", "amoonguss", 
        "landorus", "urshifu", "rillaboom"
    ];
    for (slot, species) in team1.iter().enumerate() {
        if let Some(config) = PokemonConfig::from_str(species) {
            config
                .level(50)
                .evs([252, 0, 0, 252, 4, 0])
                .spawn(&mut state, 0, slot);
        }
    }
    
    // Player 2 team
    let team2 = [
        "incineroar", "tornadus", "arcanine", 
        "gastrodon", "kingambit", "gholdengo"
    ];
    for (slot, species) in team2.iter().enumerate() {
        if let Some(config) = PokemonConfig::from_str(species) {
            config
                .level(50)
                .evs([252, 252, 0, 0, 4, 0])
                .spawn(&mut state, 1, slot);
        }
    }
    
    // Add some battle conditions
    state.turn = 5;
    state.weather = 1; // Some weather
    
    state
}

fn bench_state_copy(c: &mut Criterion) {
    let state = setup_full_battle();
    
    // Verify Copy is implemented (compile-time check)
    fn assert_copy<T: Copy>() {}
    assert_copy::<BattleState>();
    
    c.bench_function("state_copy", |b| {
        b.iter(|| {
            let copied: BattleState = black_box(state);
            black_box(copied)
        })
    });
}

fn bench_state_clone(c: &mut Criterion) {
    let state = setup_full_battle();
    
    c.bench_function("state_clone", |b| {
        b.iter(|| {
            let cloned = black_box(state).clone();
            black_box(cloned)
        })
    });
}

fn bench_state_copy_throughput(c: &mut Criterion) {
    let state = setup_full_battle();
    
    let mut group = c.benchmark_group("state_copy_throughput");
    
    for count in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_function(format!("{}_copies", count), |b| {
            b.iter(|| {
                for _ in 0..*count {
                    let copied: BattleState = black_box(state);
                    black_box(copied);
                }
            })
        });
    }
    
    group.finish();
}

fn bench_state_size(c: &mut Criterion) {
    // Report the size of BattleState for optimization tracking
    let size = std::mem::size_of::<BattleState>();
    println!("BattleState size: {} bytes", size);
    
    // Benchmark based on expected size (should fit in L1 cache ~32KB)
    assert!(size < 32 * 1024, "BattleState should fit in L1 cache");
    
    c.bench_function("state_memcpy_equivalent", |b| {
        let state = setup_full_battle();
        
        b.iter(|| {
            // Direct memory copy (what Copy does under the hood)
            let dest: BattleState = unsafe {
                std::ptr::read(&state as *const BattleState)
            };
            black_box(dest)
        })
    });
}

fn bench_minimax_simulation(c: &mut Criterion) {
    let base_state = setup_full_battle();
    
    // Simulate minimax tree exploration: copy state, mutate, evaluate
    c.bench_function("minimax_depth3_branching4", |b| {
        b.iter(|| {
            let branching = 4;
            let mut states_evaluated = 0u32;
            
            // Depth 1
            for _ in 0..branching {
                let mut s1 = base_state;
                s1.hp[0] = s1.hp[0].saturating_sub(50);
                
                // Depth 2
                for _ in 0..branching {
                    let mut s2 = s1;
                    s2.hp[6] = s2.hp[6].saturating_sub(50);
                    
                    // Depth 3
                    for _ in 0..branching {
                        let mut s3 = s2;
                        s3.turn += 1;
                        states_evaluated += 1;
                        black_box(&s3);
                    }
                }
            }
            
            states_evaluated
        })
    });
}

fn bench_monte_carlo_simulation(c: &mut Criterion) {
    let base_state = setup_full_battle();
    
    // Simulate Monte Carlo rollouts
    c.bench_function("monte_carlo_1000_rollouts", |b| {
        b.iter(|| {
            let rollouts = 1000;
            let rollout_depth = 10;
            
            for _ in 0..rollouts {
                let mut state = base_state;
                for turn in 0..rollout_depth {
                    // Simulate turn progression
                    state.turn += 1;
                    // Simulate damage (simplified)
                    let damage = (turn * 10) as u16;
                    state.hp[0] = state.hp[0].saturating_sub(damage);
                    state.hp[6] = state.hp[6].saturating_sub(damage);
                }
                black_box(&state);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_state_copy,
    bench_state_clone,
    bench_state_copy_throughput,
    bench_state_size,
    bench_minimax_simulation,
    bench_monte_carlo_simulation,
);

criterion_main!(benches);
