# Battle Rules

Battle Rules define the constraints and environment under which a match is conducted, ensuring competitive integrity and format consistency.

## 1. Standard Clauses

Clauses are optional rules that can be enabled to modify battle behavior.

- **Sleep Clause Mod**: Prevents a player from putting more than one of the opponent's Pokémon to sleep at a time.
- **Species Clause**: Prevents a player from having two Pokémon of the same National Dex number on their team.
- **Evasion Clause**: Bans moves that specifically increase a Pokémon's Evasion stage (e.g., *Double Team*, *Minimize*).
- **Endless Battle Clause**: Prevents matches from continuing indefinitely by forcing a loss on a player who intentionally prevents the battle from ending.

## 2. Format Settings

Each battle format (e.g., Gen 9 OU, VGC 2024) has specific configurations:

- **Level Scaling**: Pokémon are usually scaled to Level 50 or Level 100.
- **Team Size**: Defines how many Pokémon are in a team and how many are brought to the actual battle (e.g., "Bring 6, Pick 4" for VGC).
- **Timers**: Enforces strict time limits for Team Preview and individual turn decisions.

## 3. RNG and Integrity

The engine uses a Pseudo-Random Number Generator (PRNG) to handle chance-based events.

- **PRNG Seed**: Every battle is initialized with a seed. This seed ensures that the sequence of "random" events is deterministic for a given set of inputs, allowing for perfect replays.
- **Rollbacks**: The engine state can be serialized and restored, allowing the system to handle disconnects or "undo" actions in specific debug scenarios.

## 4. Implementation Reference

- **Clause Logic**: `sim/rulesets.ts`.
- **Format Definitions**: `config/formats.ts`.
- **PRNG Implementation**: `sim/prng.ts`.
