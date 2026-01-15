# Turn Order and Priority

The Pokémon engine uses a multi-layered sorting system to determine the sequence of actions in a turn. This system ensures that high-priority categories (like switching) occur before moves, and that speed is calculated accurately even during mid-turn changes.

## 1. Action Queue Layers

Each action in the `BattleQueue` is assigned an **Order** value which acts as a categorical priority. Actions are sorted primarily by this value (lower values first).

| Category | Order | Description |
| :--- | :--- | :--- |
| Team Preview | 1 | Choosing leads at the start of battle. |
| Start | 2 | The very beginning of the turn. |
| Switch | 103 | Intentional switching or forced switches. |
| Mega Evolution | 104 | Transformation before the move. |
| Terastallization| 106 | Pre-move type change (Gen 9). |
| Moves | 200 | Standard move execution. |
| Residual | 300 | End-of-turn effects (Weather, Status). |

### End-of-Turn Resolution Order
In the Residual phase (Order 300), effects are resolved in a specific priority order. Lower values are resolved first.

1.  **Weather** (Rain, Sun, Sandstorm, Hail/Snow)
2.  **Future Sight** / **Doom Desire**
3.  **Wish**
4.  **Leftovers** / **Black Sludge** (Healing/Damage)
5.  **Aqua Ring**
6.  **Leech Seed**
7.  **Status Damage** (Poison, Toxic)
8.  **Status Damage** (Burn)
9.  **Curse** (from Ghost-types)
10. **Partial Trapping Damage** (Bind, Wrap, Fire Spin, etc.)
11. **Syrup Bomb**
12. **Taunt** (ends)
13. **Encore** (ends)
14. **Disable** (ends)
15. **Telekinesis** (ends)
16. **Embargo** (ends)
17. **Throat Chop** (ends)
18. **Yawn**
19. **Perish Song**

## 2. Sorting Hierarchy

When actions have the same **Order** (e.g., two Pokémon using moves), the following hierarchy is used to determine who goes first:

1.  **Priority Bracket** (Higher first): Moves with higher priority (e.g., *Quick Attack* at +1) always beat moves with lower priority.
2.  **Speed Stat** (Higher first): The current calculated Speed of the Pokémon.
3.  **Sub-order / Effect-order** (Lower first): Used for tie-breaking between simultaneous internal events.
4.  **Random Tie-break**: If all values above are identical, the engine shuffles the tied actions using a Fisher-Yates shuffle via the battle's PRNG.

## 3. Speed Calculation

A Pokémon's Speed is not static and is recalculated at the start of the turn or when specific events trigger.

### Trick Room
In **Trick Room**, the speed calculation is inverted. The engine typically uses a transformation: `10000 - Speed`. This ensures that slower Pokémon effectively have higher "action speed."

### Mid-Turn Speed Changes
In modern generations (Gen 8+), speed changes take effect immediately. For example, if a teammate uses *Tailwind*, the speed of the other active Pokémon is updated before their next action in the queue.

## 4. Special Priority Modifiers

-   **Prankster**: Adds +1 priority to Status moves.
-   **Gale Wings**: Adds +1 priority to Flying-type moves (at 100% HP in later gens).
-   **Deferred Priority**: In Gen 7, a Pokémon's speed for the turn is locked *before* it Mega Evolves. In Gen 8+, the new speed is used immediately.

## 5. Implementation Reference

-   **Queue Management**: `sim/battle-queue.ts` -> `resolveAction()` and `addChoice()`.
-   **Sorting Logic**: `sim/battle.ts` -> `comparePriority()` and `speedSort()`.
-   **End-of-Turn Order**: `sim/battle.ts` -> `fieldEvent('Residual')` and `data/` files (checking `onResidualOrder` property).
-   **Stat Access**: `sim/pokemon.ts` -> `getActionSpeed()`.