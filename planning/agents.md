# Parallel Agent Task Assignments

This document defines independent work streams for parallel development. Each agent should work within their designated files/modules to minimize merge conflicts.

---

## General Guidelines (All Agents)

1. **Stay in your lane** – Only modify files listed in your task section
2. **Add, don't refactor** – Extend existing code rather than restructuring shared interfaces
3. **New files preferred** – When adding significant logic, create new files in your designated area
4. **Minimal imports** – Import from `lib.rs` re-exports, not internal module paths
5. **Feature flags** – If your work is incomplete, gate behind `#[cfg(feature = "wip")]` or `// TODO:`
6. **Test in isolation** – Add tests in the same file or a dedicated `_test.rs` file in your area
7. **No `mod.rs` edits without coordination** – If you need to add a `pub mod`, note it in your PR description

---

## Task A: Damage Modifiers - Items

**Owner:** Agent A  
**Branch:** `feat/damage-items`

### Scope
Implement item-based damage modifiers in the damage calculation pipeline.

### Files to Modify
- `crates/poke_engine/src/damage/modifiers.rs` – Add item modifier functions
- `crates/poke_engine/src/damage/context.rs` – Add item fields if needed

### Files to Create
- `crates/poke_engine/src/damage/items.rs` (optional, if modifiers.rs gets too large)

### Tasks
| Priority | Item | Effect |
|----------|------|--------|
| High | Choice Band | 1.5× Attack |
| High | Choice Specs | 1.5× Sp. Attack |
| High | Life Orb | 1.3× damage (5765/4096) |
| Medium | Expert Belt | 1.2× for super effective |
| Medium | Type-boosting (Charcoal, Mystic Water, etc.) | 1.2× for matching type |
| Low | Metronome (item) | Stacking boost per consecutive use |

### Implementation Pattern
```rust
// In modifiers.rs, add near other modifier functions:
pub fn apply_item_modifier(ctx: &DamageContext, base: u32) -> u32 {
    let modifier = match ctx.attacker_item {
        ItemId::CHOICE_BAND if ctx.move_category == MoveCategory::Physical => 6144, // 1.5×
        ItemId::LIFE_ORB => 5324, // 1.3× (technically applied differently)
        // ...
        _ => 4096,
    };
    chain_mods(base, modifier)
}
```

### Avoid
- Do not modify `formula.rs` (Task B territory)
- Do not modify generation files in `generations/` (Task C territory)

---

## Task B: Edge Case Moves

**Owner:** Agent B  
**Branch:** `feat/edge-moves`

### Scope
Implement special move formulas that don't follow standard damage calculation.

### Files to Modify
- `crates/poke_engine/src/damage/mod.rs` – Extend `get_fixed_damage()` function
- `crates/poke_engine/src/damage/formula.rs` – Add specialized base power calculations

### Files to Create
- `crates/poke_engine/src/damage/special_moves.rs` (for complex move logic)

### Tasks
| Priority | Move Category | Examples |
|----------|---------------|----------|
| High | Weight-based | Grass Knot, Low Kick, Heavy Slam, Heat Crash |
| High | HP-based | Eruption, Water Spout, Flail, Reversal |
| Medium | Weather Ball | Type + power change based on weather |
| Medium | Struggle | Typeless, 1/4 recoil, hits Ghost |
| Low | Flying Press | Dual-type (Fighting + Flying) |
| Low | Thousand Arrows | Ground-type that hits Flying |
| Low | Freeze-Dry | Ice-type super effective vs Water |

### Implementation Pattern
```rust
// In special_moves.rs (new file):
pub fn get_weight_based_power(attacker_weight: u16, defender_weight: u16, move_id: MoveId) -> u16 {
    match move_id.data().name {
        "Grass Knot" | "Low Kick" => {
            // Power based on defender weight
            match defender_weight {
                0..=99 => 20,
                100..=249 => 40,
                250..=499 => 60,
                500..=999 => 80,
                1000..=1999 => 100,
                _ => 120,
            }
        }
        "Heavy Slam" | "Heat Crash" => {
            // Power based on weight ratio
            let ratio = attacker_weight / defender_weight.max(1);
            match ratio {
                0..=1 => 40,
                2 => 60,
                3 => 80,
                4 => 100,
                _ => 120,
            }
        }
        _ => move_id.data().base_power,
    }
}
```

### Avoid
- Do not modify `modifiers.rs` (Task A territory)
- Do not modify `context.rs` unless adding move-specific fields

### Dependency Note
Weight-based moves require `weight` field on Pokemon. If not present, use placeholder:
```rust
// Temporary until entities.rs adds weight
let weight = state.species[idx].data().weight.unwrap_or(100);
```

---

## Task C: Generation Mechanics

**Owner:** Agent C  
**Branch:** `feat/gen-mechanics`

### Scope
Complete generation-specific trait implementations for older generations.

### Files to Modify
- `crates/poke_engine/src/damage/generations/gen8.rs`
- `crates/poke_engine/src/damage/generations/gen7.rs`
- `crates/poke_engine/src/damage/generations/gen6.rs`
- `crates/poke_engine/src/damage/generations/gen5.rs`
- `crates/poke_engine/src/damage/generations/gen4.rs`
- `crates/poke_engine/src/damage/generations/gen3.rs`

### Files to Create
- `crates/poke_engine/src/damage/generations/gen1.rs`
- `crates/poke_engine/src/damage/generations/gen2.rs`

### Tasks
| Generation | Key Differences |
|------------|-----------------|
| Gen 8 | Dynamax HP doubling, Max Move power scaling |
| Gen 7 | Z-Move power table, terrain 1.5× (not 1.3×) |
| Gen 6 | Mega stat boosts, Parental Bond |
| Gen 5 | 2.0× crit multiplier, no terrain |
| Gen 4 | Physical/Special split introduced |
| Gen 3 | No abilities before this gen (some games) |
| Gen 1-2 | Completely different formula, no split |

### Implementation Pattern
```rust
// In gen7.rs:
impl GenMechanics for Gen7 {
    const GEN: u8 = 7;
    
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<u16> {
        if !is_grounded { return None; }
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(6144), // 1.5× in Gen 7 (not 1.3×)
            (Terrain::Grassy, Type::Grass) => Some(6144),
            (Terrain::Psychic, Type::Psychic) => Some(6144),
            _ => None,
        }
    }
    // ... other overrides
}
```

### Avoid
- Do not modify `generations/mod.rs` beyond adding `pub mod gen1; pub mod gen2;`
- Do not change `GenMechanics` trait signature (coordinate if needed)

---

## Task D: Battle State & Entities

**Owner:** Agent D  
**Branch:** `feat/state-entities`

### Scope
Extend battle state and entity configuration with missing fields and screen/hazard logic.

### Files to Modify
- `crates/poke_engine/src/state.rs` – Add fields, implement screen/hazard methods
- `crates/poke_engine/src/entities.rs` – Add weight, gender, forme fields

### Files to Create
- `crates/poke_engine/src/state/screens.rs` (optional, for screen logic)
- `crates/poke_engine/src/state/hazards.rs` (optional, for hazard logic)

### Tasks
| Area | Task |
|------|------|
| Entities | Add `weight: u16` field to `PokemonConfig` |
| Entities | Add `gender: Gender` enum and field |
| Entities | Add forme-change method for Megas/Primals |
| State | Implement `apply_screens()` damage reduction |
| State | Implement `apply_entry_hazards()` on switch |
| State | Add screen turn counters |

### Implementation Pattern
```rust
// In state.rs, add to BattleState impl:
impl BattleState {
    /// Apply screen damage reduction (Reflect/Light Screen/Aurora Veil)
    pub fn screen_modifier(&self, side: usize, is_physical: bool) -> u16 {
        let conditions = self.side_conditions[side];
        
        if conditions.contains(SideConditions::AURORA_VEIL) {
            return 2732; // 0.5× in singles (2048 in doubles)
        }
        
        if is_physical && conditions.contains(SideConditions::REFLECT) {
            return 2732;
        }
        
        if !is_physical && conditions.contains(SideConditions::LIGHT_SCREEN) {
            return 2732;
        }
        
        4096 // No reduction
    }
}
```

### Avoid
- Do not modify damage calculation files (Task A/B territory)
- Keep `BattleState` struct additions at the END to minimize diff conflicts

### Coordination
- Task B (edge moves) needs `weight` field – add this first
- Document new fields with `/// Field description` comments

---

## Task E: Ability Hooks System

**Owner:** Agent E  
**Branch:** `feat/ability-hooks`

### Scope
Design and implement the ability hook registry system for runtime ability effects.

### Files to Create
- `crates/poke_engine/src/abilities/mod.rs` – Hook trait and registry
- `crates/poke_engine/src/abilities/registry.rs` – Static lookup table
- `crates/poke_engine/src/abilities/hooks/` – Individual ability implementations

### Files to Modify
- `crates/poke_engine/src/lib.rs` – Add `pub mod abilities;` (coordinate timing)

### Tasks
| Hook Type | Abilities |
|-----------|-----------|
| `on_switch_in` | Intimidate, Drizzle, Drought, Sand Stream |
| `on_before_move` | Prankster, Gale Wings, Triage |
| `on_modify_damage` | Multiscale, Shadow Shield, Solid Rock, Filter |
| `on_stat_change` | Contrary, Simple, Defiant, Competitive |
| `on_after_damage` | Color Change, Justified, Weak Armor |

### Implementation Pattern
```rust
// In abilities/mod.rs:
pub type OnSwitchInHook = fn(&mut BattleState, pokemon_idx: usize);
pub type OnModifyDamageHook = fn(&BattleState, attacker: usize, defender: usize, damage: u16) -> u16;

#[derive(Clone, Copy, Default)]
pub struct AbilityHooks {
    pub on_switch_in: Option<OnSwitchInHook>,
    pub on_modify_damage: Option<OnModifyDamageHook>,
    // ...
}

// In abilities/registry.rs:
pub static ABILITY_REGISTRY: [AbilityHooks; 300] = {
    let mut registry = [AbilityHooks::NONE; 300];
    registry[AbilityId::INTIMIDATE.0 as usize] = AbilityHooks {
        on_switch_in: Some(intimidate::on_switch_in),
        ..AbilityHooks::NONE
    };
    registry
};
```

### Avoid
- Do not modify existing damage pipeline until hooks are ready
- Do not modify `state.rs` (Task D territory)

### Integration Note
This is foundational work. Once complete, Task A (items) and the damage pipeline can call:
```rust
if let Some(hook) = ABILITY_REGISTRY[defender_ability].on_modify_damage {
    damage = hook(state, attacker, defender, damage);
}
```

---

## Task F: Tooling & Documentation

**Owner:** Agent F  
**Branch:** `feat/tooling-docs`

### Scope
Improve test fixtures, debugging tools, and documentation.

### Files to Modify
- `scripts/scrape_damage_tests.mts` – Capture more edge cases
- `scripts/generate_damage_calc_fixtures.mts` – Add strict mode per-test
- `crates/poke_engine/docs/mechanics/*.md` – Complete documentation

### Files to Create
- `scripts/diff_damage.mts` – Visual diff tool for damage mismatches
- `crates/poke_engine/docs/mechanics/generation-diffs.md` – Gen-by-gen cheat sheet
- `planning/test-coverage.md` – Track which tests pass per category

### Tasks
| Area | Task |
|------|------|
| Fixtures | Add weight data to Pokemon fixtures |
| Fixtures | Add screen/hazard scenarios |
| Fixtures | Support per-test skip annotations |
| Tooling | Create damage diff visualization |
| Tooling | Benchmark `BattleState` memory layout |
| Docs | Complete `abilities.md` hook documentation |
| Docs | Complete `items.md` modifier documentation |
| Docs | Create gen 1-9 difference cheat sheet |

### Implementation Pattern
```typescript
// In diff_damage.mts:
interface DamageDiff {
    testId: string;
    expected: number[];
    actual: number[];
    diff: number;       // Percentage difference
    category: string;   // "item", "ability", "weather", etc.
}

function visualizeDiff(diffs: DamageDiff[]): void {
    // Group by category, show pass/fail bars
}
```

### Avoid
- Do not modify Rust source files
- Documentation should describe INTENDED behavior, not just current state

---

## Conflict Resolution Protocol

If you need to modify a file outside your scope:

1. **Check the owner** – See which task owns that file
2. **Coordinate via PR description** – Note the cross-boundary change
3. **Prefer extension over modification** – Add new functions, don't change signatures
4. **Use `// TASK-X:` comments** – Mark code that another task should integrate

Example:
```rust
// TASK-A: Integrate this into apply_all_modifiers() when ready
pub fn apply_item_modifier_standalone(ctx: &DamageContext, damage: u32) -> u32 {
    // ...
}
```

---

## Merge Order Recommendation

```
1. Task D (State/Entities)     ─► Adds weight field needed by Task B
2. Task E (Ability Hooks)      ─► Foundational for ability modifiers
3. Task C (Gen Mechanics)      ─► Independent, can merge anytime
4. Task F (Tooling/Docs)       ─► Independent, can merge anytime
5. Task A (Item Modifiers)     ─► Uses structures from D/E
6. Task B (Edge Moves)         ─► Uses weight from D
```

---

## Quick Reference

| Task | Branch | Primary Files | Owner |
|------|--------|---------------|-------|
| A | `feat/damage-items` | `damage/modifiers.rs` | Agent A |
| B | `feat/edge-moves` | `damage/mod.rs`, `damage/formula.rs` | Agent B |
| C | `feat/gen-mechanics` | `damage/generations/*.rs` | Agent C |
| D | `feat/state-entities` | `state.rs`, `entities.rs` | Agent D |
| E | `feat/ability-hooks` | `abilities/` (new) | Agent E |
| F | `feat/tooling-docs` | `scripts/`, `docs/` | Agent F |
