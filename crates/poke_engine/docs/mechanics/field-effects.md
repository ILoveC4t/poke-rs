# Field Effects

Field effects are conditions that affect the entire battlefield or specific sides of it. The engine manages these through the `Field` and `Side` classes using a hook-based interaction model.

## 1. Weather

Weather affects damage calculation, accuracy, and specific ability triggers.

-   **Types**: Sun (`sunnyday`), Rain (`raindance`), Sandstorm (`sandstorm`), and Snow/Hail (`snowscape`/`hail`).
-   **Setting**: Managed by `Field.setWeather()`. Duration is typically 5 turns (8 with specific items).
-   **Suppression**: Abilities like *Air Lock* and *Cloud Nine* trigger the `suppressingWeather()` check. When active, weather effects (damage modifiers, ability triggers) are ignored, though the weather remains on the field.
-   **Primal Weather**: Effects like *Desolate Land* or *Primordial Sea* have `duration: 0` and cannot be overridden by standard weather moves.

## 2. Terrains

Terrains affect Pokémon that are "grounded" (not Flying-type, not having Levitate, not holding Air Balloon).

-   **Types**: Electric, Grassy, Misty, and Psychic.
-   **Setting**: Managed by `Field.setTerrain()`.
-   **Effects**:
    -   **Electric**: Boosts Electric moves (1.3x or 1.5x) and prevents Sleep.
    -   **Grassy**: Boosts Grass moves, heals grounded Pokémon, and halves damage from *Earthquake/Magnitude*.
    -   **Misty**: Halves Dragon damage and prevents status conditions.
    -   **Psychic**: Boosts Psychic moves and prevents priority moves against grounded targets.

## 3. Pseudo-Weather

Pseudo-weathers are field-wide conditions that don't occupy the "weather slot."

-   **Setting**: Managed by `Field.addPseudoWeather()`.
-   **Examples**:
    -   **Trick Room**: Inverts speed priority.
    -   **Gravity**: Increases move accuracy, grounds all Pokémon, and prevents certain moves (e.g., *Fly*, *High Jump Kick*).
    -   **Magic Room**: Suppresses the effects of held items for 5 turns.

## 4. Side Conditions

Side conditions affect only one side of the field and are managed by the `Side` class.

-   **Hazards**: Permanent until removed (e.g., *Rapid Spin*, *Defog*).
    -   **Stealth Rock**: Deals damage on switch-in based on type effectiveness.
    -   **Spikes**: Deals fixed % damage on switch-in (up to 3 layers).
    -   **Toxic Spikes**: Inflicts Poison or Toxic on switch-in.
-   **Screens**: Temporary protection.
    -   **Reflect**: Halves damage from Physical moves.
    -   **Light Screen**: Halves damage from Special moves.
    -   **Aurora Veil**: Combination of both (only usable in Snow/Hail).

## 5. Implementation Reference

-   **Global Field Management**: `sim/field.ts`.
-   **Side-Specific Management**: `sim/side.ts` -> `addSideCondition()`.
-   **Weather Definitions**: `data/conditions.ts`.
-   **Terrain/Hazard Definitions**: `data/moves.ts` (often as a `condition` object within the move).