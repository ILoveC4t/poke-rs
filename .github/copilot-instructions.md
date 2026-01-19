# Role
You are an expert Rust developer working on `poke-rs`, a Pokemon battle engine.

# Architecture & Codebase Awareness
- **Core Crate**: `crates/poke_engine` is the main engine.
- **Damage Pipeline**: Located in `crates/poke_engine/src/damage/`. Flow: context -> base power -> effective stats -> pre-random modifiers -> final rolls.
- **Generations**: Mechanics are in `crates/poke_engine/src/damage/generations/`. Gen9 is default; older generations override deltas via the `GenMechanics` trait.
- **State**: Uses `state.rs` (SoA layout) and `entities.rs`.
- **Abilities/Items**:
    - Modifiers often live in `damage/modifiers.rs`.
    - New ability effects should go to `crates/poke_engine/src/abilities/` and hooked into the pipeline.
    - **Note**: The ability registry is wired but incomplete. Prefer adding effects to `abilities/` and hooking them up.

# Development Workflow (CRITICAL)
- **Test Runner**: ALWAYS use the dedicated test runner found in `crates/test_runner`.
- **Pre-Change**: Run `cargo run -p test_runner -- run` to establish a baseline.
- **Post-Change**: Run `cargo run -p test_runner -- run` to verify changes.
- **Regression Check**: Run `cargo run -p test_runner -- analyze --base oldest` to check for regressions.

# Coding Style & Patterns
- **Math**: Use 4096-scale modifiers for precision. Use `apply_modifier()` for correct rounding.
- **Comments**: Write comments explaining *why* and *how* code works. DO NOT add "meta-comments" or change logs like `// FIXED BUG` or `// NEW FEAT`.
- **Generations**: Put generation-specific logic in `damage/generations/genN.rs`.
