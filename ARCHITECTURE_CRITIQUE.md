# Architecture Critique: poke-rs

This document provides a comprehensive analysis of the poke-rs repository, identifying architectural strengths, weaknesses, and recommendations for improvement.

---

## Executive Summary

**poke-rs** is a Pokémon battle engine written in Rust, focused on damage calculation accuracy across all nine generations. The project demonstrates solid foundational design choices but exhibits signs of organic growth that have introduced inconsistencies and technical debt.

**Current Status:**
- Cargo Tests: 118 passed, 8 failed, 1 ignored
- Fixture Tests: 294 passed, 138 failed (68% passing, out of 432 total fixture cases)

---

## Architectural Strengths

### 1. Struct-of-Arrays (SoA) Memory Layout ✅

The `BattleState` uses a cache-friendly SoA layout optimized for AI rollout simulations:

```rust
pub struct BattleState {
    pub species: [SpeciesId; MAX_ENTITIES],
    pub hp: [u16; MAX_ENTITIES],
    pub stats: [[u16; 6]; MAX_ENTITIES],
    pub boosts: [[i8; BOOST_STATS]; MAX_ENTITIES],
    // ...
}
```

**Why this is good:**
- Enables SIMD optimizations for batch processing
- Excellent cache locality for Monte Carlo simulations
- Stack-allocated (`Copy` trait) for cheap cloning in search trees

### 2. Generation Delta Architecture ✅

The `GenMechanics` trait implements generational differences as deltas from Gen 9:

```rust
pub trait GenMechanics: Copy + Clone + Send + Sync + 'static {
    const GEN: u8;
    fn crit_multiplier(&self) -> Modifier;
    fn terrain_modifier(&self, ...) -> Option<Modifier>;
    // ...
}
```

**Why this is good:**
- Minimizes code duplication
- Clear upgrade path for new generations
- Custom rulesets can implement the trait

### 3. Hook-Based Extension System ✅

The registry pattern for abilities/items/moves allows clean separation:

```rust
pub struct AbilityHooks {
    pub on_modify_base_power: Option<OnModifyBasePower>,
    pub on_type_immunity: Option<OnTypeImmunity>,
    pub on_defender_final_mod: Option<OnDefenderFinalMod>,
    // ...
}
```

**Why this is good:**
- Adding new abilities doesn't require modifying core damage code
- Easy to unit test individual hooks
- Clear contract for each hook type

### 4. Fixed-Point Arithmetic ✅

Using 4096-scale integer math matches the actual games:

```rust
pub const MOD_SCALE: u16 = 4096;

pub fn apply_modifier(value: u32, modifier: Modifier) -> u32 {
    let product = of32(value as u64 * modifier.0 as u64);
    pokeround(product, 4096)
}
```

**Why this is good:**
- Bit-exact accuracy with cartridge behavior
- No floating-point precision issues
- `pokeround` correctly handles Game Freak's rounding convention

### 5. Code Generation Pipeline ✅

Build script generates type-safe identifiers from JSON data:

```rust
// Generated from data/moves.json
pub struct MoveId(u16);
impl MoveId {
    pub const Tackle: MoveId = MoveId(1);
    pub fn data(&self) -> &'static Move { ... }
}
```

**Why this is good:**
- Compile-time type safety
- No runtime parsing overhead
- Easy to update from Smogon data sources

---

## Architectural Weaknesses

### 1. Orphaned Code Fragments 🔴 CRITICAL

The repository contains partially deleted code that causes build failures:

```rust
// In modifiers.rs - lines 577-605 (before fix)
/// Check if an item is a type-boosting item for the given type.
#[allow(dead_code)]
// get_type_boost_item_mod removed (migrated to item hooks)
    use crate::types::Type;
    
    let matches = match (item, move_type) {
        // ... orphaned match body
    };
```

**Root Cause:** Careless refactoring where function signatures were commented out but bodies remained.

**Recommendation:** Implement pre-commit hooks or CI checks that run `cargo build` before allowing merges.

### 2. Duplicate Registry Entries 🟡 MEDIUM

The same ability can be registered multiple times:

```rust
// In registry.rs
registry[AbilityId::Guts as usize] = Some(AbilityHooks {
    on_modify_attack: Some(on_modify_attack_guts),  // Line 116
    // ...
});

// Later in the same file
registry[AbilityId::Guts as usize] = Some(AbilityHooks {
    on_modify_attack: Some(status::on_modify_attack_guts),  // Line 165
    // ...
});
```

**Impact:** The second entry silently overwrites the first. If they're different, this causes bugs.

**Recommendation:** 
1. Use a compile-time macro to prevent duplicate registrations
2. Or use a builder pattern that panics on duplicates during initialization

### 3. Inconsistent Hook Locations 🟡 MEDIUM

Some ability effects are still hardcoded in `modifiers.rs` instead of using the hook system:

```rust
// In context.rs - inline type conversion for Aerilate, Pixilate, etc.
match attacker_ability {
    AbilityId::Aerilate if move_type == Type::Normal => move_type = Type::Flying,
    AbilityId::Pixilate if move_type == Type::Normal => move_type = Type::Fairy,
    // ...
}
```

**Issue:** This violates the "hooks are the single source of truth" principle stated in AGENTS.md.

**Recommendation:** Add an `on_modify_type` hook and migrate these abilities.

### 4. Giant Enum Delegation in `Generation` 🟡 MEDIUM

The `Generation` enum has ~200 lines of boilerplate delegating to inner types:

```rust
fn calculate_damage(&self, ctx: &DamageContext<Self>) -> DamageResult {
    match self {
        Generation::Gen1(g) => {
            let inner = DamageContext { gen: *g, state: ctx.state, /* ... 18 fields */ };
            g.calculate_damage(&inner)
        },
        Generation::Gen2(g) => { /* same pattern */ },
        // ...repeated 9 times
    }
}
```

**Issue:** 
- High maintenance burden
- Error-prone when adding fields to `DamageContext`
- Violates DRY principle

**Recommendation:** Use a macro to generate the delegation, or restructure `DamageContext` to be generic-agnostic.

### 5. Missing Test Coverage Annotations 🟡 MEDIUM

Test modules are inconsistently annotated:

```rust
// In mod.rs
mod ate_tests;                    // Missing #[cfg(test)]
mod conditional_moves_tests;      // Missing #[cfg(test)]
```

**Impact:** Compiler warnings about unused imports in non-test builds.

**Recommendation:** Add `#[cfg(test)]` consistently or move to `tests/` directory.

### 6. Leaky Abstraction in Type Effectiveness 🟡 MEDIUM

The effectiveness calculation mixes generation logic with ability logic:

```rust
// In context.rs
let effectiveness = super::effectiveness::calculate_effectiveness(/* ... */);

// Then separately
if effectiveness > 0 {
    effectiveness = Self::check_ability_immunity(/* ... */);
}
```

**Issue:** Type effectiveness is sometimes 0 (immune) but ability immunity also returns 0, conflating two different concepts.

**Recommendation:** Use an enum like `EffectivenessResult { Effective(u8), TypeImmune, AbilityImmune }`.

### 7. No Separation Between Calculation and Mutation 🟡 MEDIUM

The engine conflates two distinct operations:
1. **Calculation:** Pure damage number computation
2. **Application:** Modifying battle state (HP, status, etc.)

```rust
// DamageContext is passed by mutable reference for efficiency
fn calculate_damage_with_overrides<G: GenMechanics>(/* ... */) -> DamageResult
```

**Issue:** Harder to reason about side effects; complicates parallel rollouts.

**Recommendation:** Make damage calculation completely pure, returning a `DamageIntent` that is applied separately.

### 8. Hardcoded Player Indices 🟠 LOW

Player slot logic is scattered throughout:

```rust
fn has_screen(&self, is_physical: bool) -> bool {
    let side = if self.defender >= 6 { 1 } else { 0 };  // Magic number 6
    // ...
}
```

**Issue:** Assumes 2-player singles format; makes doubles/multi harder.

**Recommendation:** Use `Side` enum or `entity.side()` method.

---

## Missing Implementations

Based on fixture analysis (138 failures / 432 total = ~32% failing):

### High Priority
| Feature | Impact | Estimated Effort |
|---------|--------|------------------|
| Parental Bond multi-hit | Many fixtures | Medium |
| Z-Moves (Gen 7) | Entire category | High |
| Some defensive abilities | Individual fixtures | Low each |

### Medium Priority
| Feature | Notes |
|---------|-------|
| Metronome (item) | Needs `consecutive_move_count` state |
| Terrain Pulse | Type/power changes based on terrain |
| Weather Ball type | Partially implemented, needs fixes |

### Low Priority
| Feature | Notes |
|---------|-------|
| Magic Room | Item suppression field |
| Wonder Room | Def/SpD swap field |
| Pledge combinations | Grass/Fire/Water pledges |

---

## Code Quality Issues

### 1. Unused Constants and Functions

There are 29+ compiler warnings about unused code:

```
warning: constant `STAT_INDEX_SP_DEFENSE` is never used
warning: function `has_contact_ability_boost` is never used
```

**Recommendation:** Either use or remove dead code. Consider `#[expect(dead_code)]` with a TODO comment for planned features.

### 2. Inconsistent Documentation

Some modules have excellent rustdoc:

```rust
//! Damage calculation pipeline.
//!
//! This module implements a modular, pipeline-style damage calculator
//! focused on Generation 9 mechanics with extensibility for other generations.
```

Others have none.

**Recommendation:** Enforce documentation via `#![deny(missing_docs)]` for public items.

### 3. Test Pollution

Unit tests in `modifiers.rs` are ~700 lines, making the file unwieldy:

```
modifiers.rs: 1306 lines (46% tests)
```

**Recommendation:** Move tests to `modifiers_tests.rs` or `tests/` directory.

---

## Recommendations Summary

### Immediate Actions (Pre-Merge Required)

1. ✅ Fix orphaned code blocks (completed in this PR)
2. Run `cargo clippy` and address warnings
3. Add CI check for `cargo build` on PRs

### Short-Term (Next Sprint)

1. Migrate inline ability type changes to hooks
2. Add `#[cfg(test)]` annotations consistently
3. Remove duplicate registry entries with compile-time check
4. Reduce `Generation` enum boilerplate with macro

### Medium-Term (Next Quarter)

1. Implement priority missing features (Parental Bond, Z-Moves)
2. Add `EffectivenessResult` enum for clarity
3. Separate calculation from mutation
4. Add fuzzing tests for edge cases

### Long-Term (Roadmap)

1. Implement turn engine (`BattleQueue`)
2. Add doubles/multi-battle support
3. Web API for damage calculator
4. WASM compilation for browser use

---

## Conclusion

The poke-rs architecture is fundamentally sound with excellent performance-oriented design choices. However, the codebase shows signs of rapid iteration without sufficient cleanup. The hook-based extension system is well-designed but inconsistently applied.

**Priority Focus:** Complete the migration to hooks, fix test failures, and improve CI to prevent build breakage.

**Estimated Technical Debt:** ~3-4 weeks of focused effort to reach 95%+ fixture accuracy and clean codebase.

---

*Repository State: 118 cargo tests passing, 294/432 fixtures passing (68%)*
