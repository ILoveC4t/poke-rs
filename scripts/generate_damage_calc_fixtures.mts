// Generates vendored JSON fixtures using smogon/damage-calc as the source of truth.
//
// Usage (from repo root):
//   DAMAGE_CALC_PATH=/path/to/damage-calc npx tsx scripts/generate_damage_calc_fixtures.mts
//
// Output:
//   tests/fixtures/damage-calc/stats.json

import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

const repoRoot = process.cwd();
const calcRoot = process.env.DAMAGE_CALC_PATH || path.resolve(repoRoot, 'damage-calc');
const outDir = path.resolve(repoRoot, 'tests/fixtures/damage-calc');

const statsModule = await import(
    pathToFileURL(path.join(calcRoot, 'calc/src/stats.ts')).href
);
const dataModule = await import(
    pathToFileURL(path.join(calcRoot, 'calc/src/data/index.ts')).href
);

const { Stats } = statsModule as { Stats: any };
const { Generations } = dataModule as { Generations: any };

const gen = Generations.get(9);

const cases = [
    {
        id: 'base100-adamant-hp',
        gen: 9,
        stat: 'hp',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base100-adamant-atk',
        gen: 9,
        stat: 'atk',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base100-adamant-def',
        gen: 9,
        stat: 'def',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base100-adamant-spa',
        gen: 9,
        stat: 'spa',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base100-adamant-spd',
        gen: 9,
        stat: 'spd',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base100-adamant-spe',
        gen: 9,
        stat: 'spe',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Adamant',
    },
    {
        id: 'base35-timid-spe-50',
        gen: 9,
        stat: 'spe',
        base: 90,
        iv: 31,
        ev: 252,
        level: 50,
        nature: 'Timid',
    },
];

const casesWithExpected = cases.map((c) => ({
    ...c,
    expected: Stats.calcStat(gen, c.stat, c.base, c.iv, c.ev, c.level, c.nature),
}));

const output = {
    meta: {
        source: 'smogon/damage-calc',
        calcPath: calcRoot,
        generatedAt: new Date().toISOString(),
    },
    cases: casesWithExpected,
};

fs.mkdirSync(outDir, { recursive: true });
const outPath = path.join(outDir, 'stats.json');
fs.writeFileSync(outPath, JSON.stringify(output, null, 2));

console.log(`Wrote fixtures: ${outPath}`);
// FIXME: Expand fixtures to include damage calc scenarios and edge cases.
