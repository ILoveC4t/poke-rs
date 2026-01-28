//! Fixture data structures for damage calculation tests.
//!
//! These types are deserialized from `tests/fixtures/damage-calc/damage.json`.

use serde::Deserialize;

/// Root structure for the damage fixture file.
#[derive(Deserialize)]
pub struct DamageFixture {
    #[allow(dead_code)]
    pub meta: Option<serde_json::Value>,
    pub cases: Vec<DamageTestCase>,
}

/// A single damage calculation test case.
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

/// Pokemon configuration from fixture.
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

/// Stats with u16 values (for EVs/IVs).
#[derive(Deserialize, Debug, Default, Clone)]
pub struct StatsU16 {
    pub hp: Option<u16>,
    pub atk: Option<u16>,
    pub def: Option<u16>,
    pub spa: Option<u16>,
    pub spd: Option<u16>,
    pub spe: Option<u16>,
}

/// Stats with i8 values (for boosts).
#[derive(Deserialize, Debug, Default, Clone)]
pub struct StatsI8 {
    pub hp: Option<i8>,
    pub atk: Option<i8>,
    pub def: Option<i8>,
    pub spa: Option<i8>,
    pub spd: Option<i8>,
    pub spe: Option<i8>,
}

/// Move configuration from fixture.
#[derive(Deserialize, Debug, Clone)]
pub struct MoveData {
    pub name: String,
    #[serde(rename = "useZ")]
    pub use_z: Option<bool>,
    #[serde(rename = "isCrit")]
    pub is_crit: Option<bool>,
    pub hits: Option<u8>,
}

/// Field conditions from fixture.
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

/// Side conditions (screens, tailwind, etc.).
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

/// Expected result from fixture.
#[derive(Deserialize, Debug, Clone)]
pub struct ExpectedResult {
    pub damage: serde_json::Value,
    pub desc: String,
}

impl DamageFixture {
    /// Load fixture from the standard path.
    pub fn load() -> Result<Self, String> {
        Self::load_from("../../tests/fixtures/damage-calc/damage.json")
    }

    /// Load fixture from a custom path.
    pub fn load_from(path: &str) -> Result<Self, String> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| format!("Failed to parse {}: {}", path, e))
    }
}
