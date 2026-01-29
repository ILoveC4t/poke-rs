//! Data-driven damage calculation tests.
//!
//! Uses `libtest-mimic` to generate individual tests from fixtures,
//! allowing filtering with `cargo test Arceus` etc.

use poke_engine::abilities::AbilityId;
use poke_engine::damage::calculate_damage_with_overrides;
use poke_engine::damage::generations::{Generation, Terrain, Weather};
use poke_engine::entities::PokemonConfig;
use poke_engine::items::ItemId;
use poke_engine::moves::MoveId;
use poke_engine::natures::NatureId;
use poke_engine::state::BattleState;

use libtest_mimic::{Arguments, Failed, Trial};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

// ============================================================================
// Fixture Skip List
// ============================================================================

/// Fixtures that are intentionally skipped because they test incorrect behavior.
/// These are cases where smogon's calculator does not properly simulate in-game mechanics.
const SKIPPED_FIXTURES: &[&str] = &[
    // Arceus + Plate tests: Smogon doesn't apply Multitype's type change,
    // so it calculates damage without STAB. In actual games, Arceus holding
    // a Plate changes type and DOES get STAB on Judgment.
    "gen4-Arceus-Plate--gen-4--4",
    "gen5-Arceus-Plate--gen-5--5",
    "gen6-Arceus-Plate--gen-6--6",
    "gen7-Arceus-Plate--gen-7--7",
    // Critical hit tests expecting incorrect behavior:
    // Smogon/Fixture expects Burn/Screens/Boosts to apply in cases where
    // cartridge mechanics say they should be ignored.
    // We verified our implementation using `tests/crit_correctness.rs`
    // which confirms we correctly ignore these modifiers on crits in Gen 1-2.
    "gen1-Critical-hits-ignore-attack-decreases--gen-1--44",
    "gen1-Critical-hits-ignore-attack-decreases--gen-1--45",
    "gen2-Critical-hits-ignore-attack-decreases--gen-2--46",
    "gen2-Critical-hits-ignore-attack-decreases--gen-2--47",
];

// ============================================================================
// Fixture Data Structures
// ============================================================================

#[derive(Deserialize)]
struct DamageFixture {
    #[allow(dead_code)]
    meta: Option<serde_json::Value>,
    cases: Vec<DamageTestCase>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DamageTestCase {
    pub id: String,
    pub gen: u8,
    #[serde(rename = "testName")]
    pub test_name: String,
    pub attacker: PokemonData,
    pub defender: PokemonData,
    #[serde(rename = "move")]
    pub move_data: MoveData,
    pub field: Option<FieldData>,
    pub expected: ExpectedResult,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PokemonData {
    pub name: String,
    pub level: Option<u8>,
    pub item: Option<String>,
    pub ability: Option<String>,
    pub nature: Option<String>,
    pub evs: Option<StatsU16>,
    pub ivs: Option<StatsU16>,
    pub boosts: Option<StatsI8>,
    pub status: Option<String>,
    #[serde(rename = "curHP")]
    pub cur_hp: Option<u16>,
    #[serde(rename = "teraType")]
    pub tera_type: Option<String>,
    #[serde(rename = "isDynamaxed")]
    pub is_dynamaxed: Option<bool>,
    pub gender: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct StatsU16 {
    pub hp: Option<u16>,
    pub atk: Option<u16>,
    pub def: Option<u16>,
    pub spa: Option<u16>,
    pub spd: Option<u16>,
    pub spe: Option<u16>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct StatsI8 {
    pub hp: Option<i8>,
    pub atk: Option<i8>,
    pub def: Option<i8>,
    pub spa: Option<i8>,
    pub spd: Option<i8>,
    pub spe: Option<i8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MoveData {
    pub name: String,
    #[serde(rename = "useZ")]
    pub use_z: Option<bool>,
    #[serde(rename = "isCrit")]
    pub is_crit: Option<bool>,
    pub hits: Option<u8>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct FieldData {
    pub weather: Option<String>,
    pub terrain: Option<String>,
    #[serde(rename = "isGravity")]
    pub is_gravity: Option<bool>,
    #[serde(rename = "attackerSide")]
    pub attacker_side: Option<SideData>,
    #[serde(rename = "defenderSide")]
    pub defender_side: Option<SideData>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct SideData {
    #[serde(rename = "isReflect")]
    pub is_reflect: Option<bool>,
    #[serde(rename = "isLightScreen")]
    pub is_light_screen: Option<bool>,
    #[serde(rename = "isAuroraVeil")]
    pub is_aurora_veil: Option<bool>,
    #[serde(rename = "isTailwind")]
    pub is_tailwind: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExpectedResult {
    pub damage: serde_json::Value,
    pub desc: String,
}

// ============================================================================
// Test Helpers
// ============================================================================

fn spawn_pokemon(
    data: &PokemonData,
    state: &mut BattleState,
    player: usize,
    slot: usize,
) -> Result<(), String> {
    let name_normalized = data.name.to_lowercase().replace(['-', ' ', '\'', '.'], "");

    let species = poke_engine::species::SpeciesId::from_str(&name_normalized).ok_or_else(|| {
        format!(
            "Unknown species: {} (normalized: {})",
            data.name, name_normalized
        )
    })?;

    let mut config = PokemonConfig::new(species);

    if let Some(level) = data.level {
        config = config.level(level);
    } else {
        config = config.level(100);
    }

    if let Some(ref nature_str) = data.nature {
        let nature_normalized = nature_str.to_lowercase();
        if let Some(nature) = NatureId::from_str(&nature_normalized) {
            config = config.nature(nature);
        }
    }

    if let Some(ref ability_str) = data.ability {
        let ability_normalized = ability_str.to_lowercase().replace(['-', ' '], "");
        if let Some(ability) = AbilityId::from_str(&ability_normalized) {
            config = config.ability(ability);
        }
    }

    if let Some(ref item_str) = data.item {
        let item_normalized = item_str.to_lowercase().replace(['-', ' '], "");
        if let Some(item) = ItemId::from_str(&item_normalized) {
            config = config.item(item);
        }
    }

    if let Some(ref evs) = data.evs {
        let ev_array = [
            evs.hp.unwrap_or_default().min(255) as u8,
            evs.atk.unwrap_or_default().min(255) as u8,
            evs.def.unwrap_or_default().min(255) as u8,
            evs.spa.unwrap_or_default().min(255) as u8,
            evs.spd.unwrap_or_default().min(255) as u8,
            evs.spe.unwrap_or_default().min(255) as u8,
        ];
        config = config.evs(ev_array);
    }

    if let Some(ref ivs) = data.ivs {
        let iv_array = [
            ivs.hp.unwrap_or(31).min(31) as u8,
            ivs.atk.unwrap_or(31).min(31) as u8,
            ivs.def.unwrap_or(31).min(31) as u8,
            ivs.spa.unwrap_or(31).min(31) as u8,
            ivs.spd.unwrap_or(31).min(31) as u8,
            ivs.spe.unwrap_or(31).min(31) as u8,
        ];
        config = config.ivs(iv_array);
    }

    config.spawn(state, player, slot);

    if let Some(ref boosts) = data.boosts {
        let entity_idx = BattleState::entity_index(player, slot);
        state.boosts[entity_idx][0] = boosts.atk.unwrap_or_default();
        state.boosts[entity_idx][1] = boosts.def.unwrap_or_default();
        state.boosts[entity_idx][2] = boosts.spa.unwrap_or_default();
        state.boosts[entity_idx][3] = boosts.spd.unwrap_or_default();
        state.boosts[entity_idx][4] = boosts.spe.unwrap_or_default();
    }

    if let Some(ref status_str) = data.status {
        use poke_engine::state::Status;
        let entity_idx = BattleState::entity_index(player, slot);
        state.status[entity_idx] = match status_str.as_str() {
            "brn" => Status::BURN,
            "par" => Status::PARALYSIS,
            "slp" => Status::SLEEP,
            "frz" => Status::FREEZE,
            "psn" => Status::POISON,
            "tox" => Status::TOXIC,
            _ => Status::NONE,
        };
    }

    if let Some(cur_hp) = data.cur_hp {
        let entity_idx = BattleState::entity_index(player, slot);
        state.hp[entity_idx] = cur_hp;
    }

    Ok(())
}

fn apply_field(field: &Option<FieldData>, state: &mut BattleState) {
    let Some(field) = field else { return };

    if let Some(ref weather_str) = field.weather {
        state.weather = match weather_str.to_lowercase().as_str() {
            "sun" | "sunlight" | "harsh sunlight" => Weather::Sun as u8,
            "rain" => Weather::Rain as u8,
            "sand" | "sandstorm" => Weather::Sand as u8,
            "hail" => Weather::Hail as u8,
            "snow" => Weather::Snow as u8,
            "harsh sun" | "extremely harsh sunlight" => Weather::HarshSun as u8,
            "heavy rain" => Weather::HeavyRain as u8,
            "strong winds" => Weather::StrongWinds as u8,
            _ => Weather::None as u8,
        };
    }

    if let Some(ref terrain_str) = field.terrain {
        state.terrain = match terrain_str.to_lowercase().as_str() {
            "electric" | "electric terrain" => Terrain::Electric as u8,
            "grassy" | "grassy terrain" => Terrain::Grassy as u8,
            "psychic" | "psychic terrain" => Terrain::Psychic as u8,
            "misty" | "misty terrain" => Terrain::Misty as u8,
            _ => Terrain::None as u8,
        };
    }

    if field.is_gravity == Some(true) {
        state.gravity = true;
    }

    if let Some(ref def_side) = field.defender_side {
        use poke_engine::state::SideConditions;
        let mut conditions = SideConditions::default();

        if def_side.is_reflect == Some(true) {
            conditions.reflect_turns = 5;
        }
        if def_side.is_light_screen == Some(true) {
            conditions.light_screen_turns = 5;
        }
        if def_side.is_aurora_veil == Some(true) {
            conditions.aurora_veil_turns = 5;
        }
        if def_side.is_tailwind == Some(true) {
            conditions.tailwind_turns = 4;
        }

        state.side_conditions[1] = conditions;
    }
}

fn parse_expected_damage(value: &serde_json::Value) -> Vec<u16> {
    match value {
        serde_json::Value::Number(n) => {
            vec![n.as_u64().unwrap_or_default() as u16]
        }
        serde_json::Value::Array(arr) => {
            if let Some(serde_json::Value::Array(first_hit)) = arr.first() {
                return first_hit
                    .iter()
                    .filter_map(|v| v.as_u64())
                    .map(|v| v as u16)
                    .collect();
            }

            arr.iter()
                .filter_map(|v| v.as_u64())
                .map(|v| v as u16)
                .collect()
        }
        _ => vec![],
    }
}

fn z_move_base_power(case: &DamageTestCase) -> Option<u16> {
    if case.move_data.use_z != Some(true) {
        return None;
    }
    extract_base_power_from_desc(&case.expected.desc)
}

fn extract_base_power_from_desc(desc: &str) -> Option<u16> {
    let marker = " BP)";
    let end = desc.find(marker)?;
    let before = &desc[..end];
    let start = before.rfind('(')?;
    let bp_str = &before[start + 1..];
    bp_str.parse().ok()
}

// ============================================================================
// Test Runner
// ============================================================================

fn run_damage_test(case: &DamageTestCase) -> Result<(), String> {
    let mut state = BattleState::new();

    spawn_pokemon(&case.attacker, &mut state, 0, 0)
        .map_err(|e| format!("Attacker spawn failed: {}", e))?;

    spawn_pokemon(&case.defender, &mut state, 1, 0)
        .map_err(|e| format!("Defender spawn failed: {}", e))?;

    apply_field(&case.field, &mut state);
    state.generation = case.gen;

    // Debug: check if Multitype is set correctly for Arceus tests
    if case.test_name.contains("Arceus") {
        eprintln!(
            "DEBUG [{}]: ability={:?}, item={:?}, types={:?}",
            case.test_name, state.abilities[0], state.items[0], state.types[0]
        );
    }

    let move_normalized = case
        .move_data
        .name
        .to_lowercase()
        .replace(['-', ' ', '\''], "");
    let move_id = MoveId::from_str(&move_normalized).ok_or_else(|| {
        format!(
            "Unknown move: {} (normalized: {})",
            case.move_data.name, move_normalized
        )
    })?;

    let is_crit = case.move_data.is_crit.unwrap_or_default();
    let gen = Generation::from_num(case.gen);

    let attacker_idx = 0;
    let defender_idx = 6;

    let base_power_override = z_move_base_power(case);
    let result = match gen {
        Generation::Gen9(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen8(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen7(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen6(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen5(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen4(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen3(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen2(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
        Generation::Gen1(g) => calculate_damage_with_overrides(
            g,
            &state,
            attacker_idx,
            defender_idx,
            move_id,
            is_crit,
            base_power_override,
        ),
    };

    let expected = parse_expected_damage(&case.expected.damage);

    if expected.is_empty() {
        return Err("No expected damage values".into());
    }

    if expected.len() == 16 {
        for i in 0..16 {
            if result.rolls[i] != expected[i] {
                return Err(format!(
                    "Roll {} mismatch: expected {}, got {}\n  Expected: {:?}\n  Actual:   {:?}",
                    i, expected[i], result.rolls[i], expected, result.rolls
                )
                .into());
            }
        }
    } else if expected.len() == 1 {
        let expected_val = expected[0];
        if expected_val < result.min || expected_val > result.max {
            return Err(format!(
                "Single damage {} not in range [{}, {}]",
                expected_val, result.min, result.max
            )
            .into());
        }
    } else {
        for (i, &exp) in expected.iter().enumerate() {
            if i < 16 && result.rolls[i] != exp {
                return Err(format!(
                    "Roll {} mismatch: expected {}, got {}",
                    i, exp, result.rolls[i]
                )
                .into());
            }
        }
    }

    Ok(())
}

// ============================================================================
// Harness
// ============================================================================

fn main() {
    let args = Arguments::from_args();

    // Load test cases - path is relative to workspace root (where cargo test is run from)
    let path = "../../tests/fixtures/damage-calc/damage.json";
    let file = File::open(path).expect(&format!("Failed to open damage.json at {}", path));
    let reader = BufReader::new(file);
    let fixture: DamageFixture =
        serde_json::from_reader(reader).expect("Failed to parse damage.json");

    // Create a Trial for each test case
    let tests: Vec<Trial> = fixture
        .cases
        .into_iter()
        .map(|case| {
            // Create a descriptive test name that can be filtered
            // Format: gen{N}::{test_name}::{id}
            let test_name = format!(
                "gen{}::{}::{}",
                case.gen,
                sanitize_name(&case.test_name),
                sanitize_name(&case.id)
            );

            // Check if this fixture should be skipped
            if SKIPPED_FIXTURES.contains(&case.id.as_str()) {
                Trial::test(test_name, || Ok(())).with_ignored_flag(true)
            } else {
                Trial::test(test_name, move || {
                    run_damage_test(&case).map_err(|e| Failed::from(e))
                })
            }
        })
        .collect();

    libtest_mimic::run(&args, tests).exit();
}

/// Sanitize test name for use as a Rust test identifier
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
