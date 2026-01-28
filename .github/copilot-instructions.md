# Copilot Instructions for poke-rs

## Project Overview

`poke-rs` is a high-performance Pokemon battle engine written in Rust. It calculates damage, handles abilities/items/moves, and supports all generations (1-9). The project prioritizes:

1. **Accuracy** - Match smogon/damage-calc reference implementation
2. **Performance** - >1M damage calculations/sec, <1µs state copy
3. **Completeness** - Full generation support with proper mechanics deltas

## Repository Structure

| Directory | Purpose |
|-----------|---------|
| `crates/poke_engine` | Core battle engine crate |
| `crates/poke_engine_codegen` | Code generation utilities |
| `crates/test_runner` | Custom test harness for fixtures |
| `data/*.json` | Game data (moves, species, items, etc.) from smogon/pokemon-showdown |
| `tests/fixtures/damage-calc/*.json` | Test fixtures scraped from smogon/damage-calc |
| `planning/` | Coverage tracking and CI documentation |

## Architecture & Codebase Awareness

- **Core Crate**: `crates/poke_engine` is the main engine.
- **Damage Pipeline**: Located in `crates/poke_engine/src/damage/`. Flow: context → base power → effective stats → pre-random modifiers → final rolls.
- **Generations**: Mechanics are in `crates/poke_engine/src/damage/generations/`. Gen9 is default; older generations override deltas via the `GenMechanics` trait.
- **State**: Uses `state.rs` (SoA layout) and `entities.rs`.
- **Special Moves**: Weather Ball, Struggle, weight-based moves live in `damage/special_moves/` (with `power.rs`, `fixed.rs`, etc.).
- **Abilities/Items**:
    - Modifiers often live in `damage/modifiers.rs`.
    - New ability effects should go to `crates/poke_engine/src/abilities/implementations/` and hooked into the pipeline.
    - New item effects should go to `crates/poke_engine/src/items/implementations.rs` and registered in `items/registry.rs`.
    - **Note**: The ability/move/item registries are fully wired. Always prefer adding a new hook type over adding hardcoded logic to `modifiers.rs` or `state.rs`.

## Test Architecture

The damage fixture tests use `libtest-mimic` for individual test filtering and categories.

**Key files:**
- `crates/poke_engine/tests/damage_fixtures.rs` - Main test harness
- `crates/poke_engine/tests/common/` - Shared test utilities:
  - `fixtures.rs` - Data structures for JSON fixtures
  - `helpers.rs` - Test helpers (spawn_pokemon, apply_field, run_damage_test)
  - `skip_list.rs` - Intentionally skipped fixtures (with reasons)
  - `categories.rs` - Test categories for filtering (abilities, terrain, screens, etc.)

## Development Workflow (CRITICAL)

**ALWAYS** use the dedicated test runner found in `crates/test_runner`:

```bash
# Establish baseline BEFORE making changes
cargo run -p test_runner -- run

# After making changes, verify and compare
cargo run -p test_runner -- run
cargo run -p test_runner -- analyze

# Check for regressions against oldest recorded run
cargo run -p test_runner -- analyze --base oldest

# Run only damage-related fixtures (useful during damage fixes)
cargo run -p test_runner -- run --filter damage

# Run for poke_engine package only
cargo run -p test_runner -- run -p poke_engine
```

**Filtering tests by category:**
```bash
cargo test --test damage_fixtures -- terrain     # Terrain-related tests
cargo test --test damage_fixtures -- abilities   # Ability tests
cargo test --test damage_fixtures -- screens     # Screen-breaking tests
cargo test --test damage_fixtures -- gen9        # Gen 9 only
```

Quick `cargo test` examples (when you prefer Rust's test runner):
```bash
cargo test -p poke_engine
cargo test -p poke_engine --test <test_binary_name> -- --exact "test_name"
```

## Fix Patterns

When implementing new mechanics, follow these patterns:

| Change Type | Location | Registration |
|-------------|----------|--------------|
| Ability modifier | `crates/poke_engine/src/abilities/implementations/` | Hook in `modifiers.rs` or `hooks.rs` |
| Item modifier | `crates/poke_engine/src/items/implementations.rs` | Register in `items/registry.rs` |
| Move logic | `crates/poke_engine/src/moves/implementations.rs` | Register in `moves/registry.rs` |
| Type immunity override | `abilities/implementations/immunity.rs` | Use `OnTypeImmunity` hook |
| Status/State changes | Use `BattleState::set_status` | Respects immunity hooks |

**Avoid** adding inline checks directly in `damage/modifiers.rs` or `state.rs`.

## Coding Style & Patterns

- **Math**: Use 4096-scale modifiers for precision. Use `apply_modifier()` for correct rounding.
- **Comments**: Write comments explaining *why* and *how* code works. DO NOT add "meta-comments" or change logs like `// FIXED BUG` or `// NEW FEAT`. See `CONTRIBUTING_COMMENTS.md` for detailed guidelines.
- **Generations**: Put generation-specific logic in `damage/generations/genN.rs`. Gen 9 is baseline; older gens override deltas.
- **TODOs**: Keep existing TODOs. New TODOs must be concise and action-oriented.

## Coverage Tracking

Current fixture status is tracked in `planning/coverage.md`:
- Track implemented vs missing abilities, items, moves
- Check generation support status
- Reference when prioritizing work

## CI/CD Pipelines

Automated pipelines sync data from upstream Smogon projects (see `planning/ci-pipeline.md`):
- **Sync Showdown**: Weekly sync of game data from smogon/pokemon-showdown
- **Generate Fixtures**: Weekly generation of test fixtures from smogon/damage-calc

## Related Documentation

- `AGENTS.md` - Detailed project documentation and test commands
- `CONTRIBUTING_COMMENTS.md` - Commenting guidelines
- `planning/coverage.md` - Feature coverage tracking
- `planning/ci-pipeline.md` - CI/CD pipeline documentation
