# Test Coverage Tracking

## Overview

We track test coverage across different domains of the engine.

## Categories

| Category | Status | Count | Notes |
|----------|--------|-------|-------|
| **Damage Calculation** |
| Standard Moves | 🟢 Pass | 100+ | Via `scrape_damage_tests` |
| Edge Cases | 🟡 Partial | 50+ | Limited by upstream fixtures |
| Z-Moves | 🟡 Partial | 1 | Upstream has limited coverage |
| Max Moves | 🔴 Todo | 0 | No upstream fixtures |
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

## Test Sourcing Notes

- Damage and stats fixtures are sourced from the upstream smogon/damage-calc test suite.
- Fixtures are refreshed via the workflow in `.github/workflows/generate_fixtures.yml`.
- Coverage is constrained by what upstream tests actually include (e.g., no Max Move unit tests).
- Fixture cases are executed as-is; there is no Z-Move/Dynamax skip filter in the runner.

## Known Gaps

1. **Status Effects**: Burn damage reduction implemented, but status application logic needs more tests.
2. **Field Effects**: Terrain/Weather modifiers implemented, but duration/override logic needs tests.
3. **Complex Abilities**: Abilities like Mold Breaker, Neutralizing Gas need integration tests.

## Implementation TODOs (from fixture failures)

- [ ] Weight modifiers: Heavy Metal (2x), Light Metal (0.5x), Float Stone (0.5x), Autotomize
- [ ] Type immunity negation: Ring Target, Iron Ball grounding
- [ ] Defender abilities: Multiscale/Shadow Shield (0.5x at full HP), Filter/Solid Rock
- [ ] Knock Off: 1.5x BP when target has removable item (+ Klutz interaction)
- [ ] Parental Bond: Two-hit with second at 0.25x power
- [ ] Psychic Terrain: Check attacker grounding for boost, not defender
- [ ] Arceus Plate: Type-boosting items (1.2x for matching type)
