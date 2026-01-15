# Moves and Targeting

Moves are the primary way Pokémon interact in battle. The engine categorizes moves, manages their execution flow, and resolves which targets they affect.

## 1. Execution Flow

Move execution is divided into high-level management and low-level effect processing:

-   **`runMove` (Outer Wrapper)**: Handles the "intent" to move.
    -   Deducts PP.
    -   Checks for immobilization (Paralysis, Sleep, Flinch, Confusion).
    -   Displays the "Pokémon used Move!" message.
    -   Triggers "Dancer" or "Instruct" effects.
-   **`useMove` (Inner Logic)**: Handles the "impact" of the move.
    -   Resolves targets (handling redirection like *Follow Me*).
    -   Checks accuracy and immunity.
    -   Calculates damage and applies secondary effects.
    -   Triggered directly by moves like *Sleep Talk* or *Metronome*.

## 2. Move Categories

Each move belongs to one of three categories, which determines its basic behavior:

-   **Physical**: Uses the attacker's Attack and the defender's Defense. Affected by Burn (0.5x damage).
-   **Special**: Uses the attacker's Sp. Atk and the defender's Sp. Def.
-   **Status**: Does not deal direct damage. Ignores type immunities by default (e.g., *Thunder Wave* can hit Water-types, though it still fails against Ground-types due to specific move rules).

## 3. Targeting Types

Moves specify a `target` type, which the engine resolves into actual Pokémon during execution:

| Target | Description |
| :--- | :--- |
| `normal` | Hits one adjacent Pokémon of the user's choice. |
| `self` | Affects the user only. |
| `any` | Can hit any Pokémon on the field (even non-adjacent in Triples). |
| `adjacentAlly` | Hits one teammate. |
| `allAdjacentFoes` | A spread move hitting all enemies (0.75x damage). |
| `allAdjacent` | Hits everyone on the field except the user. |

## 4. Hit Sequence and Accuracy

The engine follows a strict sequence of "Hit Steps" to determine if a move connects:

1.  **Invulnerability Check**: Checks if the target is in the semi-invulnerable state of *Fly*, *Dig*, etc.
2.  **Type Immunity**: Checks if the target is immune to the move's type.
3.  **Protection Check**: Checks for *Protect*, *Detect*, or *King's Shield*.
4.  **Accuracy Check**: 
    -   Uses the formula: `Accuracy = MoveAccuracy * (UserAccuracy / TargetEvasion)`.
    -   Accuracy and Evasion use stages from -6 to +6.
    -   Moves with `accuracy: true` (e.g., *Swift*) bypass this check.

## 5. Move Flags

Flags are metadata used to trigger or block specific mechanics:

-   **`contact`**: Triggers effects like *Iron Barbs* or *Rocky Helmet*.
-   **`sound`**: Bypassed by the *Soundproof* ability.
-   **`protect`**: Determines if the move can be blocked by protection moves.
-   **`bypasssub`**: Allows the move to hit through a *Substitute* (e.g., sound moves, *Infiltrator*).
-   **`punch`**: Boosted by the *Iron Fist* ability.

## 6. Complex Execution Flows

Some moves follow more complex logic beyond the standard hit sequence.

### Multi-Hit Moves
Moves that hit multiple times (e.g., *Bullet Seed*, *Population Bomb*) have specific hit count distributions.
-   **2-5 Hit Moves** (Gen 5+):
    -   2 hits: 35%
    -   3 hits: 35%
    -   4 hits: 15%
    -   5 hits: 15%
-   **Loaded Dice Interaction**:
    -   For 2-5 hit moves, it ensures 4 or 5 hits (evenly distributed).
    -   For *Population Bomb* (10 hits), it ensures 4 to 10 hits.

### Switch-Out Moves
Moves like *U-turn* and *Volt Switch* allow the user to switch after dealing damage.
1.  **Damage Calculation**: Damage is dealt first.
2.  **Switch Flag**: If the move is successful, a `switchFlag` is set on the user.
3.  **End of Move**: After all other effects resolve, the engine checks for the `switchFlag` and triggers a switch request.
    -   *Note:* If the user faints from recoil or *Rough Skin*, the switch does not happen.

### Charge and Recharge
-   **Two-Turn Moves** (e.g., *Solar Beam*):
    -   Turn 1: The user enters a charging state (`twoturnmove` volatile). A message is displayed.
    -   Turn 2: The move executes.
    -   *Power Herb*: Consumed to skip the charging turn.
-   **Recharge Moves** (e.g., *Hyper Beam*):
    -   Turn 1: The move executes.
    -   Turn 2: The user is unable to move (`mustrecharge` volatile).

## 7. Implementation Reference

-   **Execution Logic**: `sim/battle-actions.ts` -> `runMove()`, `useMove()`, and `trySpreadMoveHit()`.
-   **Multi-Hit Logic**: `sim/battle-actions.ts` -> `hitStepMoveHitLoop` (see `targetHits` sampling).
-   **Target Resolution**: `sim/pokemon.ts` -> `getMoveTargets()`.
-   **Data Definitions**: `sim/dex-moves.ts` -> `MoveTarget` and `MoveFlags`.