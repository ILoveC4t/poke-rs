# Items

Held items provide variety of effects, from stat boosts to status recovery. The engine tracks held items and manages their unique consumption rules.

## Implementation

Items are implemented primarily through the damage calculation pipeline and battle state checks.

- `src/damage/modifiers.rs`: Applies damage-modifying items (Life Orb, Choice Band, etc.)
- `src/state.rs`: Handles effective speed modifiers (Choice Scarf, Iron Ball)
- `src/items/`: (Planned) Item hook system similar to abilities

## 1. Item Lifecycle

Items can be passive, consumed upon use, or knocked off.

-   **`eatItem`**: Specifically for Berries. Can be triggered by the user's HP dropping, or by moves like *Bug Bite/Pluck*.
-   **`useItem`**: For items like *Gems*, *Focus Sash*, or *Power Herb* that are consumed but not "eaten."
-   **`takeItem`**: Occurs when an item is removed via *Knock Off*, *Thief*, or *Trick*.

## 2. Item Categories

### Stat Boosters
-   **Choice Items**: *Choice Band*, *Choice Specs*, *Choice Scarf* (1.5x stat, locks user into one move).
-   **Expert Belt**: 1.2x damage if the move is Super Effective.
-   **Life Orb**: 1.3x damage but deals 10% max HP recoil per attack.

### Defensive Items
-   **Focus Sash**: Prevents fainting from full HP (consumed).
-   **Leftovers**: Heals 1/16 max HP at the end of each turn.
-   **Rocky Helmet**: Deals 1/6 max HP damage if hit by a contact move.

### Berries
-   **Recovery**: *Sitrus Berry* (heals 25%), *Lum Berry* (cures status).
-   **Resist**: *Occa Berry*, *Yache Berry* (halves damage from a super-effective move of a specific type).

### Special Class Items
-   **Mega Stones**: Required for Mega Evolution. Cannot be knocked off.
-   **Z-Crystals**: Required for Z-Moves. Cannot be knocked off.
-   **Plates/Drives/Memories**: Change the type of specific Pokémon (Arceus, Genesect, Silvally).

## 3. Suppression

The pseudo-weather **Magic Room** suppresses the effects of all held items for its duration. The ability **Klutz** prevents the user from utilizing its own held item (except for items that modify Speed or EXP).

## 4. Item-Specific Actions

Certain moves utilize the held item's properties to determine their effect.

### Fling
The move *Fling* throws the user's held item at the target.
-   **Base Power**: Derived from the `fling` property of the item definition (e.g., *Iron Ball* has 130 BP).
-   **Effect**: Some items apply a status or effect when flung (e.g., *Flame Orb* burns the target).

### Natural Gift
The move *Natural Gift* consumes the user's berry to attack.
-   **Type and Power**: Derived from the `naturalGift` property of the berry.
    -   *Example:* *Liechi Berry* provides a 100 BP Grass-type move.
    -   *Example:* *Occa Berry* provides an 80 BP Fire-type move.

## References

- [Showdown: sim/pokemon.ts](https://github.com/smogon/pokemon-showdown/blob/master/sim/pokemon.ts)
- [Bulbapedia: Held Item](https://bulbapedia.bulbagarden.net/wiki/Held_item)
