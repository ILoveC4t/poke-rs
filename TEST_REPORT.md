# Offline Test Report

## Summary
**Status**: 🔴 Failed to run tests
**Reason**: Incomplete offline environment (missing `serde` and `serde_json` in `vendor/` directory) and slow/timeout-prone `crates.io` index update preventing online fetch.

## Investigation
Attempted to run `cargo run -p test_runner` but it timed out updating the crates.io index.
Attempted to use `cargo run -p test_runner --offline` with the `vendor/` directory configured as source, but verified that critical dependencies (`serde`, `serde_json`) are missing from `vendor/`.

## Static Analysis & Recommendations

Since dynamic testing failed, a static analysis of the codebase was performed against `planning/test-coverage.md` to identify implementation gaps.

### 1. Environment
**Issue**: `vendor/` directory is incomplete, making offline testing impossible.
**Recommendation**:
- Run `cargo vendor` in an online environment to download all dependencies.
- Commit the updated `vendor/` directory and `.cargo/config.toml` (configured to use vendor source).

### 2. Weight Modifiers
**Issue**: Missing implementation for `Heavy Metal`, `Light Metal` abilities, `Float Stone` item, and `Autotomize` move in damage calculation.
**Location**: `crates/poke_engine/src/damage/special_moves/power.rs` (explicit TODO).
**Recommendation**:
- Update `modify_base_power` to adjust weight based on these factors before calculating BP for `Grass Knot`, `Low Kick`, `Heavy Slam`, and `Heat Crash`.

### 3. Conditional Power Moves
**Issue**: Missing damage doubling logic for `Venoshock` (vs Poison), `Hex` (vs Status), and `Brine` (<50% HP).
**Location**: `crates/poke_engine/src/damage/special_moves/power.rs` (explicit TODOs).
**Recommendation**:
- Add logic in `modify_base_power` to check target status/HP and double BP.

### 4. Arceus Plates
**Issue**: Type-boosting logic for Arceus Plates (e.g., `Zap Plate`, `Flame Plate`) is missing.
**Location**: `crates/poke_engine/src/damage/modifiers.rs` (`get_type_boost_item_mod`).
**Recommendation**:
- Add all Plate items to the match arm in `get_type_boost_item_mod` to return `4915` (1.2x).

### 5. Parental Bond
**Issue**: Missing implementation for `Parental Bond` ability (second hit at 0.25x power).
**Location**: `crates/poke_engine/src/damage/modifiers.rs` (explicit TODO).
**Recommendation**:
- This is complex as it requires multi-hit logic in the damage pipeline or `calculate_damage` to return aggregated damage. A short-term fix could be a simplified modifier if only total damage matters, but correct implementation requires simulating two hits.

### 6. Weather Ball
**Issue**: Potential double-counting or missing weather boost verification.
**Location**: `crates/poke_engine/src/damage/special_moves/mod.rs` (TODO note).
**Recommendation**:
- Verify if `apply_weather_mod_bp` handles the 1.5x boost correctly when Weather Ball changes type. If so, remove the TODO. If not, adjust logic to ensure 100 BP * 1.5x Weather Boost = 150 BP total.
