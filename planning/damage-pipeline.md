# Damage Pipeline Implementation Plan

## Overview

**Goal:** Implement a modular damage calculation pipeline focused on Generation 9 mechanics, designed for easy extension to other generations (1-8) and custom rulesets.

**Philosophy:** Gen 9 is the "canonical" implementation. Older generations are defined as *deltas* from Gen 9 (or from their successor), overriding only what changed. This minimizes code duplication and makes the codebase easier to audit against official mechanics.

**Current Status:** Core infrastructure complete. 15/128 Gen 9 tests passing.

---

## 1. Generation Abstraction Layer

### 1.1 The `GenMechanics` Trait

Located in `src/damage/generations/mod.rs`:

```rust
pub trait GenMechanics: Copy + Clone + Send + Sync + 'static {
    const GEN: u8;
    fn crit_multiplier(&self) -> u16;        // 6144 = 1.5x, 8192 = 2.0x
    fn stab_multiplier(&self, has_adaptability: bool, is_tera_stab: bool) -> u16;
    fn weather_modifier(&self, weather: Weather, move_type: Type) -> Option<u16>;
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<u16>;
    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8;
    fn has_abilities(&self) -> bool;
    fn has_held_items(&self) -> bool;
    fn uses_physical_special_split(&self) -> bool;
    fn has_terastallization(&self) -> bool;
    fn burn_modifier(&self) -> u16;
}
```

### 1.2 Implemented Generations

| Generation | File | Key Differences from Defaults |
|------------|------|-------------------------------|
| Gen 9 | `gen9.rs` | Canonical (all defaults) |
| Gen 8 | `gen8.rs` | No Tera, has Dynamax |
| Gen 7 | `gen7.rs` | Z-Moves, Megas, 1.5x terrain |
| Gen 6 | `gen6.rs` | Megas, 1.5x terrain (no Psychic boost) |
| Gen 5 | `gen5.rs` | 2.0x crit, no terrain |
| Gen 4 | `gen4.rs` | 2.0x crit, phys/spec split introduced |
| Gen 3 | `gen3.rs` | No split, no Adaptability |

### 1.3 Runtime Generation Selection

```rust
pub enum Generation {
    Gen3(Gen3), Gen4(Gen4), Gen5(Gen5),
    Gen6(Gen6), Gen7(Gen7), Gen8(Gen8), Gen9(Gen9),
}

impl Generation {
    pub fn from_num(gen: u8) -> Self { ... }
}
```

---

## 2. Module Structure

```
crates/poke_engine/src/damage/
├── mod.rs              # Public API: calculate_damage(), DamageResult
├── context.rs          # DamageContext struct
├── formula.rs          # Base damage formula, pokeround, overflow handling
├── modifiers.rs        # Modifier pipeline (weather, crit, stab, etc.)
└── generations/
    ├── mod.rs          # GenMechanics trait + Weather/Terrain enums
    ├── gen9.rs         # Gen 9 (canonical)
    ├── gen8.rs         # Gen 8 deltas
    ├── gen7.rs         # Gen 7 deltas
    ├── gen6.rs         # Gen 6 deltas
    ├── gen5.rs         # Gen 5 deltas
    ├── gen4.rs         # Gen 4 deltas
    └── gen3.rs         # Gen 3 deltas
```

---

## 3. Test Configuration

### 3.1 Generation Filtering

Tests default to Gen 9 only. Override with environment variable:

```bash
# Run only Gen 9 tests (default)
cargo test damage

# Run only Gen 4 tests  
POKE_TEST_GEN=4 cargo test damage

# Run all generations
POKE_TEST_GEN=0 cargo test damage
```

### 3.2 Strict Mode

Set `STRICT_MODE = true` in `damage_calc_test.rs` to fail on any test mismatch (currently disabled during development).

---

## 4. Phased Implementation

### Phase 1: Core Formula ✅ COMPLETE
- [x] Create `damage/` module structure
- [x] Implement `formula.rs`: `get_base_damage()`, `pokeround()`, `of16()`, `of32()`
- [x] Implement `Gen9` as default `GenMechanics`
- [x] Implement `DamageContext` with all required fields
- [x] Wire up test harness with generation filtering

### Phase 2: Basic Modifiers (Current)
- [x] `apply_weather_mod()` (Sun/Rain for Fire/Water)
- [x] `apply_crit_mod()` (1.5x for Gen 9)
- [x] `apply_stab_mod()` (1.5x, check type match)
- [x] `apply_type_effectiveness()` (use existing `type_effectiveness()`)
- [x] `apply_burn_mod()` (0.5x for physical)
- [ ] Fix rounding discrepancies (off-by-one errors in some tests)

### Phase 3: Items & Abilities (TODO)
- [ ] Choice Band/Specs (1.5x Atk/SpA)
- [ ] Life Orb (1.3x damage)
- [ ] Expert Belt (1.2x for super effective)
- [ ] Type-boosting items (1.2x)
- [ ] Technician (1.5x for BP ≤ 60) ✅ Implemented
- [ ] Hustle (1.5x Atk) ✅ Implemented
- [ ] Adaptability (2.0x STAB) ✅ Implemented
- [ ] Solid Rock/Filter (0.75x for SE)
- [ ] Multiscale/Shadow Shield (0.5x at full HP)

### Phase 4: Edge Cases (TODO)
- [ ] Night Shade/Seismic Toss (fixed damage = level)
- [ ] Weight-based moves (Grass Knot, Heavy Slam, Low Kick, Heat Crash)
- [ ] HP-based moves (Eruption, Water Spout, Flail, Reversal)
- [ ] Struggle (typeless, hits Ghost)
- [ ] Weather Ball (type + power change)
- [ ] Flying Press (dual-type move)
- [ ] Thousand Arrows (hits Flying)
- [ ] Ring Target/Iron Ball (removes immunities)
- [ ] Multi-hit moves (return per-hit damage)
- [ ] Screens (Reflect, Light Screen, Aurora Veil)

### Phase 5: Generation Expansion (TODO)
- [ ] Implement remaining Gen 8 quirks (Dynamax)
- [ ] Implement Gen 7 (Z-Moves)
- [ ] Implement Gen 6 (Megas)
- [ ] Implement Gen 1-2 (major formula changes)

---

## 5. Current Test Results (Gen 9)

```
Passed:  15
Failed:  113
Skipped: 304 (other generations)
```

### Common Failure Categories

| Category | Count | Root Cause |
|----------|-------|------------|
| Fixed damage moves | 2 | Night Shade/Seismic Toss not implemented |
| Weight-based moves | ~10 | Grass Knot, Heavy Slam, etc. not implemented |
| Weather Ball | 4 | Type/power change not implemented |
| Struggle | 1 | Typeless damage not implemented |
| Ring Target/Iron Ball | 3 | Immunity negation not implemented |
| Rounding errors | ~20 | Minor off-by-one in modifier chain |
| Ability effects | ~30 | Multiscale, Solid Rock, etc. not implemented |
| Item effects | ~20 | Life Orb, Expert Belt, etc. not implemented |

---

## 6. Open Questions

1. **Multi-hit accuracy:** Should `calculate_damage()` handle accuracy rolls, or just return damage assuming hit?
   - **Decision:** Damage only. Accuracy is a separate concern (BattleQueue).

2. **Randomness:** Should the function accept an RNG, or always return all 16 rolls?
   - **Decision:** Return all 16 rolls. Caller picks random index when needed.

3. **Z-Moves / Dynamax / Tera:** How deep to go on gimmick support in Phase 1?
   - **Decision:** Tera STAB in Gen 9. Z-Moves and Dynamax deferred to Phase 5.

---

## 7. Success Criteria

### MVP (Gen 9 Only)
- [ ] Pass 100% of Gen 9 fixture tests (128 cases)
- [ ] Damage calculation matches smogon/damage-calc for standard scenarios
- [ ] Clean pipeline architecture with no 2000-line functions

### Full Release
- [ ] Pass 100% of all 432 fixture tests (Gens 1-9)
- [ ] Performance: >1M damage calculations/second (for AI rollouts)
- [ ] Documented generation differences in code comments
