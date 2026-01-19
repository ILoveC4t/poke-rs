# PROJECT
- **Root:** `poke-rs`
- **Core crate:** `crates/poke_engine`
- **Data:** `data/*.json` (moves/species/items/etc)
- **Generated code:** `build.rs -> $OUT_DIR` (types, moves, items, species, etc)
- **Damage pipeline:** `crates/poke_engine/src/damage/`
- **Fixtures:** `tests/fixtures/damage-calc/*.json` (scraped from smogon/damage-calc)

# ARCHITECTURE (SHORT)
- **Gen mechanics:** `GenMechanics` trait (`damage/generations/*`). Gen9 is default; older gens override deltas.
- **Damage flow:** context -> base power -> effective stats -> pre-random modifiers -> final rolls.
- **Special cases:** `damage/special_moves.rs` (Weather Ball, Struggle, etc).
- **State:** `state.rs` (SoA layout), `entities.rs` (PokemonConfig spawn).
- **Priority:** Finish full damage-fixture coverage before implementing combat simulation/turn engine (`BattleQueue`), and iterate on ability implementations as needed.

# FIX PATTERNS
- **Ability modifier:** `damage/modifiers.rs` (`compute_base_power` or `compute_effective_stats`), check `ctx.attacker_ability`.
- **Item modifier:** `damage/modifiers.rs`, use `ctx.state.items[attacker/defender]`.
- **Type immunity overrides:** `damage/context.rs` (effectiveness calculation).
- **Special move behavior:** `damage/special_moves.rs`.
- **Note:** The ability registry exists and is wired into the damage pipeline (Type Immunities, Weather, etc. implemented); prefer adding ability effects to `crates/poke_engine/src/abilities/` and hooking them into the damage pipeline as needed.

# TESTS
**Recommended:** Use the AI-friendly test runner.

> [!IMPORTANT]
> **ALWAYS** follow this workflow when implementing changes:
> 1.  **Before starting:** Run `cargo run -p test_runner -- run` to establish a baseline.
> 2.  **After implementation:** Run `cargo run -p test_runner -- run` to verify your changes.
> 3.  **Regression Check:** Run `cargo run -p test_runner -- analyze --base oldest` to ensure no regressions occurred compared to the session start.

**Basic Usage:**
- Run all tests: `cargo run -p test_runner -- run`
- Filter tests: `cargo run -p test_runner -- run --filter damage`
- Specific package: `cargo run -p test_runner -- run -p poke_engine`

**Analysis:**
- Compare vs previous run: `cargo run -p test_runner -- analyze`
- Compare vs oldest run: `cargo run -p test_runner -- analyze --base oldest`

**Results:**
- Summary: `.test_runs/latest.json`
- Raw Output: `.test_runs/last_output.txt`

**Benchmarks:** `cargo bench -p poke_engine`

# COVERAGE CHECK
1. Run damage fixture test before change, record Passed/Failed counts.
2. Apply fix.
3. Re-run same test, compare counts. Expect Passed up, Failed down.
4. Ensure no new regressions.

See `planning/test-coverage.md` for gaps and TODO list.

# STYLE
- Use 4096-scale modifiers; `apply_modifier()` for correct rounding.
- Put gen-specific behavior in `damage/generations/genN.rs`.
- Add TODO comments for missing mechanics.
- Avoid meta-comments or "change logs" in the source code (e.g., `// NEW IMPLEMENTATION` or `// FIXED BUG X`). Comments should explain the code's current logic; edit summaries belong in the commit message.