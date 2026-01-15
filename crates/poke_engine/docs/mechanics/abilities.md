# Abilities

Abilities are passive effects that Pokémon possess. The engine handles them through a robust hook-based system, managing their lifecycle and various suppression mechanics.

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

## 4. Rating System

Abilities are assigned a numerical rating (found in `data/abilities.ts`) from -1 to 5:
-   **-1**: Detrimental (e.g., *Defeatist*, *Slow Start*).
-   **0**: No significant battle effect (e.g., *Run Away*).
-   **1-2**: Niche or minor utility.
-   **3-4**: Strong, standard competitive abilities (e.g., *Intimidate*, *Regenerator*).
-   **5**: Metagame-defining or "Essential" (e.g., *Shadow Tag*, *Imposter*).

## 5. Implementation Reference

-   **Lifecycle Hooks**: `sim/pokemon.ts` -> `setAbility()`.
-   **Suppression Logic**: `sim/pokemon.ts` -> `ignoringAbility()`.
-   **Bypass Logic**: `sim/battle.ts` -> `suppressingAbility()`.
-   **Data Structure**: `sim/dex-abilities.ts` -> `Ability` class.