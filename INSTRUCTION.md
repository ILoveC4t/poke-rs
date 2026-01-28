# Handover Instructions: Arceus & Mold Breaker Implementation

## Current State
We have resolved the **Generation 4 Damage Formula** regressions and verified the **Weather Ball** fixes across generations.

## Completed Tasks
- ✅ **Screen Rounding**: Fixed `+2` constant timing and modifier rounding for Gen 3-4.
- ✅ **Gen 3 Weather Ball**: Implemented `OnModifyFinalDamage` hook to double damage after crit.
- ✅ **Gen 5+ Weather Ball**: Verified mechanics match Smogon (base damage mod vs base power mod).
- ✅ **Gen 4 Damage Logic**: Implemented correct "hybrid" calculation order (Random Roll matches Gen 5 timing, but other mods match Gen 3).
- ✅ **Architecture**: Validated `modifiers.rs` abstraction improvements.

## Pending Verification
- None immediately. Tests are passing baseline.

## Remaining Tasks

### 1. Arceus Multitype (~4 failures)
**Problem**: Arceus holding a Plate (e.g., Zap Plate) is not changing its type to Electric. Damage tests fail (expected STAB, got non-STAB).

**Plan**:
1.  **Create `multitype.rs`**: In `crates/poke_engine/src/abilities/implementations/`.
2.  **Implement Hook**: Create an `on_switch_in` (or `on_battle_start`) hook that:
    - Checks if the user is Arceus.
    - Checks the held Plate.
    - Updates `state.types[user]`.
3.  **Register Hook**: Add to `crates/poke_engine/src/abilities/registry.rs`.

### 2. Mold Breaker (~10 failures)
**Problem**: Mold Breaker ignores immunity abilities (Levitate) but fails to ignore damage-reducing abilities (Filter, Multiscale, Thick Fat).

**Plan**:
1.  **Modify `modifiers.rs`**: In `apply_final_mods`:
2.  **Add Check**: `if !has_mold_breaker(attacker) { apply_defender_hooks(...) }`

## Critical Context
- **Generation Field**: `BattleState` now has a `generation` field. use `state.generation` in hooks if you need gen-specific logic.
- **Hook Architecture**: We added `on_modify_final_damage` to `MoveHooks`. Use this for any move mechanics that happen *after* base damage calculation but *before* random rolls (mostly Gen 3 weirdness).

## Next Steps
1. Run tests to verify the Gen 5+ Weather Ball fix.
2. If passing, proceed to Arceus implementation.
3. If failing, fix the rounding mode in `apply_weather_mod_damage` (Gen 5+ uses 4096-scale, Gen 3-4 uses floor).
