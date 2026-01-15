# Pokemon State (Entity)

The `Pokemon` entity represents a specific instance of a creature during a battle. It holds all temporary and permanent state required for mechanics to function.

## 1. Core State Properties

-   **`hp` / `maxhp`**: Current and maximum Hit Points.
-   **`status`**: The current Major Status condition (e.g., `psn`, `brn`).
-   **`moveSlots`**: List of current moves and their remaining PP.
-   **`boosts`**: Object tracking stat stages (-6 to +6) for Atk, Def, SpA, SpD, Spe, Accuracy, and Evasion.

## 2. Temporary State (Volatiles)

Temporary conditions that are not major statuses are stored in the `volatiles` object.
-   **Examples**: `substitute`, `confusion`, `leechseed`, `protect`.
-   **Cleanup**: Almost all volatiles are cleared when the Pokémon switches out via the `clearVolatile()` function.

## 3. Transformations

-   **`transform`**: Flag for when a Pokémon has used *Transform* (Ditto/Mew). The Pokémon copies the stats, types, and moves of the target.
-   **`formeChange`**: Logic for permanent (Mega Evolution) or temporary (Castform, Cherrim) form changes.
-   **`illusion`**: Specific state tracking for the *Illusion* ability (Zoroark), storing the identity of the Pokémon it is mimicking.

## 4. Visibility and Details

-   **`details`**: A string representing species, gender, level, and shininess (e.g., `Pikachu, L50, M`).
-   **`apparentType`**: The types visible to the opponent (important for *Terastallization* or *Type Change* moves).

## 5. Implementation Reference

-   **Class Definition**: `sim/pokemon.ts`.
-   **State Serialization**: `sim/state.ts`.