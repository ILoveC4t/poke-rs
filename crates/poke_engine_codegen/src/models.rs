//! JSON deserialization structures for Showdown data files.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct NatureData {
    #[allow(dead_code)]
    pub name: String,
    pub plus: Option<String>,
    pub minus: Option<String>,
}

#[derive(Deserialize)]
pub struct TypeChartEntry {
    #[serde(rename = "damageTaken")]
    pub damage_taken: HashMap<String, u8>,
}

#[derive(Deserialize)]
pub struct BaseStats {
    pub hp: u8,
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}

#[derive(Deserialize)]
pub struct PokedexEntry {
    pub num: Option<i16>,
    #[allow(dead_code)]
    pub name: String,
    pub types: Option<Vec<String>>,
    #[serde(rename = "baseStats")]
    pub base_stats: Option<BaseStats>,
    #[serde(default)]
    pub abilities: HashMap<String, String>,
    #[serde(default)]
    pub weightkg: f64,
    #[serde(rename = "baseSpecies")]
    pub base_species: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(rename = "genderRatio")]
    pub gender_ratio: Option<HashMap<String, f64>>,
    #[serde(rename = "otherFormes")]
    pub other_formes: Option<Vec<String>>,
    pub evos: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct AbilityData {
    #[allow(dead_code)]
    pub name: String,
    pub num: i16,
}

#[derive(Deserialize)]
pub struct MoveData {
    #[allow(dead_code)]
    pub name: String,
    pub num: i16,
    #[serde(rename = "basePower")]
    pub base_power: Option<u16>,
    pub accuracy: Option<serde_json::Value>, // Can be bool (true = always hits) or number
    pub pp: Option<u8>,
    pub priority: Option<i8>,
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub move_type: Option<String>,
    #[serde(default)]
    pub flags: HashMap<String, u8>,
    pub terrain: Option<String>,

    // Recoil fields for Reckless ability
    pub recoil: Option<serde_json::Value>,
    #[serde(rename = "hasCrashDamage")]
    pub has_crash_damage: Option<bool>,
    #[serde(rename = "mindBlownRecoil")]
    pub mind_blown_recoil: Option<bool>,
    #[allow(dead_code)]
    #[serde(rename = "struggleRecoil")]
    pub struggle_recoil: Option<bool>,

    // Fields for Sheer Force
    pub secondary: Option<serde_json::Value>,
    pub secondaries: Option<serde_json::Value>,
    #[serde(rename = "hasSheerForce")]
    pub has_sheer_force: Option<bool>,
}

#[derive(Deserialize)]
pub struct Fling {
    #[serde(rename = "basePower")]
    pub base_power: u8,
}

#[derive(Deserialize)]
pub struct ItemData {
    pub name: String,
    pub num: Option<i16>,
    #[allow(dead_code)]
    #[serde(default)]
    pub gen: u8,
    #[serde(rename = "isNonstandard")]
    pub _is_nonstandard: Option<String>,
    pub fling: Option<Fling>,
    #[serde(default)]
    #[serde(rename = "megaStone")]
    pub mega_stone: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "zMove")]
    pub z_move: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "onPlate")]
    pub on_plate: Option<String>,
    #[serde(default)]
    #[serde(rename = "onMemory")]
    pub on_memory: Option<String>,
    #[serde(default)]
    #[serde(rename = "onDrive")]
    pub on_drive: Option<String>,
    #[serde(default)]
    #[serde(rename = "forcedForme")]
    pub forced_forme: Option<String>,
}
