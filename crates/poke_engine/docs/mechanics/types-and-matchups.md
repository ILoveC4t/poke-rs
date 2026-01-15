# Types and Matchups

The type system determines how moves interact with different Pokémon species. It governs effectiveness multipliers and complete immunities.

## 1. Effectiveness Multipliers

Each type matchup has a base multiplier.

- **Super Effective**: 2.0x
- **Neutral**: 1.0x
- **Not Very Effective**: 0.5x
- **Immune**: 0.0x

### Dual Types
For Pokémon with two types, the multipliers for each type are multiplied together.
- *Fire vs. Grass/Bug*: 2.0 * 2.0 = **4.0x** (Double Super Effective).
- *Electric vs. Ground/Rock*: 0.0 * 1.0 = **0.0x** (Immune).

## 2. STAB (Same Type Attack Bonus)

If a move's type matches one of the attacker's current types, the damage is boosted.
- **Base STAB**: 1.5x.
- **Adaptability**: 2.0x.
- **Terastallization**: If the Tera-type matches a base type, STAB becomes 2.0x.

## 3. Immunities

Immunities can be granted by Type, Ability, or Item.

- **Type-based**:
    - Ground immune to Electric.
    - Flying immune to Ground.
    - Fairy immune to Dragon.
    - Steel immune to Poison.
- **Effect-based**:
    - **Abilities**: *Levitate* (immune to Ground), *Volt Absorb* (immune to Electric), *Sap Sipper* (immune to Grass).
    - **Items**: *Air Balloon* (immune to Ground until popped).

## 4. Implementation Reference

- **Data Structure**: `sim/dex-data.ts` -> `TypeInfo`.
- **Matchup Logic**: `sim/battle.ts` -> `getEffectiveness()` and `getImmunity()`.
- **Damage Integration**: `sim/battle-actions.ts` -> `modifyDamage()`.
- **Type Chart Data**: `data/typechart.ts`.