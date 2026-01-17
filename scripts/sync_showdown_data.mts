// Sync data from smogon/pokemon-showdown into local JSON files.
//
// Usage (from repo root):
//   SHOWDOWN_PATH=/path/to/pokemon-showdown npx tsx scripts/sync_showdown_data.mts
//
// Output:
//   data/*.json

import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

const repoRoot = process.cwd();
const showdownRoot = process.env.SHOWDOWN_PATH || path.resolve(repoRoot, 'pokemon-showdown');
const outputDir = path.resolve(repoRoot, 'data');

const pokedexModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/pokedex.ts')).href
);
const movesModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/moves.ts')).href
);
const typechartModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/typechart.ts')).href
);
const itemsModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/items.ts')).href
);
const abilitiesModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/abilities.ts')).href
);
const naturesModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/natures.ts')).href
);
const learnsetsModule = await import(
    pathToFileURL(path.join(showdownRoot, 'data/learnsets.ts')).href
);

const { Pokedex } = pokedexModule as { Pokedex: unknown };
const { Moves } = movesModule as { Moves: unknown };
const { TypeChart } = typechartModule as { TypeChart: unknown };
const { Items } = itemsModule as { Items: unknown };
const { Abilities } = abilitiesModule as { Abilities: unknown };
const { Natures } = naturesModule as { Natures: unknown };
const { Learnsets } = learnsetsModule as { Learnsets: unknown };

const save = (name: string, data: unknown) => {
    if (data === undefined) {
        console.error(`Error: Data for '${name}' is undefined. The export name might have changed.`);
        process.exit(1);
    }

    const filePath = path.join(outputDir, `${name}.json`);
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));
    console.log(`Saved: ${filePath}`);
};

if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

console.log('Starting Showdown data extraction...');
save('pokedex', Pokedex);
save('moves', Moves);
save('typechart', TypeChart);
save('items', Items);
save('abilities', Abilities);
save('natures', Natures);
save('learnsets', Learnsets);
console.log('Extraction complete.');
// FIXME: Add version metadata from pokemon-showdown to the output.
