# Test Coverage Tracking

## Overview

We track test coverage across different domains of the engine.

## Categories

| Category | Status | Count | Notes |
|----------|--------|-------|-------|
| **Damage Calculation** |
| Standard Moves | 🟢 Pass | 100+ | Via `scrape_damage_tests` |
| Edge Cases | 🟡 Partial | 50+ | Synthetic tests generated |
| Z-Moves | 🔴 Todo | 0 | |
| Max Moves | 🔴 Todo | 0 | |
| **Mechanics** |
| Stat Calculation | 🟢 Pass | 100% | Full fixture coverage |
| Turn Order | 🟢 Pass | | Unit tests in `state.rs` |
| Speed Modifiers | 🟢 Pass | | Unit tests in `state.rs` |
| **Tooling** |
| Benchmarks | 🟢 Ready | 3 | `state_layout` bench added |
| Diff Tool | 🟢 Ready | | `diff_damage.mts` |

## Fixture Sources

- `tests/fixtures/damage-calc/damage.json`: Scraped from smogon/damage-calc.
- `tests/fixtures/damage-calc/stats.json`: Scraped from smogon/damage-calc.
- `tests/fixtures/damage-calc/pokemon.json`: Scraped from smogon/damage-calc.
- `tests/fixtures/synthetic_damage.json`: Generated combinations.

## Known Gaps

1. **Status Effects**: Burn damage reduction implemented, but status application logic needs more tests.
2. **Field Effects**: Terrain/Weather modifiers implemented, but duration/override logic needs tests.
3. **Complex Abilities**: Abilities like Mold Breaker, Neutralizing Gas need integration tests.
