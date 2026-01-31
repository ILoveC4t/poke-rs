# Feature Coverage

This document tracks implemented vs missing Pokémon mechanics in `poke-rs`.

---

## Current Status

| Category | Pass | Fail | Notes |
|----------|------|------|-------|
| **Fixtures** | 345 | 79 | Smogon damage-calc scraped tests |
| **Cargo Tests** | 470 | 100 | Unit + integration tests |

### Recent Fixes
- **Raging Bull & Screens**: Fixed tests 430, 431 by clearing unintended `Intimidate` boosts from test setup. Fixtures requiring clean slate stats must ensure previous `OnSwitchIn` effects are cleared.
- **Test Runner**: Updated `damage_fixtures.rs` to zero out boosts if not explicitly defined in the fixture JSON.

---

## Items

### ✅ Implemented
| Item | Effect | Hook |
|------|--------|------|
| Choice Band / Specs | 1.5x Atk/SpA | `on_modify_attack` |
| Life Orb | 1.3x damage | `on_attacker_final_mod` |
| Expert Belt | 1.2x on SE | `on_attacker_final_mod` |
| Type-boosting (Charcoal, Mystic Water, etc.) | 1.2x | `on_modify_base_power` |
| Light Ball | 2x Atk/SpA for Pikachu | `on_modify_attack` |
| Thick Club | 2x Atk for Cubone/Marowak | `on_modify_attack` |
| Deep Sea Tooth | 2x SpA for Clamperl | `on_modify_attack` |
| Deep Sea Scale | 2x SpD for Clamperl | `on_modify_defense` |
| Metal Powder | 2x Def for Ditto | `on_modify_defense` |
| Soul Dew | 1.5x SpA/SpD for Lati@s (Gen 7+) | `on_modify_attack` |
| Assault Vest | 1.5x SpD | `on_modify_defense` |
| Eviolite | 1.5x Def/SpD for NFE | `on_modify_defense` |
| Metronome | 1.0-2.0x scaling (+0.2x/use) | `on_modify_base_power` |

### ❌ Missing
| Item | Effect | Blocker |
|------|--------|---------|
| Berry resist types | 0.5x on SE | Complex condition logic |

---

## Abilities

### ✅ Implemented
| Ability | Effect |
|---------|--------|
| Technician | 1.5x for BP ≤ 60 |
| Hustle | 1.5x Atk |
| Adaptability | 2x STAB |
| Tinted Lens | 2x on NVE |
| Flash Fire | 1.5x Fire when activated |
| Scrappy / Mind's Eye | Hit Ghost with Normal/Fighting |
| Levitate | Ground immunity |
| Filter / Solid Rock / Prism Armor | 0.75x on SE |
| Multiscale / Shadow Shield | 0.5x at full HP |
| Fluffy | 0.5x contact, 2x Fire |
| Punk Rock | 0.5x sound |
| Ice Scales | 0.5x Special |

### ❌ Missing
| Ability | Effect | Priority |
|---------|--------|----------|
| Neuroforce | 1.25x on SE | Low |
| Sniper | 1.5x on crit | Low |

---

## Moves

### ✅ Migrated to MOVE_REGISTRY
| Move | Effect |
|------|--------|
| Knock Off | 1.5x if target has item |
| Venoshock | 2x if poisoned |
| Hex | 2x if statused |
| Brine | 2x if ≤50% HP |

### ❌ Still Inline / Missing
| Move | Effect | Status |
|------|--------|--------|
| Facade | 2x if statused | In `special_moves/power.rs` |
| Payback | 2x if moving last | Needs turn order |
| Bolt Beak / Fishious Rend | 2x if first | Needs turn order |
| Assurance | 2x if already hit | Needs hit tracking |

---

## Special Move Mechanics

### ✅ Implemented
- Weight-based: Grass Knot, Low Kick, Heavy Slam, Heat Crash
- HP-based: Eruption, Water Spout, Flail, Reversal
- Fixed damage: Night Shade, Seismic Toss
- Status boost: Facade
- Struggle (typeless)
- Weather Ball (type/power change)
- Flying Press (Dual-type)

### ❌ Missing
| Mechanic | Examples |
|----------|----------|
| Terrain Pulse | Terrain Pulse |

---

## Field Effects

### ✅ Implemented
- Weather (Sun, Rain, Sand, Hail, Snow)
- Terrain (Electric, Grassy, Psychic, Misty)
- Screens (Reflect, Light Screen, Aurora Veil)
- Gravity (removes Flying immunity)

### ❌ Missing
| Effect | Notes |
|--------|-------|
| Magic Room | Suppresses items |
| Wonder Room | Swaps Def/SpD |

---

## Generation Support

| Gen | Status | Key Differences |
|-----|--------|-----------------|
| 9 | ✅ Primary | Tera STAB, 1.3x terrain |
| 8 | ✅ Complete | Dynamax HP doubling |
| 7 | ✅ Complete | Z-Moves, 1.5x terrain |
| 6 | ✅ Complete | Megas |
| 5 | ✅ Complete | 2.0x crit |
| 4 | ✅ Complete | Phys/Spec split |
| 3 | ✅ Complete | Pre-split mechanics |
| 2 | 🟡 Basic | Different formula |
| 1 | 🟡 Basic | Different formula |

---

## Test Sources

- `tests/fixtures/damage-calc/damage.json` — Scraped from smogon/damage-calc
- `tests/fixtures/damage-calc/stats.json` — Stat calculation tests
- Refresh via `.github/workflows/generate_fixtures.yml`
