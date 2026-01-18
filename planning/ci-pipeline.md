# CI/CD Pipeline Documentation

This document describes the automated pipelines that keep poke-rs synchronized with upstream Smogon projects.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          poke-rs CI/CD Pipelines                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────────────┐         ┌──────────────────────┐                 │
│   │   Sync Showdown      │         │  Generate Fixtures   │                 │
│   │   (Weekly: Mon 00:00)│         │  (Weekly: Mon 02:00) │                 │
│   └──────────┬───────────┘         └──────────┬───────────┘                 │
│              │                                │                             │
│              ▼                                ▼                             │
│   ┌──────────────────────┐         ┌──────────────────────┐                 │
│   │  pokemon-showdown    │         │     damage-calc      │                 │
│   │  (smogon/pokemon-    │         │  (smogon/damage-calc)│                 │
│   │   showdown)          │         │                      │                 │
│   └──────────┬───────────┘         └──────────┬───────────┘                 │
│              │                                │                             │
│              ▼                                ▼                             │
│   ┌──────────────────────┐         ┌──────────────────────┐                 │
│   │     data/*.json      │         │ tests/fixtures/*.json│                 │
│   │  - pokedex.json      │         │  - stats.json        │                 │
│   │  - moves.json        │         │  - stats-full.json   │                 │
│   │  - typechart.json    │         │  - damage.json       │                 │
│   │  - items.json        │         │  - pokemon.json      │                 │
│   │  - abilities.json    │         │                      │                 │
│   │  - natures.json      │         │                      │                 │
│   │  - learnsets.json    │         │                      │                 │
│   └──────────────────────┘         └──────────────────────┘                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Pipeline 1: Sync Showdown Data

**Workflow File:** [`.github/workflows/sync_showdown.yml`](../.github/workflows/sync_showdown.yml)

### Purpose
Extracts game data (species, moves, types, items, abilities) from the official [smogon/pokemon-showdown](https://github.com/smogon/pokemon-showdown) repository into local JSON files used by `build.rs` for code generation.

### Schedule
- **Cron:** `0 0 * * 1` (Every Monday at 00:00 UTC)
- **Manual:** Workflow dispatch available

### Script
[`scripts/sync_showdown_data.mts`](../scripts/sync_showdown_data.mts)

### Output Files

| File | Description |
|------|-------------|
| `data/pokedex.json` | Species stats, types, and forme data |
| `data/moves.json` | Move base power, accuracy, flags, effects |
| `data/typechart.json` | Type effectiveness matrix |
| `data/items.json` | Held item metadata |
| `data/abilities.json` | Ability metadata |
| `data/natures.json` | Stat modifiers per nature |
| `data/learnsets.json` | Legal move pools per species |

### Workflow Steps

1. **Clone** `smogon/pokemon-showdown` (shallow, depth=1)
2. **Run** `sync_showdown_data.mts` via `tsx`
3. **Output** JSON files to `data/` directory
4. **Create PR** on branch `data-sync-update`
5. **Cleanup** temporary clone

### Local Usage

```powershell
# From repo root
$env:SHOWDOWN_PATH = "C:\path\to\pokemon-showdown"
npx tsx scripts/sync_showdown_data.mts
```

---

## Pipeline 2: Generate Test Fixtures

**Workflow File:** [`.github/workflows/generate_fixtures.yml`](../.github/workflows/generate_fixtures.yml)

### Purpose
Generates test fixtures by running smogon/damage-calc's test suite with instrumentation, capturing all damage calculation scenarios as JSON for Rust tests.

### Schedule
- **Cron:** `0 2 * * 1` (Every Monday at 02:00 UTC, 2 hours after Showdown sync)
- **Manual:** Workflow dispatch available

### Scripts

| Script | Output |
|--------|--------|
| [`generate_damage_calc_fixtures.mts`](../scripts/generate_damage_calc_fixtures.mts) | `tests/fixtures/damage-calc/stats.json` |
| [`scrape_damage_tests.mts`](../scripts/scrape_damage_tests.mts) | `tests/fixtures/damage-calc/damage.json`, `stats-full.json`, `pokemon.json` |

### Output Files

| File | Description |
|------|-------------|
| `tests/fixtures/damage-calc/stats.json` | Basic stat calculation test cases |
| `tests/fixtures/damage-calc/stats-full.json` | Complete stat tests (displayStat, calcStat, DV/IV, Gen 2 modifiers) |
| `tests/fixtures/damage-calc/damage.json` | Damage calculation scenarios (attacker, defender, move, field, expected) |
| `tests/fixtures/damage-calc/pokemon.json` | Pokemon construction and forme validation tests |

### Workflow Steps

1. **Clone** `smogon/damage-calc` into `fixtures-temp/damage-calc` (shallow, depth=1)
2. **Setup** Node.js environment with `tsx` and `typescript`
3. **Run** `generate_damage_calc_fixtures.mts` (stat calculations)
4. **Run** `scrape_damage_tests.mts` (damage/Pokemon tests via AST parsing + runtime capture)
5. **Output** JSON files to `tests/fixtures/damage-calc/`
6. **Create PR** on branch `fixtures-update`
7. **Cleanup** temporary clone

### Local Usage

```powershell
# From repo root
$env:DAMAGE_CALC_PATH = "C:\path\to\damage-calc"
npx tsx scripts/generate_damage_calc_fixtures.mts
npx tsx scripts/scrape_damage_tests.mts
```

---

## How Fixtures Are Scraped

The `scrape_damage_tests.mts` script uses a sophisticated approach:

1. **TypeScript AST Parsing** – Reads `damage-calc/calc/src/test/*.ts` files
2. **Monkey-Patching** – Wraps `calculate()`, `Pokemon()`, `Stats.*` with capture hooks
3. **Jest Test Discovery** – Identifies `describe()` and `test()` blocks
4. **Runtime Execution** – Runs each test, capturing inputs and outputs
5. **Serialization** – Converts rich objects to minimal JSON representation

### Captured Data Per Test Case

```typescript
interface CapturedCase {
    id: string;                    // Unique identifier
    gen: number;                   // Generation (1-9)
    testName: string;              // From Jest describe/test
    attacker: {                    // Attacking Pokemon
        name, level?, item?, ability?, nature?,
        evs?, ivs?, boosts?, status?, curHP?, teraType?
    };
    defender: { /* same as attacker */ };
    move: {
        name, useZ?, isCrit?, hits?, overrides?
    };
    field?: {                      // Battle field conditions
        weather?, terrain?, isGravity?, 
        attackerSide?, defenderSide?
    };
    expected: {
        damage: number | number[] | [number, number];
        desc: string;              // Human-readable description
    };
}
```

---

## Dependency Graph

```
smogon/pokemon-showdown ────► data/*.json ────► build.rs ────► Generated Rust Code
                                                    │
                                                    ▼
smogon/damage-calc ─────────► tests/fixtures/ ────► Rust Tests ◄── Generated Types
```

---

## Troubleshooting

### Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| `Data for 'X' is undefined` | Showdown export name changed | Update import in `sync_showdown_data.mts` |
| `Cannot find module` | damage-calc internal path changed | Update import paths in scripts |
| Workflow fails at npm pkg set | npm < v9 | Workflow uses Node 20, should work |
| Fixtures differ unexpectedly | damage-calc algorithm changed | Review changes, update Rust impl |

### Debugging Locally

```powershell
# Verbose mode for fixture generation
$env:DEBUG = "1"
npx tsx scripts/scrape_damage_tests.mts 2>&1 | Tee-Object debug.log
```

### Validating Fixtures

```powershell
# Run Rust tests against fixtures
cargo test --package poke_engine --test damage_calc_test

# Filter by generation
$env:POKE_TEST_GEN = "9"
cargo test damage_calc
```

---

## Benchmarking

The project includes Criterion benchmarks to validate performance targets.

### Running Benchmarks

```powershell
# All benchmarks
cargo bench --package poke_engine

# Specific benchmark
cargo bench --package poke_engine --bench damage_calc
cargo bench --package poke_engine --bench state_clone

# Quick mode (faster, less precise)
cargo bench --package poke_engine -- --quick
```

### Benchmark Suites

| Suite | File | Measures |
|-------|------|----------|
| `damage_calc` | [benches/damage_calc.rs](../crates/poke_engine/benches/damage_calc.rs) | Damage calculation throughput, crit vs non-crit, 4-move evaluation |
| `state_clone` | [benches/state_clone.rs](../crates/poke_engine/benches/state_clone.rs) | State copy performance, minimax simulation, Monte Carlo rollouts |

### Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Damage calculations/sec | >1M | ~17M ✅ |
| State copy | <1 µs | ~480 ns ✅ |
| BattleState size | <32 KB (L1 cache) | TBD |

### HTML Reports

Criterion generates HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` after running benchmarks.

---

## Maintenance Notes

1. **Schedule Offset** – Fixtures workflow runs 2 hours after Showdown sync to avoid race conditions if both change the same week.

2. **Shallow Clones** – Using `--depth 1` reduces clone time from ~30s to ~5s per repository.

3. **PR-Based Updates** – Both workflows create PRs rather than committing directly, allowing review before merge.

4. **Temp Directory Cleanup** – `fixtures-temp/` and `showdown-temp/` are removed after each run to avoid stale data.

5. **Version Pinning** – Consider pinning upstream repos to specific commits/tags for reproducibility (currently using HEAD).
