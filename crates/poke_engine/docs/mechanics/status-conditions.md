# Status Conditions

The Pokémon engine handles both persistent (Major) and temporary (Volatile) status conditions. These effects modify stats, inflict damage over time, or restrict move selection.

## 1. Major Status Conditions

Major status conditions persist even if the Pokémon is switched out. A Pokémon can only have one major status condition at a time.

| Condition | Internal ID | Effect |
| :--- | :--- | :--- |
| **Burn** | `brn` | Deals 1/16 max HP damage per turn. Reduces Physical Attack by 50% (handled in damage calculation). |
| **Paralysis** | `par` | Reduces Speed by 50%. 25% chance to fail move execution each turn. |
| **Sleep** | `slp` | Lasts 1-3 turns (Gen 3+). Prevents moving unless using moves like *Sleep Talk*. |
| **Freeze** | `frz` | Prevents moving. 20% chance to thaw each turn. Thaws immediately if hit by certain Fire-type moves. |
| **Poison** | `psn` | Deals 1/8 max HP damage per turn. |
| **Toxic** | `tox` | Badly Poisoned. Damage starts at 1/16 and increases by 1/16 each turn. Resets to normal Poison upon switch-out. |

## 2. Volatile Status Conditions

Volatile conditions are temporary and are typically cleared when the Pokémon switches out. A Pokémon can have multiple volatile conditions simultaneously.

-   **Confusion**: Lasts 2-5 turns. 33% chance to hit self with a 40 BP physical move instead of using the chosen move.
-   **Flinch**: Prevents the Pokémon from moving for the current turn only.
-   **Leech Seed**: Drains 1/8 max HP at the end of each turn, healing the opponent.
-   **Substitute**: Replaces the Pokémon with a decoy (25% max HP). The decoy takes damage and blocks most status moves.
-   **Trapped**: Prevents the Pokémon from switching out (e.g., *Mean Look*, *Spider Web*).

## 3. Application and Immunities

The engine follows a strict process for applying status conditions:

1.  **Check Existing**: A Pokémon with a major status cannot receive another.
2.  **Immunity Check**:
    -   **Type-based**: Electric-types are immune to Paralysis; Fire-types are immune to Burn; Poison/Steel-types are immune to Poison/Toxic (unless the attacker has *Corrosion*).
    -   **Ability-based**: Abilities like *Limber* (Paralysis) or *Insomnia* (Sleep) provide immunity.
3.  **Event Triggers**: The engine runs the `SetStatus` event, allowing effects like *Safeguard* to block the application.

## 4. Lifecycle and Curing

-   **Duration**: Many conditions (Sleep, Confusion) use a `duration` or `durationCallback` to determine when they expire.
-   **Curing**: 
    -   **Manual**: Items like *Full Heal* or moves like *Refresh* call the `cureStatus()` function.
    -   **Automatic**: Some conditions have a % chance to clear every turn (Freeze, Confusion).
    -   **Switching**: Resets volatile statuses and the "Toxic counter."

## 5. Implementation Reference

-   **Core Logic**: `sim/pokemon.ts` -> `setStatus()`, `addVolatile()`, and `runStatusImmunity()`.
-   **Condition Definitions**: `data/conditions.ts`.
-   **Move-Specific Volatiles**: `data/moves.ts` (e.g., *Substitute*).