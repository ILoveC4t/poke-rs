# Architecture Design Document: Rust Pokémon Simulation Engine

## 1. Executive Summary
- **Goal:** Build a high-performance, memory-efficient Pokémon engine tailored for Monte Carlo / Minimax AI analysis.
- **Constraint:** The engine must be highly performant (millions of ticks/sec).
- **Core idea:** A data-oriented, stack-allocated library (no standard ECS, no heap).
- **Priority:** Achieve full damage-calculation fixture coverage before building the combat simulation / turn engine (see *Notes & Next Steps*).

---

## 2. The Core: "Stack-ECS" (Data Layer)
To maximize cache locality and minimize cloning costs for AI recursion, we use a Struct-of-Arrays (SoA) layout: instead of storing Pokémon objects, the engine stores arrays of primitive data so a single stat (e.g., Speed) for all entities can be loaded into the CPU cache together.

```rust
#[derive(Clone, Copy)] // Crucial: state is mem-copyable (stack allocated)
pub struct BattleState {
    // Entities (Index 0-11)
    // - 0-5: Player 1 Team
    // - 6-11: Player 2 Team
    pub active_indices: [u8; 2],  // Currently active Pokémon

    // Components (SoA Layout)
    pub hp:         [u16; 12],
    pub speeds:     [u16; 12],
    pub types:      [[Type; 2]; 12],
    pub abilities:  [AbilityID; 12],
    pub moves:      [[MoveID; 4]; 12], 

    // Bit-packed volatile statuses (Burn, Poison, etc.)
    // 1 bit per status, 32 bits per Pokémon
    pub volatiles:  [u32; 12], 

    // Event queue for turn logic (see Section 5) — planned; not yet implemented in the current codebase
    // pub queue:      BattleQueue, 
} 
```

---

## 3. Entity Generation: The "Blueprint" Pattern
We separate reusable game data (species defaults) from live battle memory. Directly writing into arrays (e.g., `battle.speeds[0] = 110`) is error-prone, so we use a transient `PokemonConfig` (a lightweight, stack-allocated blueprint) which is stamped into the engine's arrays at spawn time.

```rust
#[derive(Clone)]
pub struct PokemonConfig {
    pub species_id: SpeciesID,
    pub level: u8,
    pub base_stats: [u16; 6],
    pub types: [Type; 2],
    pub ability: AbilityID,
    pub moves: [MoveID; 4],
}

impl PokemonConfig {
    // "Builder"-style methods to customize the blueprint
    pub fn with_level(mut self, lvl: u8) -> Self { self.level = lvl; self }
    pub fn with_moves(mut self, moves: [MoveID; 4]) -> Self { self.moves = moves; self }
}
```

### Factory (Pokedex)
Factory functions return pre-configured blueprints (e.g., default species presets):

```rust
// pokedex.rs
pub fn gengar() -> PokemonConfig {
    PokemonConfig {
        species_id: SpeciesID::Gengar,
        level: 50,
        base_stats: [60, 65, 60, 130, 75, 110], // Gengar defaults
        types: [Type::Ghost, Type::Poison],
        ability: AbilityID::CursedBody,
        moves: [MoveID::None; 4], // Placeholder
    }
}
```

### Injection (spawn)
The `spawn` function takes a blueprint, computes final stats (level/IV/EV math), and writes raw integers into `BattleState` arrays.

```rust
let my_gengar = pokedex::gengar().with_level(100);
battle.spawn(0, my_gengar); // Inject into Player 1, slot 1 (index 0)

// Inside spawn():
// self.hp[index] = calculate_stat(config.base_stats[0], config.level);
// self.abilities[index] = config.ability;
```

---

## 4. The Logic Layer: Static Registry (V-Table)
To decouple ability/move logic from the main loop without the overhead of dynamic dispatch, we use a static lookup table (array) of function pointers. Abilities "opt in" to hooks by providing function pointers.

```rust
type TryHitHook = fn(&BattleState, attacker: usize, defender: usize, move_type: Type) -> bool;

struct AbilityHooks {
    on_switch_in: Option<fn(&mut BattleState, usize)>,
    on_try_hit:   Option<TryHitHook>,
    // ...
}
```

A macro generates a static `REGISTRY` array mapping `AbilityID` to `AbilityHooks`:

```rust
// abilities/levitate.rs
fn levitate_try_hit(...) -> bool { move_type == Type::Ground }

// main.rs (macro-generated)
const REGISTRY: [AbilityHooks; N] = [
    ...,
    AbilityHooks { on_try_hit: Some(levitate_try_hit), ..NO_OP }, // ID: Levitate
    ...,
];
```

**Execution flow:**
1. Trigger: engine reaches damage-calculation step.
2. Lookup: `let hooks = &REGISTRY[target.ability];` (array index).
3. Check: `if let Some(func) = hooks.on_try_hit { func(...) }` (pointer call).
4. Result: minimal branching, no virtual table lookups, cache-friendly.

---

## 5. The Flow Layer: Stack-Based Queue
To handle complex, recursive chains (e.g., Intimidate -> Eject Button -> Switch) without callback hell, logic pushes `Action`s to a queue rather than mutating state immediately. A single loop pops and executes actions until the queue is empty:

1. Pop action from `BattleState.queue`.
2. Execute action (e.g., apply damage).
3. If the action caused new events (e.g., faint), push new events to the queue.
4. Repeat until queue is empty.

---

## Notes & Next Steps
- The SoA/stack-first design favors fast cloning/copying for AI rollouts.
- Macro-generated registries keep hot-path logic cheap.
- Next: close damage fixture gaps (see planning/test-coverage.md) and achieve full damage-calculator coverage; defer construction of the `BattleQueue`/turn engine (combat simulation) until fixture coverage is green.

**Status:**
- **Damage pipeline:** Primary focus — gen1–9 mechanics and the modifier pipeline are implemented in `crates/poke_engine/src/damage/`.
- **Ability registry:** Implemented but incomplete; many abilities remain to be added to `crates/poke_engine/src/abilities/`.
- **Flow layer (`BattleQueue`/turn engine):** Planned and intentionally deferred until damage-calculator coverage is complete.
