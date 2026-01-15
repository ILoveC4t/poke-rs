# Sides and Players (Entity)

The `Side` entity represents a player and their team of Pokémon. It manages team-wide conditions and coordinates active slots.

## 1. Team Management

-   **`pokemon`**: The full list of Pokémon on the player's team (usually 6).
-   **`active`**: An array representing the Pokémon currently on the field.
    -   **Singles**: 1 active slot.
    -   **Doubles**: 2 active slots.
    -   **Triples**: 3 active slots.
-   **`pokemonLeft`**: Count of Pokémon remaining that are not fainted.

## 2. Side Conditions

Effects that apply to the player's half of the field are stored here.
-   **Hazards**: `stealthrock`, `spikes`, `toxicspikes`, `stickyweb`.
-   **Screens**: `reflect`, `lightscreen`, `auroraveil`.
-   **Tailwind**: Team-wide speed boost.

## 3. Choice Processing

The `Side` class is responsible for taking user input and validating it against the current battle request.
-   **`chooseMove`**: Validates that the move is known, has PP, and isn't disabled.
-   **`chooseSwitch`**: Validates that the target is not fainted and not already active.

## 4. Team-Wide Counters

-   **`zMoveUsed`**: Boolean tracking if the player has already used their one Z-Move.
-   **`dynamaxUsed`**: Boolean tracking if the player has used their one Dynamax.
-   **`totalFainted`**: Tracks how many Pokémon have fainted throughout the match.

## 5. Implementation Reference

-   **Class Definition**: `sim/side.ts`.
-   **Choice Resolution**: `sim/battle.ts` -> `choose()`.