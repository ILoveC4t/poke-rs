// Visual diff tool for damage calculation tests
// Usage: npx tsx scripts/diff_damage.mts [path/to/fixture.json]

import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

// Colors using ANSI codes
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const GRAY = '\x1b[90m';
const RESET = '\x1b[0m';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const fixturePath = process.argv[2] || path.resolve(repoRoot, 'tests/fixtures/damage-calc/damage.json');

interface Fixture {
    meta: any;
    cases: any[];
}

interface Output {
    rolls: number[];
    min: number;
    max: number;
}

async function main() {
    if (!fs.existsSync(fixturePath)) {
        console.error(`Fixture file not found: ${fixturePath}`);
        console.error('Usage: npx tsx scripts/diff_damage.mts [path/to/fixture.json]');
        process.exit(1);
    }

    console.log(`Loading fixtures from ${fixturePath}...`);
    const fixture: Fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf-8'));
    const cases = fixture.cases;

    console.log(`Compiling and running fixture_runner...`);

    // Spawn Rust process
    const rustProcess = spawn('cargo', ['run', '-q', '--bin', 'fixture_runner'], {
        cwd: repoRoot,
        stdio: ['pipe', 'pipe', 'inherit'], // Pipe stdin/stdout, inherit stderr
    });

    const results: any[] = [];
    let passed = 0;
    let failed = 0;

    // Read stdout line by line
    let buffer = '';
    rustProcess.stdout.on('data', (data) => {
        buffer += data.toString();
        const lines = buffer.split('\n');
        // Handle the last potentially incomplete line
        if (!buffer.endsWith('\n')) {
             buffer = lines.pop() || '';
        } else {
             buffer = '';
        }

        for (const line of lines) {
            if (line.trim()) {
                try {
                    const output: Output = JSON.parse(line);
                    results.push(output);
                } catch (e) {
                    console.error('Error parsing Rust output:', line);
                }
            }
        }
    });

    // Send inputs
    // We filter cases to only send those that match the fixture format expected by Rust runner
    // (though ideally all should match)
    for (const c of cases) {
        // Ensure minimal fields are present if scraper missed them (e.g. strict)
        if (c.strict === undefined) c.strict = false;
        rustProcess.stdin.write(JSON.stringify(c) + '\n');
    }
    rustProcess.stdin.end();

    // Wait for process to finish
    await new Promise<void>((resolve) => {
        rustProcess.on('close', resolve);
    });

    console.log(`Comparing ${cases.length} cases...`);

    if (results.length !== cases.length) {
        console.error(`Mismatch in result count! Sent ${cases.length}, got ${results.length}`);
        // This might happen if Rust panicked or printed extra debug info
    }

    for (let i = 0; i < Math.min(cases.length, results.length); i++) {
        const c = cases[i];
        const r = results[i];

        let pass = false;

        // Handle expected format
        let expectedMin = 0;
        let expectedMax = 0;

        if (Array.isArray(c.expected.damage)) {
            if (c.expected.damage.length === 16 && typeof c.expected.damage[0] === 'number') {
                // Full rolls
                 pass = arraysMatch(c.expected.damage, r.rolls, c.strict);
                 expectedMin = Math.min(...c.expected.damage);
                 expectedMax = Math.max(...c.expected.damage);
            } else if (c.expected.damage.length === 2 && typeof c.expected.damage[0] === 'number') {
                 // Min/Max pair? or just 2 rolls?
                 // Some moves deal fixed damage 2 times?
                 // Assuming [min, max] range notation sometimes used in tests?
                 // Actually smogon calc returns 16 rolls.
                 // If it returns [number, number], it might be child damage?
                 // Let's assume for now 16 rolls is standard.
                 pass = arraysMatch(c.expected.damage, r.rolls.slice(0, c.expected.damage.length), c.strict);
            } else {
                 // Empty array or other structure
                 if (c.expected.damage.length === 0 && r.max === 0) pass = true;
            }
        } else if (typeof c.expected.damage === 'number') {
             // Exact value (fixed damage or 0)
             pass = almostEqual(c.expected.damage, r.min, c.strict) && almostEqual(c.expected.damage, r.max, c.strict);
             expectedMin = c.expected.damage;
             expectedMax = c.expected.damage;
        }

        if (pass) {
            passed++;
        } else {
            failed++;
            console.log(`${RED}✗ ${c.testName || c.id}${RESET}`);
            console.log(`  ${GRAY}Expected:${RESET} ${formatExpected(c.expected.damage)}`);
            console.log(`  ${YELLOW}Actual:  ${RESET} ${formatRolls(r.rolls)} (Min: ${r.min}, Max: ${r.max})`);

            // Diff visualization
             if (Array.isArray(c.expected.damage) && c.expected.damage.length === 16 && typeof c.expected.damage[0] === 'number') {
                 visualizeDiff(c.expected.damage, r.rolls);
             }
        }
    }

    console.log(`\n${passed}/${cases.length} passed (${Math.round(passed/cases.length*100)}%)`);
    // Exit code 1 if any failed, so CI can fail
    if (failed > 0) process.exit(1);
}

function arraysMatch(a: number[], b: number[], strict: boolean): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (!almostEqual(a[i], b[i], strict)) return false;
    }
    return true;
}

function almostEqual(a: number, b: number, strict: boolean): boolean {
    if (strict) return a === b;
    return Math.abs(a - b) <= 1; // Tolerance of 1
}

function formatExpected(exp: any): string {
    if (Array.isArray(exp)) return `[${exp.join(', ')}]`;
    return `${exp}`;
}

function formatRolls(rolls: number[]): string {
    return `[${rolls.join(', ')}]`;
}

function visualizeDiff(expected: number[], actual: number[]) {
    const diffs = expected.map((e, i) => {
        const a = actual[i];
        const diff = a - e;
        if (diff === 0) return `${GRAY}${e}${RESET}`;
        if (diff > 0) return `${RED}${e}→${a}(+${diff})${RESET}`;
        return `${BLUE}${e}→${a}(${diff})${RESET}`;
    });
    console.log(`  Diff:     [${diffs.join(', ')}]`);
}

main().catch(console.error);
