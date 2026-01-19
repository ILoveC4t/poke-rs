// Scrapes test cases from smogon/damage-calc by running their test files
// with instrumentation to capture all calculate() calls.
//
// Usage (from repo root):
//   DAMAGE_CALC_PATH=/path/to/damage-calc npx tsx scripts/scrape_damage_tests.mts
//
// Output:
//   tests/fixtures/damage-calc/damage.json
//   tests/fixtures/damage-calc/stats-full.json
//   tests/fixtures/damage-calc/pokemon.json

import fs from 'fs';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import * as ts from 'typescript';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const calcRoot = process.env.DAMAGE_CALC_PATH || path.resolve(repoRoot, 'damage-calc');
const testDir = path.join(calcRoot, 'calc/src/test');
const outDir = path.resolve(repoRoot, 'tests/fixtures/damage-calc');

// --- Import real calc modules ---
const calcModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/calc.ts')).href);
const pokemonModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/pokemon.ts')).href);
const moveModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/move.ts')).href);
const fieldModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/field.ts')).href);
const dataModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/data/index.ts')).href);
const statsModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/stats.ts')).href);
const mechanicsUtilModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/mechanics/util.ts')).href);

const { calculate } = calcModule;
const { Pokemon } = pokemonModule;
const { Move } = moveModule;
const { Field, Side } = fieldModule;
const { Generations } = dataModule;
const { Stats } = statsModule;
const { getModifiedStat } = mechanicsUtilModule;

// --- Captured test cases ---
interface CapturedCase {
    id: string;
    gen: number;
    testName: string;
    attacker: {
        name: string;
        level?: number;
        item?: string;
        ability?: string;
        nature?: string;
        evs?: Record<string, number>;
        ivs?: Record<string, number>;
        boosts?: Record<string, number>;
        status?: string;
        curHP?: number;
        teraType?: string;
        isDynamaxed?: boolean;
        weightkg?: number;
    };
    defender: {
        name: string;
        level?: number;
        item?: string;
        ability?: string;
        nature?: string;
        evs?: Record<string, number>;
        ivs?: Record<string, number>;
        boosts?: Record<string, number>;
        status?: string;
        curHP?: number;
        teraType?: string;
        isDynamaxed?: boolean;
        weightkg?: number;
    };
    move: {
        name: string;
        useZ?: boolean;
        isCrit?: boolean;
        hits?: number;
        overrides?: Record<string, unknown>;
    };
    field?: {
        weather?: string;
        terrain?: string;
        isGravity?: boolean;
        isMagicRoom?: boolean;
        isWonderRoom?: boolean;
        isAuraBreak?: boolean;
        isFairyAura?: boolean;
        isDarkAura?: boolean;
        isBeadsOfRuin?: boolean;
        isSwordOfRuin?: boolean;
        isTabletsOfRuin?: boolean;
        isVesselOfRuin?: boolean;
        attackerSide?: Record<string, unknown>;
        defenderSide?: Record<string, unknown>;
    };
    expected: {
        damage: number | number[] | [number, number];
        desc: string;
    };
    strict: boolean;
}

const capturedCases: CapturedCase[] = [];

// --- Serialization helpers ---
function serializePokemon(p: any): CapturedCase['attacker'] {
    const result: CapturedCase['attacker'] = { name: p.name };
    if (p.level !== 100) result.level = p.level;
    if (p.item) result.item = p.item;
    if (p.ability) result.ability = p.ability;
    if (p.nature) result.nature = p.nature;
    if (p.evs && Object.values(p.evs).some((v: any) => v !== 0)) result.evs = { ...p.evs };
    if (p.ivs && Object.values(p.ivs).some((v: any) => v !== 31)) result.ivs = { ...p.ivs };
    if (p.boosts && Object.values(p.boosts).some((v: any) => v !== 0)) result.boosts = { ...p.boosts };
    if (p.status) result.status = p.status;
    if (p.originalCurHP !== undefined && p.originalCurHP !== p.maxHP()) result.curHP = p.originalCurHP;
    if (p.teraType) result.teraType = p.teraType;
    if (p.isDynamaxed) result.isDynamaxed = p.isDynamaxed;

    // Always include weight
    if (p.weightkg !== undefined) {
        result.weightkg = p.weightkg;
    } else if (p.species && p.species.weightkg) {
        result.weightkg = p.species.weightkg;
    } else if (p.gen && p.gen.species) {
         // Fallback if species object is not directly on p (though usually it is)
         const data = p.gen.species.get(p.name);
         if (data) result.weightkg = data.weightkg;
    }

    return result;
}

function serializeMove(m: any): CapturedCase['move'] {
    const result: CapturedCase['move'] = { name: m.name };
    if (m.useZ) result.useZ = m.useZ;
    if (m.isCrit) result.isCrit = m.isCrit;
    if (m.hits && m.hits > 1) result.hits = m.hits;
    if (m.overrides && Object.keys(m.overrides).length > 0) result.overrides = { ...m.overrides };
    return result;
}

function serializeField(f: any): CapturedCase['field'] | undefined {
    if (!f) return undefined;
    const result: CapturedCase['field'] = {};
    if (f.weather) result.weather = f.weather;
    if (f.terrain) result.terrain = f.terrain;
    if (f.isGravity) result.isGravity = f.isGravity;
    if (f.isMagicRoom) result.isMagicRoom = f.isMagicRoom;
    if (f.isWonderRoom) result.isWonderRoom = f.isWonderRoom;
    if (f.isAuraBreak) result.isAuraBreak = f.isAuraBreak;
    if (f.isFairyAura) result.isFairyAura = f.isFairyAura;
    if (f.isDarkAura) result.isDarkAura = f.isDarkAura;
    if (f.isBeadsOfRuin) result.isBeadsOfRuin = f.isBeadsOfRuin;
    if (f.isSwordOfRuin) result.isSwordOfRuin = f.isSwordOfRuin;
    if (f.isTabletsOfRuin) result.isTabletsOfRuin = f.isTabletsOfRuin;
    if (f.isVesselOfRuin) result.isVesselOfRuin = f.isVesselOfRuin;

    // Serialize sides if they have non-default values
    if (f.attackerSide) {
        const side: Record<string, unknown> = {};
        for (const [key, val] of Object.entries(f.attackerSide)) {
            if (val && val !== false && val !== 0) side[key] = val;
        }
        if (Object.keys(side).length > 0) result.attackerSide = side;
    }
    if (f.defenderSide) {
        const side: Record<string, unknown> = {};
        for (const [key, val] of Object.entries(f.defenderSide)) {
            if (val && val !== false && val !== 0) side[key] = val;
        }
        if (Object.keys(side).length > 0) result.defenderSide = side;
    }

    if (Object.keys(result).length === 0) return undefined;
    return result;
}

// --- Process a single test file ---
async function processTestFile(filePath: string): Promise<CapturedCase[]> {
    console.log(`Reading tests from ${filePath}...`);
    let tsCode = await fs.promises.readFile(filePath, 'utf-8');

    // Remove import statements (we inject dependencies via scope)
    tsCode = tsCode.replace(/^import\s+.*?(?:from\s+)?['"][^'"]*['"];?\s*$/gm, '');

    // Remove export statements
    tsCode = tsCode.replace(/^export\s+/gm, '');

    // Transpile TS -> JS
    const jsCode = ts.transpileModule(tsCode, {
        compilerOptions: {
            target: ts.ScriptTarget.ES2020,
            module: ts.ModuleKind.CommonJS,
            esModuleInterop: true,
        },
    }).outputText;

    console.log(`Executing ${path.basename(filePath)} with instrumentation...`);

    const fileCases: CapturedCase[] = [];
    let currentTestName = '';
    let currentGen = 0;

    // --- Spy calculate function ---
    function createCalculateSpy(gen: any) {
        return (attacker: any, defender: any, move: any, field?: any) => {
            const result = calculate(gen, attacker, defender, move, field);

            // ID will be assigned later to ensure deterministic order
            const caseId = '';

            fileCases.push({
                id: caseId,
                gen: gen.num,
                testName: currentTestName,
                attacker: serializePokemon(attacker),
                defender: serializePokemon(defender),
                move: serializeMove(move),
                field: serializeField(field),
                expected: {
                    damage: result.damage,
                    desc: result.desc(),
                },
                strict: false, // Default to false
            });

            // Patch toMatch for the 'tests' helper pattern
            (result as any).toMatch = (_genNum: any, _expected: any) => { };

            return result;
        };
    }

    // --- Mock test runner functions ---
    function describeMock(_name: string, fn: () => void) {
        fn();
    }

    function testMock(name: string, fn: () => void) {
        const prevTestName = currentTestName;
        // Replace [object Object] with actual gen number if present
        currentTestName = name.replace('[object Object]', `${currentGen}`);
        try {
            fn();
        } catch (e) {
            // Ignore assertion errors - we just want to run the code
        }
        currentTestName = prevTestName;
    }

    function expectMock(_val: any) {
        return {
            toBe: () => { },
            toEqual: () => { },
            toMatch: () => { },
            toMatchSnapshot: () => { },
            toBeGreaterThan: () => { },
            toBeLessThan: () => { },
            toBeGreaterThanOrEqual: () => { },
            toBeLessThanOrEqual: () => { },
            toContain: () => { },
            toBeDefined: () => { },
            toBeUndefined: () => { },
            toBeTruthy: () => { },
            toBeFalsy: () => { },
            toThrow: () => { },
            not: {
                toBe: () => { },
                toEqual: () => { },
                toMatch: () => { },
                toContain: () => { },
                toBeDefined: () => { },
                toBeUndefined: () => { },
                toBeTruthy: () => { },
                toBeFalsy: () => { },
                toThrow: () => { },
            },
        };
    }

    // --- inGens helper mock ---
    function inGensMock(from: number | ((ctx: any) => void), to?: number, fn?: (ctx: any) => void) {
        // Handle overload: inGens(fn) -> all gens
        if (typeof from === 'function') {
            fn = from;
            from = 1;
            to = 9;
        }
        // Handle overload: inGens(from, fn) -> single gen
        if (typeof to === 'function') {
            fn = to;
            to = from as number;
        }

        for (let i = from as number; i <= (to as number); i++) {
            currentGen = i;
            try {
                const gen = Generations.get(i);
                fn!({
                    gen,
                    calculate: createCalculateSpy(gen),
                    Pokemon: (name: string, opts?: any) => new Pokemon(gen, name, opts),
                    Move: (name: string, opts?: any) => new Move(gen, name, opts),
                    Field: (opts?: any) => new Field(opts),
                    Side: (opts?: any) => new Side(opts),
                });
            } catch (e) {
                // Some gens may not support certain features
            }
        }
    }

    // --- inGen helper mock (single gen version) ---
    function inGenMock(genNum: number, fn: (ctx: any) => void) {
        currentGen = genNum;
        try {
            const gen = Generations.get(genNum);
            fn({
                gen,
                calculate: createCalculateSpy(gen),
                Pokemon: (name: string, opts?: any) => new Pokemon(gen, name, opts),
                Move: (name: string, opts?: any) => new Move(gen, name, opts),
                Field: (opts?: any) => new Field(opts),
                Side: (opts?: any) => new Side(opts),
            });
        } catch (e) {
            // Gen may not support certain features
        }
    }

    // --- tests helper mock (multi-gen expect pattern) ---
    function testsMock(name: string, fn: (ctx: any) => void) {
        // The 'tests' helper runs across all gens and provides a toMatch helper
        for (let i = 1; i <= 9; i++) {
            currentGen = i;
            currentTestName = `${name} (gen ${i})`;
            try {
                const gen = Generations.get(i);
                fn({
                    gen,
                    calculate: createCalculateSpy(gen),
                    Pokemon: (name: string, opts?: any) => new Pokemon(gen, name, opts),
                    Move: (name: string, opts?: any) => new Move(gen, name, opts),
                    Field: (opts?: any) => new Field(opts),
                    Side: (opts?: any) => new Side(opts),
                });
            } catch (e) {
                // Gen may not support certain features
            }
        }
    }

    // Create a sandboxed execution context
    const runTests = new Function(
        'describe', 'test', 'it', 'expect', 'inGens', 'inGen', 'tests',
        jsCode
    );

    try {
        runTests(describeMock, testMock, testMock, expectMock, inGensMock, inGenMock, testsMock);
    } catch (e) {
        console.error(`Error running captured code from ${filePath}:`, e);
    }
    return fileCases;
}

// --- Run the scraper ---
async function run() {
    // 1. Main calc test
    const calcTestPath = path.join(testDir, 'calc.test.ts');
    if (!fs.existsSync(calcTestPath)) {
        console.error(`Could not find upstream test file at: ${calcTestPath}`);
        console.error('Please ensure DAMAGE_CALC_PATH points to the damage-calc repository.');
        process.exit(1);
    }

    const tasks: Promise<CapturedCase[]>[] = [];
    tasks.push(processTestFile(calcTestPath));

    // 2. Mechanics tests
    const mechanicsDir = path.join(testDir, 'mechanics');
    if (fs.existsSync(mechanicsDir)) {
        const mechanicFiles = fs.readdirSync(mechanicsDir)
            .filter(f => f.endsWith('.test.ts'))
            .map(f => path.join(mechanicsDir, f));

        for (const file of mechanicFiles) {
            tasks.push(processTestFile(file));
        }
    }

    // 3. Items and Abilities tests (if they exist)
    const extraFiles = ['items.test.ts', 'abilities.test.ts'];
    for (const f of extraFiles) {
        const p = path.join(testDir, f);
        if (fs.existsSync(p)) {
            tasks.push(processTestFile(p));
        }
    }

    const results = await Promise.all(tasks);
    for (const cases of results) {
        capturedCases.push(...cases);
    }

    // Assign IDs deterministically
    for (let i = 0; i < capturedCases.length; i++) {
        const c = capturedCases[i];
        c.id = `gen${c.gen}-${c.testName.replace(/[^a-zA-Z0-9]/g, '-')}-${i}`;
    }

    console.log(`Captured ${capturedCases.length} test scenarios.`);

    // Write output
    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, 'damage.json');

    const output = {
        meta: {
            source: 'smogon/damage-calc/calc/src/test/',
            calcPath: calcRoot,
            generatedAt: new Date().toISOString(),
            caseCount: capturedCases.length,
        },
        cases: capturedCases,
    };

    fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
    console.log(`Wrote fixtures: ${outPath}`);
}

// --- Generate stats-full.json from stats.test.ts ---
async function generateStatsFixtures() {
    console.log('Generating stats-full fixtures...');
    const cases: any[] = [];

    // displayStat tests
    const statNames = ['hp', 'atk', 'def', 'spa', 'spd', 'spe', 'spc'] as const;
    for (const stat of statNames) {
        cases.push({
            id: `displayStat-${stat}`,
            fn: 'displayStat',
            input: stat,
            expected: Stats.displayStat(stat),
        });
    }

    // calcStat across all gens with Adamant nature
    for (let gen = 1; gen <= 9; gen++) {
        const genObj = Generations.get(gen);
        for (const stat of ['hp', 'atk', 'def', 'spa', 'spd', 'spe'] as const) {
            cases.push({
                id: `calcStat-gen${gen}-${stat}-adamant`,
                fn: 'calcStat',
                gen,
                stat,
                base: 100,
                iv: 31,
                ev: 252,
                level: 100,
                nature: 'Adamant',
                expected: Stats.calcStat(genObj, stat, 100, 31, 252, 100, 'Adamant'),
            });
        }
    }

    // Shedinja special case (HP always 1)
    cases.push({
        id: 'calcStat-shedinja-hp',
        fn: 'calcStat',
        gen: 8,
        stat: 'hp',
        base: 1,
        iv: 31,
        ev: 252,
        level: 100,
        nature: 'Jolly',
        expected: Stats.calcStat(Generations.get(8), 'hp', 1, 31, 252, 100, 'Jolly'),
    });

    // No nature (undefined)
    cases.push({
        id: 'calcStat-no-nature',
        fn: 'calcStat',
        gen: 8,
        stat: 'atk',
        base: 100,
        iv: 31,
        ev: 252,
        level: 100,
        nature: null,
        expected: Stats.calcStat(Generations.get(8), 'atk', 100, 31, 252, 100),
    });

    // DV <-> IV conversions
    for (let dv = 0; dv <= 15; dv++) {
        const iv = Stats.DVToIV(dv);
        cases.push({
            id: `DVToIV-${dv}`,
            fn: 'DVToIV',
            input: dv,
            expected: iv,
        });
        cases.push({
            id: `IVToDV-${iv}`,
            fn: 'IVToDV',
            input: iv,
            expected: Stats.IVToDV(iv),
        });
    }

    // HP DV calculation from other DVs
    const hpDvCases = [
        { atk: 15, def: 15, spc: 15, spe: 15, expected: 15 },
        { atk: 5, def: 15, spc: 13, spe: 13, expected: 15 },
        { atk: 15, def: 3, spc: 11, spe: 10, expected: 13 },
    ];
    for (const hpCase of hpDvCases) {
        cases.push({
            id: `getHPDV-${hpCase.atk}-${hpCase.def}-${hpCase.spc}-${hpCase.spe}`,
            fn: 'getHPDV',
            input: {
                atk: Stats.DVToIV(hpCase.atk),
                def: Stats.DVToIV(hpCase.def),
                spc: Stats.DVToIV(hpCase.spc),
                spe: Stats.DVToIV(hpCase.spe),
            },
            expected: hpCase.expected,
        });
    }

    // Gen 2 stat modifications (after Curse)
    const gen2ModCases = [
        { stat: 158, boost: -1, desc: 'Snorlax after Curse' },
        { stat: 238, boost: -1, desc: 'Skarmory after Curse' },
    ];
    for (const modCase of gen2ModCases) {
        cases.push({
            id: `getModifiedStat-gen2-${modCase.stat}-${modCase.boost}`,
            fn: 'getModifiedStat',
            gen: 2,
            stat: modCase.stat,
            boost: modCase.boost,
            desc: modCase.desc,
            expected: getModifiedStat(modCase.stat, modCase.boost, Generations.get(2)),
        });
    }

    const output = {
        meta: {
            source: 'smogon/damage-calc/calc/src/test/stats.test.ts',
            calcPath: calcRoot,
            generatedAt: new Date().toISOString(),
            caseCount: cases.length,
        },
        cases,
    };

    const outPath = path.join(outDir, 'stats-full.json');
    fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
    console.log(`Wrote ${cases.length} stats cases to: ${outPath}`);
}

// --- Generate pokemon.json from pokemon.test.ts ---
async function generatePokemonFixtures() {
    console.log('Generating pokemon fixtures...');
    const cases: any[] = [];

    // Pokemon defaults test cases
    const defaultTests = [
        { gen: 7, name: 'Gengar' },
        { gen: 1, name: 'Tauros' },
        { gen: 8, name: 'Snorlax' },
        { gen: 9, name: 'Pikachu' },
    ];

    for (const tc of defaultTests) {
        try {
            const p = new Pokemon(Generations.get(tc.gen), tc.name);
            cases.push({
                id: `pokemon-defaults-gen${tc.gen}-${tc.name}`,
                gen: tc.gen,
                name: tc.name,
                expected: {
                    types: [...p.types],
                    stats: { ...p.stats },
                    ability: p.ability,
                    weightkg: p.weightkg,
                    gender: p.gender,
                },
            });
        } catch (e) {
            // Pokemon might not exist in that gen
        }
    }

    // Pokemon with full options
    const fullTests = [
        {
            gen: 7,
            name: 'Suicune',
            opts: {
                level: 50,
                ability: 'Inner Focus',
                item: 'Leftovers',
                nature: 'Bold',
                ivs: { spa: 30 },
                evs: { spd: 4, def: 252, hp: 252 },
                boosts: { atk: -1, spa: 2, spd: 1 },
                curHP: 60,
                status: 'tox',
                toxicCounter: 2,
            },
        },
        {
            gen: 1,
            name: 'Tauros',
            opts: {
                level: 100,
                ivs: { spc: 20, def: 16 },
                evs: { atk: 200 },
            },
        },
    ];

    for (const tc of fullTests) {
        try {
            const p = new Pokemon(Generations.get(tc.gen), tc.name, tc.opts);
            cases.push({
                id: `pokemon-full-gen${tc.gen}-${tc.name}`,
                gen: tc.gen,
                name: tc.name,
                opts: tc.opts,
                expected: {
                    types: [...p.types],
                    stats: { ...p.stats },
                    ivs: { ...p.ivs },
                    evs: { ...p.evs },
                    ability: p.ability,
                    nature: p.nature,
                    level: p.level,
                    curHP: p.curHP(),
                    maxHP: p.maxHP(),
                },
            });
        } catch (e) {
            // Skip if error
        }
    }

    // getForme tests
    const formeTests = [
        { gen: 1, name: 'Gengar', item: undefined, move: undefined, expected: 'Gengar' },
        { gen: 7, name: 'Gengar', item: 'Black Sludge', move: 'Hypnosis', expected: 'Gengar' },
        { gen: 7, name: 'Gengar', item: 'Gengarite', move: 'Hypnosis', expected: 'Gengar-Mega' },
        { gen: 7, name: 'Charizard', item: undefined, move: undefined, expected: 'Charizard' },
        { gen: 7, name: 'Charizard', item: 'Charizardite X', move: undefined, expected: 'Charizard-Mega-X' },
        { gen: 7, name: 'Charizard', item: 'Charizardite Y', move: undefined, expected: 'Charizard-Mega-Y' },
        { gen: 7, name: 'Mewtwo', item: 'Choice Specs', move: 'Psystrike', expected: 'Mewtwo' },
        { gen: 7, name: 'Mewtwo', item: 'Mewtwonite X', move: 'Psystrike', expected: 'Mewtwo-Mega-X' },
        { gen: 7, name: 'Mewtwo', item: 'Mewtwonite Y', move: 'Psystrike', expected: 'Mewtwo-Mega-Y' },
        { gen: 7, name: 'Groudon', item: 'Choice Band', move: 'Earthquake', expected: 'Groudon' },
        { gen: 7, name: 'Groudon', item: 'Red Orb', move: 'Earthquake', expected: 'Groudon-Primal' },
        { gen: 7, name: 'Kyogre', item: 'Choice Specs', move: 'Surf', expected: 'Kyogre' },
        { gen: 7, name: 'Kyogre', item: 'Blue Orb', move: 'Surf', expected: 'Kyogre-Primal' },
    ];

    for (const ft of formeTests) {
        try {
            const result = Pokemon.getForme(Generations.get(ft.gen), ft.name, ft.item, ft.move);
            cases.push({
                id: `getForme-gen${ft.gen}-${ft.name}-${ft.item || 'none'}`,
                fn: 'getForme',
                gen: ft.gen,
                name: ft.name,
                item: ft.item,
                move: ft.move,
                expected: result,
            });
        } catch (e) {
            // Skip if error
        }
    }

    // Move Z-move transformation test
    try {
        const zMove = new Move(Generations.get(7), 'Blizzard', { useZ: true });
        cases.push({
            id: 'move-zmove-blizzard',
            fn: 'Move',
            gen: 7,
            name: 'Blizzard',
            opts: { useZ: true },
            expected: {
                name: zMove.name,
                bp: zMove.bp,
            },
        });
    } catch (e) {
        // Skip if error
    }

    const output = {
        meta: {
            source: 'smogon/damage-calc/calc/src/test/pokemon.test.ts + move.test.ts',
            calcPath: calcRoot,
            generatedAt: new Date().toISOString(),
            caseCount: cases.length,
        },
        cases,
    };

    const outPath = path.join(outDir, 'pokemon.json');
    fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
    console.log(`Wrote ${cases.length} pokemon cases to: ${outPath}`);
}

// --- Main execution ---
async function main() {
    fs.mkdirSync(outDir, { recursive: true });

    await run();
    await generateStatsFixtures();
    await generatePokemonFixtures();

    console.log('\nAll fixtures generated successfully!');
}

main().catch((e) => {
    console.error('Fatal error:', e);
    process.exit(1);
});
