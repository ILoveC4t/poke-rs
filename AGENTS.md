# PROJECT
- **Root:** `poke-rs`
- **Core crate:** `crates/poke_engine`
- **Data:** `data/*.json` (moves/species/items/etc)
- **Generated code:** `build.rs -> $OUT_DIR` (types, moves, items, species, etc)
- **Damage pipeline:** `crates/poke_engine/src/damage/`
- **Fixtures:** `tests/fixtures/damage-calc/*.json` (scraped from smogon/damage-calc)

# ARCHITECTURE (SHORT)
- **Gen mechanics:** `GenMechanics` trait (`damage/generations/*`). Gen9 is default; older gens override deltas.
- **Damage pipeline:** `DamagePipeline` trait (`damage/pipeline.rs`). Defines order of final damage operations (random roll, STAB, effectiveness, etc.).
- **Damage flow:** context -> base power -> effective stats -> pre-random modifiers -> final rolls (via pipeline).
- **Special cases:** `damage/special_moves.rs` (Weather Ball, Struggle, etc).
- **State:** `state.rs` (SoA layout), `entities.rs` (PokemonConfig spawn).
- **Priority:** Finish full damage-fixture coverage before implementing combat simulation/turn engine (`BattleQueue`), and iterate on ability implementations as needed.

# FIX PATTERNS
- **Skip List**: If a fixture is logically incorrect (e.g., Smogon fixture doesn't apply a mechanics change that the engine correctly does), add its ID to `SKIPPED_FIXTURES` in `tests/damage_fixtures.rs` and add a correct custom test in `tests/` (e.g., `tests/multitype_correctness.rs`).
- **Ability modifier:** Implement hooks in `crates/poke_engine/src/abilities/implementations/` and register them in `modifiers.rs` or `hooks.rs`. Avoid adding inline checks in `damage/modifiers.rs`.
- **Item modifier:** Implement hooks in `crates/poke_engine/src/items/implementations/` and register in `items/registry.rs`.
- **Move logic:** Implement hooks in `crates/poke_engine/src/moves/implementations.rs` and register in `moves/registry.rs`.
- **Type immunity overrides:** Use `OnTypeImmunity` hook in `abilities/implementations/immunity.rs`.
- **Status/State:** Use `BattleState::set_status` which respects immunity hooks.
- **Note:** The ability/move/item registries are fully wired. Always prefer adding a new hook type over adding hardcoded logic to `modifiers.rs` or `state.rs`.

## TESTS — Exact commands to run

Follow these exact steps so results are reproducible and Jules does not mark the repo dirty.

1) Establish a baseline (before making changes)

```bash
# from repo root
cargo run -p test_runner -- run
```

- This produces the run artifacts in `.test_runs/` (e.g. `.test_runs/latest.json` and `.test_runs/last_output.txt`).

2) Run tests for the `poke_engine` package only

```bash
cargo run -p test_runner -- run -p poke_engine
```

3) Run only damage-related fixtures (useful during damage fixes)

```bash
cargo run -p test_runner -- run --filter damage
```

4) After making changes, re-run the baseline command and compare results

```bash
cargo run -p test_runner -- run
cargo run -p test_runner -- analyze   # compare to previous run
```

5) Regression check against the oldest recorded run

```bash
cargo run -p test_runner -- analyze --base oldest
```

6) Running tests offline (recommended for Jules snapshots / flaky network)

- Generate a vendored copy of crates locally and commit it (do this on your machine):

```bash
# from repo root
cargo vendor vendor
git add vendor
# cargo vendor prints a snippet you can copy to .cargo/config.toml; alternatively create it as below
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
git add .cargo/config.toml
git commit -m "Add vendored crates for offline CI"
git push
```

- In Jules, set the environment variable to require vendored mode (optional):

```text
# In Jules repo Configuration -> Environment variables
POKE_RS_USE_VENDOR=1
```

- Then run tests offline inside Jules (or locally):

```bash
cargo run --offline -p test_runner -- run
```

7) Quick `cargo test` examples (when you prefer Rust's test runner)

```bash
# run package unit tests
cargo test -p poke_engine
# run a single test by name
cargo test -p poke_engine --test <test_binary_name> -- --exact "test_name"
```

8) Troubleshooting

- If you see timeouts downloading `https://index.crates.io/config.json`, either commit `vendor/` as above or run the test runner with `--offline` after verifying `.cargo/config.toml` exists.
- If `cargo vendor` is blocked in Jules due to network, run it locally and commit `vendor/` to the repo; this is the most reliable approach for reproducible CI.

9) Results and artifacts

- Latest run summary: `.test_runs/latest.json`
- Raw output: `.test_runs/last_output.txt`
- Use `cargo run -p test_runner -- analyze` to compare runs programmatically.

10) Benchmarks (optional)

```bash
cargo bench -p poke_engine
```

# COVERAGE CHECK
1. Run damage fixture test before change, record Passed/Failed counts.
2. Apply fix.
3. Re-run same test, compare counts. Expect Passed up, Failed down.
4. Ensure no new regressions.

See `planning/coverage.md` for gaps and TODO list.

# STYLE
- Use 4096-scale modifiers; `apply_modifier()` for correct rounding.
- Put gen-specific behavior in `damage/generations/genN.rs`.
- Add TODO comments for missing mechanics.
- Avoid meta-comments or "change logs" in the source code (e.g., `// NEW IMPLEMENTATION` or `// FIXED BUG X`). Comments should explain the code's current logic; edit summaries belong in the commit message.
