# Damage Calculation

The damage calculation in the Pokémon engine is a two-step process:
1.  **Base Damage Calculation**: Determining the raw damage based on Level, Stats, and Base Power.
2.  **Modifier Application**: Applying sequential multipliers for mechanics like STAB, Type Effectiveness, and Items.

## 1. The Damage Formula

The base damage is calculated using the following formula (using integer truncation at each step):

```text
BaseDamage = floor(floor(floor(floor(2 * Level / 5 + 2) * BasePower * Attack) / Defense) / 50)
```

In the second phase (`modifyDamage`), an initial `+ 2` is added to this result before further multipliers are applied.

### Stats Selection
- **Physical Moves**: Use the `atk` stat of the attacker and `def` stat of the defender.
- **Special Moves**: Use the `spa` stat of the attacker and `spd` stat of the defender.
- **Exceptions**: Moves like *Psyshock* (Special vs Physical Defense) or *Body Press* (Defense as Attack) override these defaults.

## 2. Modifier Sequence

Multipliers are applied in a strict sequence. Truncation (`tr`) is often applied after each multiplication to maintain cartridge accuracy.

1.  **Base Power (BP)**: Modified by events (e.g., *Technician*, *Sheer Force*) and items (e.g., *Muscle Band*, *Plate* boosts).
2.  **Stat Modifiers**: Attack and Defense are calculated with boosts (-6 to +6).
    - **Critical Hits**: Ignore the attacker's negative offensive boosts and the defender's positive defensive boosts.
3.  **Multi-target (Spread)**: 0.75x if the move hits multiple targets (0.5x in Battle Royales).
4.  **Weather**:
    - 1.5x for Water in Rain / Fire in Sun.
    - 0.5x for Fire in Rain / Water in Sun.
5.  **Critical Hit**: 1.5x (Gen 6+) or 2x (Gen 1-5).
    - **Probability Table (Gen 7+)**:
        - Stage 0: 1/24 (~4.17%)
        - Stage 1: 1/8 (12.5%)
        - Stage 2: 1/2 (50%)
        - Stage 3+: 1/1 (100%)
    - *Note:* Sources of crit stages include *Scope Lens*, *Focus Energy*, and high-crit moves like *Stone Edge*.
6.  **Random Factor**: A random integer multiplier between 85 and 100, then divided by 100 (0.85x to 1.0x).
7.  **STAB (Same Type Attack Bonus)**:
    - 1.5x if the move matches the user's type.
    - 2x if the user has *Adaptability* or is Terastallized into that type.
8.  **Type Effectiveness**:
    - **Super Effective**: 2x or 4x.
    - **Not Very Effective**: 0.5x or 0.25x.
    - **Immune**: 0x.
9.  **Burn**: 0.5x to Physical moves if the attacker is burned (unless they have *Guts* or use *Facade*).
10. **Final Modifiers**: Items like *Life Orb* (1.3x), *Expert Belt* (1.2x), and abilities like *Solid Rock* (0.75x).

## 3. Implementation Reference

- **Core Logic**: `sim/battle-actions.ts` -> `getDamage()` and `modifyDamage()`.
- **Critical Hit Logic**: `sim/battle-actions.ts` -> `getDamage()` (calculates `critMult` array).
- **Stat Calculation**: `sim/pokemon.ts` -> `calculateStat()`.
- **Matchup Logic**: `sim/battle.ts` -> `getEffectiveness()`.

## 4. Key Functions
- `tr(num)`: Truncates a number to an integer.
- `modify(value, multiplier)`: Multiplies and potentially truncates according to specific generation rules.