# Abilities

Abilities are passive effects that Pokémon possess. The engine handles them through a robust hook-based system, managing their lifecycle and various suppression mechanics.

## Implementation

Abilities are implemented via the hook system in `src/abilities/`:

- `hooks.rs` - Hook type definitions (`OnSwitchIn`, `OnModifyPriority`, etc.)
- `registry.rs` - `ABILITY_REGISTRY` lookup table
- `implementations/` - Individual ability logic

## Hook Types

| Hook | When Called | Example Abilities |
|------|-------------|-------------------|
| `on_switch_in` | Pokemon enters battle | Intimidate, Drizzle |
| `on_modify_priority` | Turn ordering | Prankster, Gale Wings |
| `on_modify_damage` | Damage calculation | Multiscale, Shadow Shield |
| `on_start` | Ability activation | Download, Trace |

## Adding a New Ability

```rust
// In abilities/implementations/your_ability.rs
pub fn your_ability_on_switch_in(state: &mut BattleState, idx: usize) {
    // implementation
}

// In abilities/registry.rs
AbilityId::YOUR_ABILITY => Some(AbilityHooks {
    on_switch_in: Some(your_ability_on_switch_in),
    ..Default::default()
}),
```

## 1. Ability Lifecycle

The engine triggers specific hooks at key points in an ability's existence:

-   **`onStart`**: Triggered when the ability first becomes active on a Pokémon. This occurs during:
    -   Switching in.
    -   Mega Evolution or Primal Reversion.
    -   Ability changes via moves (e.g., *Skill Swap*, *Entrainment*).
-   **`onUpdate`**: Triggered whenever the Pokémon's state changes. This is used for "check-in" abilities like *Zen Mode* or *Power Construct* that activate based on HP thresholds.
-   **`onEnd`**: Triggered immediately before an ability is removed or replaced. Used for cleanup, such as removing the effect of *Neutralizing Gas*.

## 2. Suppression Mechanics

Abilities can be disabled or bypassed through several mechanisms:

### Persistent Suppression
Handled by the `ignoringAbility()` check. An ability is ignored if:
-   The Pokémon has the **Gastro Acid** volatile status.
-   A Pokémon with **Neutralizing Gas** is active on the field (and not suppressed itself).
-   **Exception**: Abilities with the `cantsuppress` flag (e.g., *Multitype*, *Stance Change*) cannot be suppressed.
-   **Protection**: The item **Ability Shield** prevents the holder's ability from being suppressed.

### Temporary Bypassing (Mold Breaker)
Handled by the `suppressingAbility(target)` check during move execution. 
-   If a move has the `ignoreAbility` flag (e.g., used by a Pokémon with *Mold Breaker*, *Teravolt*, or *Turboblaze*), it will ignore the target's ability hooks.
-   This typically affects "breakable" abilities like *Levitate*, *Sturdy*, or *Volt Absorb*.
-   Like persistent suppression, `cantsuppress` abilities and holders of **Ability Shield** are immune to this effect.

## 3. Ability Flags

-   **`breakable`**: Marks abilities that are bypassed by *Mold Breaker*.
-   **`cantsuppress`**: Marks essential abilities that cannot be removed or ignored.
-   **`failroleplay` / `failskillswap`**: Prevents certain move interactions.

## References

- [Showdown: sim/pokemon.ts](https://github.com/smogon/pokemon-showdown/blob/master/sim/pokemon.ts)
- [Bulbapedia: Abilities](https://bulbapedia.bulbagarden.net/wiki/Ability)
