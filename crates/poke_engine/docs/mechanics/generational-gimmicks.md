# Generational Gimmicks

Generational Gimmicks are metagame-defining mechanics introduced in specific generations that fundamentally alter a Pokémon's stats, types, or moves.

## 1. Dynamax (Gen 8)

Dynamax increases a Pokémon's size and HP for 3 turns.

- **HP Multiplier**: Dynamaxed Pokémon have their current and max HP multiplied by a factor (1.5x at Dynamax Level 0, up to 2.0x at Dynamax Level 10). When Dynamax ends, HP is scaled back down.
- **Max Moves**: All offensive moves become "Max Moves" with increased power and secondary field effects. Status moves become "Max Guard."
    - **Power Derivation**: Max Move power is based on the base move's Base Power (BP).
        - **Fighting / Poison Moves**:
            - BP < 45: 70
            - BP 45-54: 75
            - BP 55-64: 80
            - BP 65-74: 85
            - BP 75-109: 90
            - BP 110-149: 95
            - BP ≥ 150: 100
        - **Other Types**:
            - BP < 45: 90
            - BP 45-54: 100
            - BP 55-64: 110
            - BP 65-74: 120
            - BP 75-109: 130
            - BP 110-149: 140
            - BP ≥ 150: 150
- **Immunities**:
    - Weight-based moves (e.g., *Low Kick*, *Heavy Slam*) fail.
    - Flinching is prevented.
    - Forced switching moves (e.g., *Roar*, *Whirlwind*) fail.
    - Signature "OHKO" moves fail.

## 2. Z-Moves (Gen 7)

Z-Moves are powerful, one-time-use moves triggered by a held Z-Crystal.

- **Z-Power**: Offensive Z-Moves have their power derived from the base move.
    - **Calculation**:
        - BP < 60: 100
        - BP 60-69: 120
        - BP 70-79: 140
        - BP 80-89: 160
        - BP 90-99: 175
        - BP 100-109: 180
        - BP 110-119: 185
        - BP 120-129: 190
        - BP 130-139: 195
        - BP ≥ 140: 200
    - *Note:* Variable multi-hit moves (e.g., *Bullet Seed*) assume 3 hits for Z-Power calculation (`BP * 3`). Fixed multi-hit moves (e.g., *Double Hit*) typically have explicit Z-Power overrides.
- **Status Z-Effects**: Using a Z-Crystal with a Status move adds a "Z-Power" effect (e.g., *Z-Splash* raises Attack by 3 stages) before the move's standard effect.
- **Protection Bypass**: Offensive Z-Moves deal 25% damage through protection moves like *Protect*.

## 3. Terastallization (Gen 9)

Terastallization changes a Pokémon's type to its "Tera Type" for the remainder of the battle.

- **Type Change**: The Pokémon's types are replaced by its Tera Type for defensive calculations.
- **STAB Interaction**: 
    - The Pokémon retains STAB from its original types.
    - If the Tera Type matches an original type, STAB for that type increases from 1.5x to 2.0x.
- **Stellar Tera Type**: A special Tera Type that provides a one-time boost to each move type used and has unique interactions with *Tera Blast*.
- **Tera Blast**: Changes type to the user's Tera Type. It becomes Physical or Special depending on which of the user's offensive stats is higher.
- **Apparent Type**: Upon Terastallizing, the Pokémon's "Apparent Type" (what is shown to the opponent) immediately updates to match its Tera Type. This overrides any previous apparent type (e.g., from *Illusion*).

## 4. Implementation Reference

- **Dynamax Logic**: `sim/pokemon.ts` -> `volatiles['dynamax']`.
- **Z-Move Logic**: `sim/battle-actions.ts` -> `getZMove()`.
- **Power Calculation**: `sim/dex-moves.ts` -> `Move` constructor (contains the lookup tables for Z-Move and Max Move power).
- **Terastallization Logic**: `sim/pokemon.ts` -> `terastallize()`.
