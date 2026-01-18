// Generates synthetic test coverage for modifier combinations
// Usage: DAMAGE_CALC_PATH=... npx tsx scripts/generate_synthetic_fixtures.mts

import fs from 'fs';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const calcRoot = process.env.DAMAGE_CALC_PATH || path.resolve(repoRoot, 'damage-calc');
const outDir = path.resolve(repoRoot, 'tests/fixtures');

// --- Import real calc modules ---
// We need to import these dynamically to work with the damage-calc repo structure
if (!fs.existsSync(calcRoot)) {
    console.error(`DAMAGE_CALC_PATH not found: ${calcRoot}`);
    process.exit(1);
}

const calcModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/calc.ts')).href);
const pokemonModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/pokemon.ts')).href);
const moveModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/move.ts')).href);
const dataModule = await import(pathToFileURL(path.join(calcRoot, 'calc/src/data/index.ts')).href);

const { calculate } = calcModule;
const { Pokemon } = pokemonModule;
const { Move } = moveModule;
const { Generations } = dataModule;

// --- Configuration ---
const ITEMS = [
    undefined,
    'Life Orb',
    'Choice Band',
    'Choice Specs',
    'Expert Belt',
    'Muscle Band',
    'Wise Glasses',
    'Black Belt', // Type boosting
    'Metronome', // Needs context, maybe skip for simple synthetic
];

const ABILITIES = [
    undefined,
    'Huge Power',
    'Pure Power',
    'Guts',
    'Solar Power',
    'Technician',
    'Adaptability',
    'Sheer Force',
    'Overgrow', // Pinch abilities
    'Blaze',
    'Torrent',
    'Swarm',
];

const MOVES = [
    'Tackle', // Physical, Normal
    'Thunderbolt', // Special, Electric
    'Earthquake', // Physical, Ground
    'Ice Beam', // Special, Ice
    'Mach Punch', // Physical, Fighting, Priority
    'Quick Attack', // Physical, Normal, Priority
];

const ATTACKERS = [
    'Mew', // 100 base all
    'Pikachu', // Fast, frail
    'Snorlax', // Slow, bulky
    'Scizor', // Technician user
    'Charizard', // Solar Power user
];

const DEFENDERS = [
    'Mew',
    'Snorlax',
    'Chansey',
    'Skarmory',
];

// --- Generation ---

const cases: any[] = [];
const gen = Generations.get(9); // Default to Gen 9 for synthetic tests

function generate() {
    console.log('Generating synthetic fixtures...');

    let count = 0;

    for (const attackerName of ATTACKERS) {
        for (const defenderName of DEFENDERS) {
            for (const moveName of MOVES) {
                // Base check
                runCase(attackerName, defenderName, moveName);

                // Iterate items on attacker
                for (const item of ITEMS) {
                    if (item) runCase(attackerName, defenderName, moveName, { attackerItem: item });
                }

                // Iterate abilities on attacker
                for (const ability of ABILITIES) {
                    if (ability) runCase(attackerName, defenderName, moveName, { attackerAbility: ability });
                }
            }
        }
    }

    // Write output
    const output = {
        meta: {
            source: 'synthetic',
            generatedAt: new Date().toISOString(),
            caseCount: cases.length,
        },
        cases,
    };

    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, 'synthetic_damage.json');
    fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
    console.log(`Generated ${cases.length} synthetic cases at ${outPath}`);
}

function runCase(
    attackerName: string,
    defenderName: string,
    moveName: string,
    opts: { attackerItem?: string, attackerAbility?: string } = {}
) {
    const attacker = new Pokemon(gen, attackerName, {
        item: opts.attackerItem,
        ability: opts.attackerAbility,
        level: 100,
        // Standard competitive spread
        ivs: { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 },
        evs: { hp: 0, atk: 252, def: 0, spa: 252, spd: 4, spe: 252 },
    });

    const defender = new Pokemon(gen, defenderName, {
        level: 100,
        ivs: { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 },
        evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 },
    });

    const move = new Move(gen, moveName);

    const result = calculate(gen, attacker, defender, move);

    cases.push({
        id: `synth-${cases.length}`,
        gen: 9,
        testName: `Synthetic: ${attackerName} vs ${defenderName} using ${moveName}`,
        attacker: {
            name: attackerName,
            level: attacker.level,
            item: attacker.item,
            ability: attacker.ability,
            evs: attacker.evs,
            ivs: attacker.ivs,
            nature: attacker.nature,
            weightkg: attacker.weightkg, // Pokemon object has this populated
        },
        defender: {
            name: defenderName,
            level: defender.level,
            item: defender.item,
            ability: defender.ability,
            evs: defender.evs,
            ivs: defender.ivs,
            nature: defender.nature,
            weightkg: defender.weightkg,
        },
        move: {
            name: moveName,
            isCrit: false,
        },
        expected: {
            damage: result.damage,
            desc: result.desc(),
        },
        strict: true, // Synthetic tests should match exactly as they are simple
    });
}

// Run
generate();
