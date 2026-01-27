use std::io::{self, BufRead};
use serde::{Deserialize, Serialize};
use poke_engine::state::{BattleState, Status};
use poke_engine::entities::PokemonConfig;
use poke_engine::damage::{calculate_damage, DamageResult, Generation};
use poke_engine::moves::MoveId;
use poke_engine::items::ItemId;
use poke_engine::abilities::AbilityId;
use poke_engine::natures::NatureId;
use poke_engine::terrains::TerrainId;

#[derive(Serialize)]
struct Output {
    rolls: [u16; 16],
    min: u16,
    max: u16,
}

impl From<DamageResult> for Output {
    fn from(res: DamageResult) -> Self {
        Self {
            rolls: res.rolls,
            min: res.min,
            max: res.max,
        }
    }
}

#[derive(Deserialize)]
struct MoveData {
    name: String,
    #[serde(default, rename = "isCrit")]
    is_crit: bool,
    #[serde(default, rename = "useZ")]
    _use_z: bool,
    #[serde(default)]
    _hits: u8,
}

#[derive(Deserialize, Default)]
struct FieldData {
    #[serde(default)]
    weather: Option<String>,
    #[serde(default)]
    terrain: Option<String>,
    #[serde(default, rename = "isGravity")]
    is_gravity: bool,
    #[serde(default, rename = "isMagicRoom")]
    _is_magic_room: bool,
    #[serde(default, rename = "isWonderRoom")]
    _is_wonder_room: bool,
}

#[derive(Deserialize, Default, Clone, Copy)]
struct Boosts {
    #[serde(default)]
    atk: i8,
    #[serde(default)]
    def: i8,
    #[serde(default)]
    spa: i8,
    #[serde(default)]
    spd: i8,
    #[serde(default)]
    spe: i8,
}

#[derive(Deserialize, Default, Clone, Copy)]
struct StatsOptions {
    hp: Option<u8>,
    atk: Option<u8>,
    def: Option<u8>,
    spa: Option<u8>,
    spd: Option<u8>,
    spe: Option<u8>,
}

fn resolve_ivs(stats: Option<StatsOptions>) -> [u8; 6] {
    let mut ivs = [31; 6];
    if let Some(s) = stats {
        if let Some(v) = s.hp { ivs[0] = v; }
        if let Some(v) = s.atk { ivs[1] = v; }
        if let Some(v) = s.def { ivs[2] = v; }
        if let Some(v) = s.spa { ivs[3] = v; }
        if let Some(v) = s.spd { ivs[4] = v; }
        if let Some(v) = s.spe { ivs[5] = v; }
    }
    ivs
}

fn resolve_evs(stats: Option<StatsOptions>) -> [u8; 6] {
    let mut evs = [0; 6];
    if let Some(s) = stats {
        if let Some(v) = s.hp { evs[0] = v; }
        if let Some(v) = s.atk { evs[1] = v; }
        if let Some(v) = s.def { evs[2] = v; }
        if let Some(v) = s.spa { evs[3] = v; }
        if let Some(v) = s.spd { evs[4] = v; }
        if let Some(v) = s.spe { evs[5] = v; }
    }
    evs
}

#[derive(Deserialize)]
struct Fixture {
    #[serde(default)]
    gen: u8,
    attacker: PokemonData,
    defender: PokemonData,
    #[serde(rename = "move")]
    move_data: MoveData,
    #[serde(default)]
    field: FieldData,
}

#[derive(Deserialize)]
struct PokemonData {
    name: String,
    #[serde(default)]
    level: Option<u8>,
    #[serde(default)]
    item: Option<String>,
    #[serde(default)]
    ability: Option<String>,
    #[serde(default)]
    nature: Option<String>,
    #[serde(default)]
    evs: Option<StatsOptions>,
    #[serde(default)]
    ivs: Option<StatsOptions>,
    #[serde(default)]
    boosts: Option<Boosts>,
    #[serde(default, rename = "curHP")]
    cur_hp: Option<u16>,
    #[serde(rename = "current_hp")]
    current_hp_alt: Option<u16>,
    #[serde(default)]
    status: Option<String>,
}

fn setup_pokemon(state: &mut BattleState, player: usize, data: &PokemonData) {
    // Basic config
    let mut config = PokemonConfig::from_str(&data.name.to_lowercase())
        .unwrap_or_else(|| {
             // Fallback to Pikachu if unknown (shouldn't happen in valid tests)
             PokemonConfig::from_str("pikachu").unwrap()
        });

    // Level
    if let Some(lvl) = data.level {
        config = config.level(lvl);
    } else {
        config = config.level(100);
    }

    // IVs
    config = config.ivs(resolve_ivs(data.ivs));

    // EVs
    config = config.evs(resolve_evs(data.evs));

    // Nature
    if let Some(nature_name) = &data.nature {
        if let Some(nature) = NatureId::from_str(&nature_name.to_lowercase()) {
            config = config.nature(nature);
        }
    }

    // Item
    if let Some(item_name) = &data.item {
        let clean_name = item_name.to_lowercase().replace(" ", "").replace("-", "");
        if let Some(item) = ItemId::from_str(&clean_name) {
            config = config.item(item);
        }
    }

    // Ability
    if let Some(ability_name) = &data.ability {
        let clean_name = ability_name.to_lowercase().replace(" ", "").replace("-", "");
        if let Some(ability) = AbilityId::from_str(&clean_name) {
            config = config.ability(ability);
        }
    }

    // Spawn
    config.spawn(state, player, 0);
    let index = BattleState::entity_index(player, 0);

    // Post-spawn overrides

    // Current HP
    if let Some(hp) = data.cur_hp.or(data.current_hp_alt) {
        state.hp[index] = hp;
    }

    // Boosts
    if let Some(boosts) = data.boosts {
        state.boosts[index][0] = boosts.atk;
        state.boosts[index][1] = boosts.def;
        state.boosts[index][2] = boosts.spa;
        state.boosts[index][3] = boosts.spd;
        state.boosts[index][4] = boosts.spe;
    }

    // Status
    if let Some(status) = &data.status {
        state.status[index] = match status.to_lowercase().as_str() {
            "brn" => Status::BURN,
            "par" => Status::PARALYSIS,
            "psn" => Status::POISON,
            "tox" => Status::TOXIC,
            "slp" => Status::SLEEP,
            "frz" => Status::FREEZE,
            _ => Status::NONE,
        };
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() { continue; }

        let fixture: Fixture = match serde_json::from_str(&line) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error parsing fixture: {}", e);
                continue;
            }
        };

        let mut state = BattleState::new();

        // Setup Attacker (Player 0)
        setup_pokemon(&mut state, 0, &fixture.attacker);

        // Setup Defender (Player 1)
        setup_pokemon(&mut state, 1, &fixture.defender);

        // Parse Move
        let move_clean = fixture.move_data.name.to_lowercase().replace(" ", "").replace("-", "");
        let move_id = MoveId::from_str(&move_clean).unwrap_or(MoveId::default());

        // Field Settings
        if let Some(weather) = &fixture.field.weather {
            state.weather = match weather.as_str() {
                "Sun" | "SunnyDay" => 1,
                "Rain" | "RainDance" => 2,
                "Sand" | "Sandstorm" => 3,
                "Hail" => 4,
                "Snow" => 5,
                "Harsh Sun" | "DesolateLand" => 6,
                "Heavy Rain" | "PrimordialSea" => 7,
                "Strong Winds" | "DeltaStream" => 8,
                _ => 0,
            };
        }

        if let Some(terrain) = &fixture.field.terrain {
            state.terrain = match terrain.as_str() {
                "Electric" | "Electric Terrain" => TerrainId::Electric as u8,
                "Grassy" | "Grassy Terrain" => TerrainId::Grassy as u8,
                "Psychic" | "Psychic Terrain" => TerrainId::Psychic as u8,
                "Misty" | "Misty Terrain" => TerrainId::Misty as u8,
                _ => 0,
            };
        }

        if fixture.field.is_gravity {
            state.gravity = true;
        }

        // Debug output
        eprintln!("DEBUG: Attacker types: {:?}", state.types[0]);
        eprintln!("DEBUG: Attacker ability: {:?}", state.abilities[0]);
        eprintln!("DEBUG: Attacker item: {:?}", state.items[0]);
        eprintln!("DEBUG: Defender types: {:?}", state.types[6]);
        
        // Calculate
        let gen = Generation::from_num(fixture.gen);
        let result = calculate_damage(
            gen,
            &state,
            0, // attacker index
            6, // defender index (player 1, slot 0)
            move_id,
            fixture.move_data.is_crit,
        );

        eprintln!("DEBUG: Final BP: {}", result.final_base_power);
        eprintln!("DEBUG: Effectiveness: {}", result.effectiveness);

        let output = Output::from(result);
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}
