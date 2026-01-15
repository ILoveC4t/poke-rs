# Stats Calculation

The Pokémon engine calculates stats based on a combination of permanent base data and temporary in-battle modifications.

## 1. Out-of-Battle Stats

Base stats are calculated when a Pokémon is initialized or changes forme permanently.

### HP Calculation
```text
HP = floor((2 * Base + IV + floor(EV / 4)) * Level / 100) + Level + 10
```
*Note: Shedinja's HP is always locked to 1.*

### Other Stats (Atk, Def, SpA, SpD, Spe)
```text
Stat = floor(floor((2 * Base + IV + floor(EV / 4)) * Level / 100 + 5) * Nature)
```
- **Nature**: 1.1x for a boosted stat, 0.9x for a hindered stat, 1.0x otherwise.

## 2. In-Battle Stat Stages (Boosts)

Pokémon can increase or decrease their stats during battle using stages from -6 to +6.

| Stage | Multiplier |
| :--- | :--- |
| -6 | 2/8 (0.25x) |
| -5 | 2/7 (0.28x) |
| -4 | 2/6 (0.33x) |
| -3 | 2/5 (0.40x) |
| -2 | 2/4 (0.50x) |
| -1 | 2/3 (0.66x) |
| 0 | 2/2 (1.00x) |
| +1 | 3/2 (1.50x) |
| +2 | 4/2 (2.00x) |
| +3 | 5/2 (2.50x) |
| +4 | 6/2 (3.00x) |
| +5 | 7/2 (3.50x) |
| +6 | 8/2 (4.00x) |

- **Accuracy/Evasion**: Use a different scale (3/3 base, incrementing the numerator/denominator by 1 per stage).

## 3. Dynamic Modifiers

After stage multipliers are applied, the engine runs stat-specific events to apply ability and item modifiers.

- **Abilities**: *Huge Power* (2x Atk), *Chlorophyll* (2x Spe in Sun), *Slow Start* (0.5x Atk/Spe).
- **Items**: *Choice Band* (1.5x Atk), *Eviolite* (1.5x Def/SpD for NFE Pokémon).

## 4. Implementation Reference

- **Base Formula**: `sim/pokemon.ts` -> `setSpecies()`.
- **Stage Multipliers**: `sim/pokemon.ts` -> `calculateStat()`.
- **In-Battle Access**: `sim/pokemon.ts` -> `getStat()` (runs hooks for items/abilities).
- **Nature Data**: `sim/dex-data.ts` -> `Nature`.