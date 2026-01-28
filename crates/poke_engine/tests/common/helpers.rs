//! Test helper functions for damage calculation tests.
//!
//! These helpers convert fixture data into engine state and verify results.

use poke_engine::abilities::AbilityId;
use poke_engine::damage::calculate_damage_with_overrides;
use poke_engine::damage::generations::{Generation, Terrain, Weather};
use poke_engine::entities::PokemonConfig;
use poke_engine::items::ItemId;
use poke_engine::moves::MoveId;
use poke_engine::natures::NatureId;
use poke_engine::state::BattleState;

use super::fixtures::{DamageTestCase, FieldData, PokemonData};

/// Spawn a Pokemon from fixture data into the battle state.
pub fn spawn_pokemon(
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

    // Level (default 100)
    config = config.level(data.level.unwrap_or(100));

    // Nature
    if let Some(ref nature_str) = data.nature {
        let nature_normalized = nature_str.to_lowercase();
        if let Some(nature) = NatureId::from_str(&nature_normalized) {
            config = config.nature(nature);
        }
    }

    // Ability
    if let Some(ref ability_str) = data.ability {
        let ability_normalized = ability_str.to_lowercase().replace(['-', ' '], "");
        if let Some(ability) = AbilityId::from_str(&ability_normalized) {
            config = config.ability(ability);
        }
    }

    // Item
    if let Some(ref item_str) = data.item {
        let item_normalized = item_str.to_lowercase().replace(['-', ' '], "");
        if let Some(item) = ItemId::from_str(&item_normalized) {
            config = config.item(item);
        }
    }

    // EVs
    if let Some(ref evs) = data.evs {
        let ev_array = [
            evs.hp.unwrap_or(0).min(255) as u8,
            evs.atk.unwrap_or(0).min(255) as u8,
            evs.def.unwrap_or(0).min(255) as u8,
            evs.spa.unwrap_or(0).min(255) as u8,
            evs.spd.unwrap_or(0).min(255) as u8,
            evs.spe.unwrap_or(0).min(255) as u8,
        ];
        config = config.evs(ev_array);
    }

    // IVs
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

    // Spawn
    config.spawn(state, player, slot);

    // Apply boosts after spawning
    if let Some(ref boosts) = data.boosts {
        let entity_idx = BattleState::entity_index(player, slot);
        state.boosts[entity_idx][0] = boosts.atk.unwrap_or(0);
        state.boosts[entity_idx][1] = boosts.def.unwrap_or(0);
        state.boosts[entity_idx][2] = boosts.spa.unwrap_or(0);
        state.boosts[entity_idx][3] = boosts.spd.unwrap_or(0);
        state.boosts[entity_idx][4] = boosts.spe.unwrap_or(0);
    }

    // Apply status
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

    // Apply current HP
    if let Some(cur_hp) = data.cur_hp {
        let entity_idx = BattleState::entity_index(player, slot);
        state.hp[entity_idx] = cur_hp;
    }

    Ok(())
}

/// Apply field conditions to the battle state.
pub fn apply_field(field: &Option<FieldData>, state: &mut BattleState) {
    let Some(field) = field else { return };

    // Weather
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

    // Terrain
    if let Some(ref terrain_str) = field.terrain {
        state.terrain = match terrain_str.to_lowercase().as_str() {
            "electric" | "electric terrain" => Terrain::Electric as u8,
            "grassy" | "grassy terrain" => Terrain::Grassy as u8,
            "psychic" | "psychic terrain" => Terrain::Psychic as u8,
            "misty" | "misty terrain" => Terrain::Misty as u8,
            _ => Terrain::None as u8,
        };
    }

    // Gravity
    if field.is_gravity == Some(true) {
        state.gravity = true;
    }

    // Defender side conditions (screens)
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

    // Attacker side conditions
    if let Some(ref atk_side) = field.attacker_side {
        use poke_engine::state::SideConditions;
        let mut conditions = SideConditions::default();

        if atk_side.is_tailwind == Some(true) {
            conditions.tailwind_turns = 4;
        }

        state.side_conditions[0] = conditions;
    }
}

/// Parse expected damage values from fixture JSON.
pub fn parse_expected_damage(value: &serde_json::Value) -> Vec<u16> {
    match value {
        serde_json::Value::Number(n) => {
            vec![n.as_u64().unwrap_or(0) as u16]
        }
        serde_json::Value::Array(arr) => {
            // Multi-hit fixtures store arrays per hit; use first hit
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

/// Extract Z-move base power from fixture description.
pub fn z_move_base_power(case: &DamageTestCase) -> Option<u16> {
    if case.move_data.use_z != Some(true) {
        return None;
    }
    extract_base_power_from_desc(&case.expected.desc)
}

/// Extract base power from description string (e.g., "(120 BP)").
fn extract_base_power_from_desc(desc: &str) -> Option<u16> {
    let marker = " BP)";
    let end = desc.find(marker)?;
    let before = &desc[..end];
    let start = before.rfind('(')?;
    let bp_str = &before[start + 1..];
    bp_str.parse().ok()
}

/// Run a damage calculation test and verify results.
pub fn run_damage_test(case: &DamageTestCase) -> Result<(), String> {
    let mut state = BattleState::new();

    spawn_pokemon(&case.attacker, &mut state, 0, 0)
        .map_err(|e| format!("Attacker spawn failed: {}", e))?;

    spawn_pokemon(&case.defender, &mut state, 1, 0)
        .map_err(|e| format!("Defender spawn failed: {}", e))?;

    apply_field(&case.field, &mut state);
    state.generation = case.gen;

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

    let is_crit = case.move_data.is_crit.unwrap_or(false);
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

    // Compare results
    if expected.len() == 16 {
        for i in 0..16 {
            if result.rolls[i] != expected[i] {
                return Err(format!(
                    "Roll {} mismatch: expected {}, got {}",
                    i, expected[i], result.rolls[i]
                ));
            }
        }
    } else if expected.len() == 1 {
        let expected_val = expected[0];
        if expected_val < result.min || expected_val > result.max {
            return Err(format!(
                "Single damage {} not in range [{}, {}]",
                expected_val, result.min, result.max
            ));
        }
    } else {
        for (i, &exp) in expected.iter().enumerate() {
            if i < 16 && result.rolls[i] != exp {
                return Err(format!(
                    "Roll {} mismatch: expected {}, got {}",
                    i, exp, result.rolls[i]
                ));
            }
        }
    }

    Ok(())
}

/// Sanitize test name for use as a Rust test identifier.
pub fn sanitize_name(name: &str) -> String {
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
