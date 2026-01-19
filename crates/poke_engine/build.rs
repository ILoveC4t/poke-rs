//! Build script that parses JSON data files from smogon/pokemon-showdown
//! and generates optimized Rust types for the battle engine.

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a key to a valid Rust identifier in PascalCase.
/// Handles keys starting with digits by prefixing with underscore.
fn to_valid_ident(key: &str) -> String {
    let pascal = key.to_pascal_case();
    if pascal.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", pascal)
    } else {
        pascal
    }
}

/// Check if a move has secondary effects for Sheer Force boost criteria.
/// Returns true if the move has secondary or secondaries fields (that are not null),
/// or if it has the explicit has_sheer_force flag set.
fn has_secondary_effects(data: &MoveData) -> bool {
    data.secondary.as_ref().map_or(false, |v| !v.is_null())
        || data.secondaries.as_ref().map_or(false, |v| !v.is_null())
        || data.has_sheer_force.unwrap_or(false)
}

// ============================================================================
// JSON Deserialization Structures
// ============================================================================

#[derive(Deserialize)]
struct NatureData {
    #[allow(dead_code)]
    name: String,
    plus: Option<String>,
    minus: Option<String>,
}

#[derive(Deserialize)]
struct TypeChartEntry {
    #[serde(rename = "damageTaken")]
    damage_taken: HashMap<String, u8>,
}

#[derive(Deserialize)]
struct BaseStats {
    hp: u8,
    atk: u8,
    def: u8,
    spa: u8,
    spd: u8,
    spe: u8,
}

#[derive(Deserialize)]
struct PokedexEntry {
    num: Option<i16>,
    #[allow(dead_code)]
    name: String,
    types: Option<Vec<String>>,
    #[serde(rename = "baseStats")]
    base_stats: Option<BaseStats>,
    #[serde(default)]
    abilities: HashMap<String, String>,
    #[serde(default)]
    weightkg: f64,
    #[serde(rename = "baseSpecies")]
    base_species: Option<String>,
    #[serde(default)]
    gender: Option<String>,
    #[serde(rename = "genderRatio")]
    gender_ratio: Option<HashMap<String, f64>>,
    #[serde(rename = "otherFormes")]
    other_formes: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct AbilityData {
    #[allow(dead_code)]
    name: String,
    num: i16,
}

#[derive(Deserialize)]
struct MoveData {
    #[allow(dead_code)]
    name: String,
    num: i16,
    #[serde(rename = "basePower")]
    base_power: Option<u16>,
    accuracy: Option<serde_json::Value>, // Can be bool (true = always hits) or number
    pp: Option<u8>,
    priority: Option<i8>,
    category: Option<String>,
    #[serde(rename = "type")]
    move_type: Option<String>,
    #[serde(default)]
    flags: HashMap<String, u8>,
    terrain: Option<String>,
    
    // Recoil fields for Reckless ability
    recoil: Option<serde_json::Value>,
    #[serde(rename = "hasCrashDamage")]
    has_crash_damage: Option<bool>,
    #[serde(rename = "mindBlownRecoil")]
    mind_blown_recoil: Option<bool>,
    #[allow(dead_code)]
    #[serde(rename = "struggleRecoil")]
    struggle_recoil: Option<bool>,

    // Fields for Sheer Force
    secondary: Option<serde_json::Value>,
    secondaries: Option<serde_json::Value>,
    #[serde(rename = "hasSheerForce")]
    has_sheer_force: Option<bool>,
}

#[derive(Deserialize)]
struct Fling {
    #[serde(rename = "basePower")]
    base_power: u8,
}

#[derive(Deserialize)]
struct ItemData {
    #[allow(dead_code)]
    name: String,
    num: Option<i16>,
    #[allow(dead_code)]
    #[serde(default)]
    gen: u8,
    #[serde(rename = "isNonstandard")]
    is_nonstandard: Option<String>,
    fling: Option<Fling>,
    #[serde(default)]
    #[serde(rename = "megaStone")]
    mega_stone: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "zMove")]
    z_move: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "onPlate")]
    on_plate: Option<String>,
    #[serde(default)]
    #[serde(rename = "onMemory")]
    on_memory: Option<String>,
    #[serde(default)]
    #[serde(rename = "onDrive")]
    on_drive: Option<String>,
    #[serde(default)]
    #[serde(rename = "forcedForme")]
    forced_forme: Option<String>,
}

// ============================================================================
// Code Generation
// ============================================================================

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = Path::new(&manifest_dir).join("../../data");

    // Rerun if any data file changes
    for file in &[
        "natures.json",
        "typechart.json",
        "pokedex.json",
        "abilities.json",
        "moves.json",
        "items.json",
    ] {
        println!("cargo:rerun-if-changed={}", data_dir.join(file).display());
    }

    let out_path = Path::new(&out_dir);

    // Generate each module
    generate_types(out_path, &data_dir);
    generate_natures(out_path, &data_dir);
    generate_abilities(out_path, &data_dir);
    generate_species(out_path, &data_dir);
    generate_moves(out_path, &data_dir);
    generate_items(out_path, &data_dir);
    generate_terrains(out_path, &data_dir);
}

/// Generate Type enum and type chart
fn generate_types(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("typechart.json")).expect("typechart.json");
    let chart: BTreeMap<String, TypeChartEntry> =
        serde_json::from_str(&json).expect("parse typechart");

    // Canonical type order (alphabetical, matching JSON keys)
    let type_names: Vec<&str> = chart.keys().map(|s| s.as_str()).collect();
    let type_count = type_names.len();

    // Generate Type enum variants
    let variants: Vec<TokenStream> = type_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let ident = format_ident!("{}", name.to_pascal_case());
            let idx = i as u8;
            quote! { #ident = #idx }
        })
        .collect();

    // Generate match arms for from_str
    let from_str_arms: Vec<TokenStream> = type_names
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name.to_pascal_case());
            let lower = name.to_lowercase();
            let pascal = name.to_pascal_case();
            quote! {
                #lower | #pascal => Some(Type::#ident)
            }
        })
        .collect();

    // Collect type names as PascalCase for distinguishing types from status keys
    let type_name_set: HashSet<String> = type_names
        .iter()
        .map(|s| s.to_pascal_case())
        .collect();

    // Extract status immunity keys dynamically (keys in damageTaken that aren't type names)
    let mut status_key_set: BTreeSet<String> = BTreeSet::new();
    for entry in chart.values() {
        for key in entry.damage_taken.keys() {
            if !type_name_set.contains(key) {
                status_key_set.insert(key.clone());
            }
        }
    }
    let status_keys: Vec<String> = status_key_set.into_iter().collect();

    // Generate bitflags constants dynamically
    let immunity_flags: Vec<TokenStream> = status_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let ident = format_ident!("{}", key.to_uppercase());
            let bit = 1u16 << i;
            quote! { const #ident = #bit; }
        })
        .collect();

    let immunity_rows: Vec<TokenStream> = type_names
        .iter()
        .map(|type_name| {
            let entry = &chart[*type_name];
            let mut bits: u16 = 0;
            for (i, key) in status_keys.iter().enumerate() {
                if entry.damage_taken.get(key) == Some(&3) {
                    bits |= 1 << i;
                }
            }
            quote! { TypeImmunities::from_bits_truncate(#bits) }
        })
        .collect();

    // Build effectiveness matrix
    // Matrix[defender][attacker] = effectiveness
    let mut matrix: Vec<Vec<u8>> = vec![vec![0; type_count]; type_count];
    for (def_idx, def_name) in type_names.iter().enumerate() {
        let entry = &chart[*def_name];
        for (atk_idx, atk_name) in type_names.iter().enumerate() {
            let atk_pascal = atk_name.to_pascal_case();
            let eff = entry.damage_taken.get(&atk_pascal).copied().unwrap_or(0);
            matrix[def_idx][atk_idx] = eff;
        }
    }

    let matrix_rows: Vec<TokenStream> = matrix
        .iter()
        .map(|row| {
            let cells: Vec<TokenStream> = row
                .iter()
                .map(|&v| {
                    let ident = match v {
                        0 => format_ident!("Normal"),
                        1 => format_ident!("SuperEffective"),
                        2 => format_ident!("Resistant"),
                        3 => format_ident!("Immune"),
                        _ => format_ident!("Normal"),
                    };
                    quote! { TypeEffectiveness::#ident }
                })
                .collect();
            quote! { [#(#cells),*] }
        })
        .collect();

    let type_count_lit = type_count;

    let code = quote! {
        use bitflags::bitflags;

        /// Pokemon type (19 types including Stellar)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum Type {
            #(#variants),*
        }

        impl Type {
            /// Total number of types
            pub const COUNT: usize = #type_count_lit;

            /// Parse type from string (case-insensitive)
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_arms,)*
                    _ => None,
                }
            }
        }

        /// Type effectiveness multiplier
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u8)]
        pub enum TypeEffectiveness {
            /// 1.0x damage
            Normal = 0,
            /// 2.0x damage
            SuperEffective = 1,
            /// 0.5x damage
            Resistant = 2,
            /// 0.0x damage (immune)
            Immune = 3,
        }

        impl TypeEffectiveness {
            /// Convert to fixed-point multiplier (4 = 1.0x)
            /// Returns: 0 (immune), 2 (0.5x), 4 (1.0x), 8 (2.0x)
            #[inline]
            pub const fn multiplier(self) -> u8 {
                match self {
                    Self::Normal => 4,
                    Self::SuperEffective => 8,
                    Self::Resistant => 2,
                    Self::Immune => 0,
                }
            }
        }

        bitflags! {
            /// Type-based immunities to status conditions and effects
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub struct TypeImmunities: u16 {
                #(#immunity_flags)*
            }
        }

        /// Type chart: TYPE_CHART[defender][attacker] = effectiveness
        pub static TYPE_CHART: [[TypeEffectiveness; #type_count_lit]; #type_count_lit] = [
            #(#matrix_rows),*
        ];

        /// Status immunities by type
        pub static TYPE_IMMUNITIES: [TypeImmunities; #type_count_lit] = [
            #(#immunity_rows),*
        ];

        /// Calculate type effectiveness for an attack
        /// Returns fixed-point multiplier: 0, 1, 2, 4, 8, 16 (representing 0x, 0.25x, 0.5x, 1x, 2x, 4x)
        #[inline]
        pub fn type_effectiveness(attacker: Type, defender1: Type, defender2: Option<Type>) -> u8 {
            let mut mult = TYPE_CHART[defender1 as usize][attacker as usize].multiplier();
            if let Some(t2) = defender2 {
                mult = mult * TYPE_CHART[t2 as usize][attacker as usize].multiplier() / 4;
            }
            mult
        }
    };

    let dest = out_dir.join("types.rs");
    fs::write(&dest, code.to_string()).expect("write types.rs");
}

/// Generate Nature enum with inline stat modifier calculation
fn generate_natures(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("natures.json")).expect("natures.json");
    let natures: BTreeMap<String, NatureData> =
        serde_json::from_str(&json).expect("parse natures");

    // Stats affected by natures (HP is never affected)
    let stats = ["atk", "def", "spa", "spd", "spe"];

    // Build nature list with plus/minus indices
    // We'll order natures in a 5x5 grid: nature_id = plus * 5 + minus
    // Neutral natures go on the diagonal (plus == minus)
    let mut nature_grid: Vec<Option<String>> = vec![None; 25];
    let mut neutral_slot = 0usize;

    for (key, data) in &natures {
        if let (Some(plus), Some(minus)) = (&data.plus, &data.minus) {
            let plus_idx = stats.iter().position(|s| s == plus).unwrap();
            let minus_idx = stats.iter().position(|s| s == minus).unwrap();
            let grid_idx = plus_idx * 5 + minus_idx;
            nature_grid[grid_idx] = Some(key.clone());
        } else {
            // Neutral nature - place on diagonal
            while nature_grid[neutral_slot * 6].is_some() {
                neutral_slot += 1;
            }
            nature_grid[neutral_slot * 6] = Some(key.clone());
            neutral_slot += 1;
        }
    }

    // Generate enum variants
    let variants: Vec<TokenStream> = nature_grid
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            name.as_ref().map(|n| {
                let ident = format_ident!("{}", n.to_pascal_case());
                let idx = i as u8;
                quote! { #ident = #idx }
            })
        })
        .collect();

    // Generate from_str arms
    let from_str_arms: Vec<TokenStream> = nature_grid
        .iter()
        .filter_map(|name| {
            name.as_ref().map(|n| {
                let ident = format_ident!("{}", n.to_pascal_case());
                let lower = n.to_lowercase();
                let pascal = n.to_pascal_case();
                quote! { #lower | #pascal => Some(NatureId::#ident) }
            })
        })
        .collect();

    let code = quote! {
        /// Pokemon nature (affects stat growth)
        /// Ordered in a 5x5 grid: nature_id = plus_stat * 5 + minus_stat
        /// Diagonal entries (where plus == minus) are neutral natures
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(u8)]
        pub enum NatureId {
            #[default]
            #(#variants),*
        }

        /// Stat index for nature-affected stats (HP excluded)
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u8)]
        pub enum BattleStat {
            Atk = 0,
            Def = 1,
            SpA = 2,
            SpD = 3,
            Spe = 4,
        }

        impl NatureId {
            /// Parse nature from string (case-insensitive)
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_arms,)*
                    _ => None,
                }
            }

            /// Get stat modifier for a given stat
            /// Returns: 9 (-10%), 10 (neutral), 11 (+10%)
            /// Multiply by stat/10 to apply
            #[inline]
            pub const fn stat_modifier(self, stat: BattleStat) -> u8 {
                let id = self as u8;
                let plus = id / 5;
                let minus = id % 5;
                let stat_idx = stat as u8;

                if plus == minus {
                    10 // Neutral nature
                } else if stat_idx == plus {
                    11 // +10%
                } else if stat_idx == minus {
                    9 // -10%
                } else {
                    10 // Unaffected
                }
            }

            /// Check if this is a neutral nature (no stat changes)
            #[inline]
            pub const fn is_neutral(self) -> bool {
                let id = self as u8;
                (id / 5) == (id % 5)
            }
        }
    };

    let dest = out_dir.join("natures.rs");
    fs::write(&dest, code.to_string()).expect("write natures.rs");
}

/// Generate AbilityId enum (IDs only, no hooks yet)
fn generate_abilities(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("abilities.json")).expect("abilities.json");
    let abilities: BTreeMap<String, AbilityData> =
        serde_json::from_str(&json).expect("parse abilities");

    // Sort by num to get stable ordering
    let mut ability_list: Vec<(&String, &AbilityData)> = abilities.iter().collect();
    ability_list.sort_by_key(|(_, data)| data.num);

    // Filter out negative nums (non-standard) and create mapping
    let valid_abilities: Vec<(&String, &AbilityData)> = ability_list
        .into_iter()
        .filter(|(_, data)| data.num >= 0)
        .collect();

    // Build index mapping: ability_key -> sequential index
    let mut key_to_index: BTreeMap<&str, u16> = BTreeMap::new();
    for (i, (key, _)) in valid_abilities.iter().enumerate() {
        key_to_index.insert(key.as_str(), i as u16);
    }

    let count = valid_abilities.len();

    // Generate enum variants
    let variants: Vec<TokenStream> = valid_abilities
        .iter()
        .enumerate()
        .map(|(i, (key, _))| {
            let ident = format_ident!("{}", key.to_pascal_case());
            let idx = i as u16;
            quote! { #ident = #idx }
        })
        .collect();

    // Generate phf map for string -> AbilityId lookup
    let mut phf_map = phf_codegen::Map::new();
    for (key, _) in &valid_abilities {
        let ident = key.to_pascal_case();
        phf_map.entry(key.as_str(), &format!("AbilityId::{}", ident));
    }

    let phf_str = phf_map.build().to_string();

    let code = quote! {
        /// Ability identifier (sorted by game index)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(u16)]
        pub enum AbilityId {
            #[default]
            #(#variants),*
        }

        impl AbilityId {
            /// Total number of abilities
            pub const COUNT: usize = #count;

            /// Look up ability by key string
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                ABILITY_LOOKUP.get(s).copied()
            }
        }
    };

    let dest = out_dir.join("abilities.rs");
    let mut file = BufWriter::new(File::create(&dest).expect("create abilities.rs"));
    writeln!(file, "{}", code).unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "static ABILITY_LOOKUP: phf::Map<&'static str, AbilityId> = {};",
        phf_str
    )
    .unwrap();
}

/// Generate Species data and lookup maps
fn generate_species(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("pokedex.json")).expect("pokedex.json");
    let pokedex: BTreeMap<String, PokedexEntry> =
        serde_json::from_str(&json).expect("parse pokedex");

    // Load abilities for lookup
    let abilities_json =
        fs::read_to_string(data_dir.join("abilities.json")).expect("abilities.json");
    let abilities: BTreeMap<String, AbilityData> =
        serde_json::from_str(&abilities_json).expect("parse abilities");

    // Build ability key -> index mapping
    let mut ability_list: Vec<(&String, &AbilityData)> = abilities.iter().collect();
    ability_list.sort_by_key(|(_, data)| data.num);
    let valid_abilities: Vec<&String> = ability_list
        .into_iter()
        .filter(|(_, data)| data.num >= 0)
        .map(|(key, _)| key)
        .collect();
    let ability_to_idx: HashMap<&str, u16> = valid_abilities
        .iter()
        .enumerate()
        .map(|(i, key)| (key.as_str(), i as u16))
        .collect();

    // Load typechart for type name -> index
    let typechart_json =
        fs::read_to_string(data_dir.join("typechart.json")).expect("typechart.json");
    let typechart: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&typechart_json).expect("parse typechart");
    let type_names: Vec<&str> = typechart.keys().map(|s| s.as_str()).collect();
    let type_to_idx: HashMap<&str, u8> = type_names
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, i as u8))
        .collect();

    // Filter valid species (has num > 0 and has base stats)
    let valid_species: Vec<(&String, &PokedexEntry)> = pokedex
        .iter()
        .filter(|(_, entry)| {
            entry.num.map(|n| n > 0).unwrap_or(false)
                && entry.base_stats.is_some()
                && entry.types.is_some()
        })
        .collect();

    // Build key -> index mapping
    let species_keys: Vec<&str> = valid_species.iter().map(|(k, _)| k.as_str()).collect();
    let key_to_idx: HashMap<&str, u16> = species_keys
        .iter()
        .enumerate()
        .map(|(i, k)| (*k, i as u16))
        .collect();

    // Collect all base species for lookup (normalized to key format)
    let base_species_set: HashSet<String> = valid_species
        .iter()
        .filter_map(|(_, entry)| {
            entry.base_species.as_ref().map(|name| {
                name.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
        })
        .collect();

    // Also include species that are in "otherFormes" of someone else?
    // Not strictly needed if we just iterate and resolve IDs.

    // Generate species data array
    let count = valid_species.len();

    let species_data: Vec<TokenStream> = valid_species
        .iter()
        .map(|(key, entry)| {
            let stats = entry.base_stats.as_ref().unwrap();
            let hp = stats.hp;
            let atk = stats.atk;
            let def = stats.def;
            let spa = stats.spa;
            let spd = stats.spd;
            let spe = stats.spe;

            // Types
            let types = entry.types.as_ref().unwrap();
            let type1 = types
                .first()
                .and_then(|t| type_to_idx.get(t.to_lowercase().as_str()))
                .copied()
                .unwrap_or(0);
            let type2 = types
                .get(1)
                .and_then(|t| type_to_idx.get(t.to_lowercase().as_str()))
                .map(|&t| t + 1) // +1 so 0 means "no second type"
                .unwrap_or(0);

            // Weight (fixed-point: kg * 10)
            let weight = (entry.weightkg * 10.0).round() as u16;

            // Abilities (up to 3: slot 0, slot 1, hidden)
            let ability_key = |slot: &str| -> u16 {
                entry
                    .abilities
                    .get(slot)
                    .and_then(|name| {
                        // Convert ability name to key format
                        let key = name
                            .to_lowercase()
                            .chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect::<String>();
                        ability_to_idx.get(key.as_str()).copied()
                    })
                    .unwrap_or(0)
            };
            let ability0 = ability_key("0");
            let ability1 = ability_key("1");
            let hidden = ability_key("H");

            // Base species (for forms)
            let base = entry
                .base_species
                .as_ref()
                .and_then(|name| {
                    let base_key = name
                        .to_lowercase()
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>();
                    key_to_idx.get(base_key.as_str()).copied()
                })
                .map(|i| i + 1) // +1 so 0 means "is base species"
                .unwrap_or(0);

            // Flags
            // Shedinja always has 1 HP (mechanics/stats.md)
            let flags: u8 = if entry.name == "Shedinja" { 1 << 0 } else { 0 };

            // Gender Ratio
            let gender_ratio_tokens = match entry.gender.as_deref() {
                Some("N") => quote! { GenderRatio::Genderless },
                Some("M") => quote! { GenderRatio::AlwaysMale },
                Some("F") => quote! { GenderRatio::AlwaysFemale },
                _ => {
                    match &entry.gender_ratio {
                        Some(ratio) => {
                             let m = ratio.get("M").copied().unwrap_or(0.5);
                             if m >= 0.875 { quote! { GenderRatio::SevenToOne } }
                             else if m >= 0.75 { quote! { GenderRatio::ThreeToOne } }
                             else if m >= 0.5 { quote! { GenderRatio::OneToOne } }
                             else if m >= 0.25 { quote! { GenderRatio::OneToThree } }
                             else if m >= 0.125 { quote! { GenderRatio::OneToSeven } }
                             else { quote! { GenderRatio::AlwaysFemale } }
                        },
                        None => quote! { GenderRatio::OneToOne },
                    }
                }
            };

            // Helper to find related species ID
            let _find_related = |suffix: &str| -> u16 {
                // Suffix check: key + suffix
                let target_key = format!("{}{}", key, suffix);
                if let Some(&idx) = key_to_idx.get(target_key.as_str()) {
                     return idx + 1; // 0 = None, idx+1 = valid ID
                }

                // If not found by direct suffix concatenation, try standard naming conventions
                // e.g., "charizard" -> "charizardmegax" (already covered above)
                // But sometimes "groudon" -> "groudonprimal"
                0
            };

            // Forme Lookups
            // We search for standard keys.
            // Note: This is a simplistic heuristic. Pokedex has "otherFormes" array which lists names.
            // But we need to map those names to IDs.

            // Mega Forme
            // Usually Key + "megax" or Key + "mega" or Key + "megay"
            let mut mega = 0u16;
            let mut mega_y = 0u16;
            let mut primal = 0u16;

            if let Some(formes) = &entry.other_formes {
                 for forme_name in formes {
                     let forme_key = forme_name
                        .to_lowercase()
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>();

                     if let Some(&idx) = key_to_idx.get(forme_key.as_str()) {
                         if forme_key.ends_with("megax") {
                             mega = idx + 1;
                         } else if forme_key.ends_with("megay") {
                             mega_y = idx + 1;
                         } else if forme_key.ends_with("mega") {
                             mega = idx + 1;
                         } else if forme_key.ends_with("primal") {
                             primal = idx + 1;
                         }
                     }
                 }
            }

            quote! {
                Species {
                    base_stats: [#hp, #atk, #def, #spa, #spd, #spe],
                    type1: #type1,
                    type2: #type2,
                    weight: #weight,
                    ability0: #ability0,
                    ability1: #ability1,
                    hidden_ability: #hidden,
                    base_species: #base,
                    flags: #flags,
                    gender_ratio: #gender_ratio_tokens,
                    mega_forme: #mega,
                    mega_forme_y: #mega_y,
                    primal_forme: #primal,
                }
            }
        })
        .collect();

    // Generate phf map for string -> SpeciesId lookup
    let mut phf_map = phf_codegen::Map::new();
    for (key, idx) in &key_to_idx {
        phf_map.entry(*key, &format!("SpeciesId({})", idx));
    }
    let phf_str = phf_map.build().to_string();

    // Generate base species lookup map
    let mut base_phf = phf_codegen::Map::new();
    for (key, entry) in &valid_species {
        if entry.base_species.is_none() && base_species_set.contains(*key) {
            // This is a base species that has forms
            if let Some(&idx) = key_to_idx.get(key.as_str()) {
                base_phf.entry(*key, &format!("SpeciesId({})", idx));
            }
        }
    }
    let base_phf_str = base_phf.build().to_string();

    let code = quote! {
        /// Species identifier (index into SPECIES array)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(transparent)]
        pub struct SpeciesId(pub u16);

        /// Gender ratio for species
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum GenderRatio {
            AlwaysMale,      // 100% male
            AlwaysFemale,    // 100% female
            Genderless,      // No gender
            OneToOne,        // 50/50
            OneToThree,      // 25% male, 75% female
            ThreeToOne,      // 75% male, 25% female
            OneToSeven,      // 12.5% male, 87.5% female
            SevenToOne,      // 87.5% male, 12.5% female
        }

        /// Static species data
        #[derive(Clone, Copy, Debug)]
        #[repr(C)]
        pub struct Species {
            /// Base stats: [hp, atk, def, spa, spd, spe]
            pub base_stats: [u8; 6],
            /// Primary type index
            pub type1: u8,
            /// Secondary type index + 1 (0 = no second type)
            pub type2: u8,
            /// Weight in 0.1 kg units
            pub weight: u16,
            /// Primary ability index
            pub ability0: u16,
            /// Secondary ability index (0 = none)
            pub ability1: u16,
            /// Hidden ability index (0 = none)
            pub hidden_ability: u16,
            /// Base species index + 1 for forms (0 = is base species)
            pub base_species: u16,
            /// Species flags (e.g., Force 1 HP)
            pub flags: u8,
            /// Gender ratio
            pub gender_ratio: GenderRatio,
            /// Mega Forme ID + 1 (0 = none)
            pub mega_forme: u16,
            /// Mega Y Forme ID + 1 (0 = none)
            pub mega_forme_y: u16,
            /// Primal Forme ID + 1 (0 = none)
            pub primal_forme: u16,
        }

        /// Flag: Shedinja's HP is always 1
        pub const FLAG_FORCE_1_HP: u8 = 1 << 0;

        impl SpeciesId {
            /// Total number of species
            pub const COUNT: usize = #count;

            /// Look up species by key string
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                SPECIES_LOOKUP.get(s).copied()
            }

            /// Get species data
            #[inline]
            pub fn data(self) -> &'static Species {
                &SPECIES[self.0 as usize]
            }

            /// Get base species (returns self if already base)
            #[inline]
            pub fn base(self) -> Self {
                let base = SPECIES[self.0 as usize].base_species;
                if base == 0 {
                    self
                } else {
                    SpeciesId(base - 1)
                }
            }
        }

        impl Species {
            /// Get primary type
            #[inline]
            pub fn primary_type(&self) -> super::types::Type {
                unsafe { core::mem::transmute(self.type1) }
            }

            /// Get secondary type (if any)
            #[inline]
            pub fn secondary_type(&self) -> Option<super::types::Type> {
                if self.type2 == 0 {
                    None
                } else {
                    Some(unsafe { core::mem::transmute(self.type2 - 1) })
                }
            }

            /// Get primary ability
            #[inline]
            pub fn primary_ability(&self) -> super::abilities::AbilityId {
                unsafe { core::mem::transmute(self.ability0) }
            }

            /// Get Mega Forme (if any)
            #[inline]
            pub fn mega_forme(&self) -> Option<SpeciesId> {
                if self.mega_forme == 0 { None } else { Some(SpeciesId(self.mega_forme - 1)) }
            }

            /// Get Mega Y Forme (if any)
            #[inline]
            pub fn mega_forme_y(&self) -> Option<SpeciesId> {
                if self.mega_forme_y == 0 { None } else { Some(SpeciesId(self.mega_forme_y - 1)) }
            }

            /// Get Primal Forme (if any)
            #[inline]
            pub fn primal_forme(&self) -> Option<SpeciesId> {
                if self.primal_forme == 0 { None } else { Some(SpeciesId(self.primal_forme - 1)) }
            }
        }

        /// Static species data array
        pub static SPECIES: [Species; #count] = [
            #(#species_data),*
        ];
    };

    let dest = out_dir.join("species.rs");
    let mut file = BufWriter::new(File::create(&dest).expect("create species.rs"));
    writeln!(file, "{}", code).unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "static SPECIES_LOOKUP: phf::Map<&'static str, SpeciesId> = {};",
        phf_str
    )
    .unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "/// Lookup for base species only (species that have alternate forms)"
    )
    .unwrap();
    writeln!(
        file,
        "pub static BASE_SPECIES_LOOKUP: phf::Map<&'static str, SpeciesId> = {};",
        base_phf_str
    )
    .unwrap();
}

/// Generate MoveId enum and move data
fn generate_moves(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("moves.json")).expect("moves.json");
    let moves: BTreeMap<String, MoveData> = serde_json::from_str(&json).expect("parse moves");

    // Sort by num to get stable ordering, filter out invalid
    let mut move_list: Vec<(&String, &MoveData)> = moves.iter().collect();
    move_list.sort_by_key(|(_, data)| data.num);

    let valid_moves: Vec<(&String, &MoveData)> = move_list
        .into_iter()
        .filter(|(_, data)| data.num >= 0)
        .collect();

    let count = valid_moves.len();

    // 1. Collect Flags
    let mut flag_names = BTreeSet::new();
    for (_, data) in &valid_moves {
        for flag in data.flags.keys() {
            flag_names.insert(flag.clone());
        }
        
        // Reckless boost criteria: has recoil, crash damage, or mind blown recoil
        if data.recoil.is_some() 
            || data.has_crash_damage.unwrap_or(false)
            || data.mind_blown_recoil.unwrap_or(false) 
        {
            flag_names.insert("Recoil".to_string());
        }

        // Sheer Force boost criteria: has secondary effects or explicit flag
        if has_secondary_effects(data) {
            flag_names.insert("HasSecondaryEffects".to_string());
        }
    }
    let flag_count = flag_names.len();
    let use_u64 = flag_count > 32;
    let flags_repr = if use_u64 { quote!(u64) } else { quote!(u32) };

    let flag_consts: Vec<TokenStream> = flag_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let ident = format_ident!("{}", name.to_shouty_snake_case());
            let val = if use_u64 {
                let v = 1u64 << i;
                quote! { #v }
            } else {
                let v = 1u32 << i;
                quote! { #v }
            };
            quote! { const #ident = #val; }
        })
        .collect();

    // 2. Generate Enum Variants
    let variants: Vec<TokenStream> = valid_moves
        .iter()
        .enumerate()
        .map(|(i, (key, _))| {
            let ident = format_ident!("{}", to_valid_ident(key));
            let idx = i as u16;
            quote! { #ident = #idx }
        })
        .collect();

    // 3. Generate Move Data Entries
    let move_data_entries: Vec<TokenStream> = valid_moves.iter().map(|(_, data)| {
         let name = &data.name;
         let type_str = data.move_type.as_deref().unwrap_or("Normal");
         let type_ident = format_ident!("{}", type_str);

         let cat_str = data.category.as_deref().unwrap_or("Status");
         let cat_ident = format_ident!("{}", cat_str);

         let power = data.base_power.unwrap_or(0);

         let accuracy = match &data.accuracy {
             Some(serde_json::Value::Bool(true)) => 0,
             Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as u8,
             _ => 0,
         };

         let pp = data.pp.unwrap_or(0);
         let priority = data.priority.unwrap_or(0);

         // Flags
         let mut flag_bits = 0u64;
         for (flag_key, _) in &data.flags {
             if let Some(pos) = flag_names.iter().position(|x| x == flag_key) {
                 flag_bits |= 1 << pos;
             }
         }
         
         // Inject Recoil flag bit
         if data.recoil.is_some() 
             || data.has_crash_damage.unwrap_or(false)
             || data.mind_blown_recoil.unwrap_or(false) 
         {
             if let Some(pos) = flag_names.iter().position(|x| x == "Recoil") {
                 flag_bits |= 1 << pos;
             }
         }

         // Inject HasSecondaryEffects flag bit
         if has_secondary_effects(data) {
             if let Some(pos) = flag_names.iter().position(|x| x == "HasSecondaryEffects") {
                 flag_bits |= 1 << pos;
             }
         }

         let flag_bits_lit = if use_u64 {
             quote! { #flag_bits }
         } else {
             let val = flag_bits as u32;
             quote! { #val }
         };

         // Terrain
         let terrain_ident = if let Some(t) = &data.terrain {
             let t_ident = format_ident!("{}", t.replace("terrain", "").to_pascal_case());
             quote! { TerrainId::#t_ident }
         } else {
             quote! { TerrainId::None }
         };

         quote! {
             Move {
                 name: #name,
                 primary_type: Type::#type_ident,
                 category: MoveCategory::#cat_ident,
                 power: #power,
                 accuracy: #accuracy,
                 pp: #pp,
                 priority: #priority,
                 flags: MoveFlags::from_bits_truncate(#flag_bits_lit),
                 terrain: #terrain_ident,
             }
         }
    }).collect();

    // Generate phf map for string -> MoveId lookup
    let mut phf_map = phf_codegen::Map::new();
    for (key, _) in &valid_moves {
        let ident = to_valid_ident(key);
        phf_map.entry(key.as_str(), &format!("MoveId::{}", ident));
    }
    let phf_str = phf_map.build().to_string();

    let code = quote! {
        use super::types::Type;
        use super::terrains::TerrainId;
        use bitflags::bitflags;

        /// Move identifier (sorted by game index)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(u16)]
        pub enum MoveId {
            #[default]
            #(#variants),*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u8)]
        pub enum MoveCategory {
            Physical,
            Special,
            Status,
        }

        bitflags! {
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
            pub struct MoveFlags: #flags_repr {
                #(#flag_consts)*
            }
        }

        /// Static move data
        #[derive(Clone, Copy, Debug)]
        pub struct Move {
            pub name: &'static str,
            pub primary_type: Type,
            pub category: MoveCategory,
            pub power: u16,
            pub accuracy: u8, // 0 = always hits
            pub pp: u8,
            pub priority: i8,
            pub flags: MoveFlags,
            pub terrain: TerrainId,
        }

        impl MoveId {
            /// Total number of moves
            pub const COUNT: usize = #count;

            /// Look up move by key string
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                MOVE_LOOKUP.get(s).copied()
            }

            /// Get move data
            #[inline]
            pub fn data(self) -> &'static Move {
                &MOVES[self as usize]
            }
        }

        /// Static move data array
        pub static MOVES: [Move; #count] = [
            #(#move_data_entries),*
        ];
    };

    let dest = out_dir.join("moves.rs");
    let mut file = BufWriter::new(File::create(&dest).expect("create moves.rs"));
    writeln!(file, "{}", code).unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "static MOVE_LOOKUP: phf::Map<&'static str, MoveId> = {};",
        phf_str
    )
    .unwrap();
}

/// Generate ItemId enum
fn generate_items(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("items.json")).expect("items.json");
    let items: BTreeMap<String, ItemData> = serde_json::from_str(&json).expect("parse items");

    // Filter valid items (has num, not nonstandard "Past"/"Future" unless we want them)
    let mut item_list: Vec<(&String, &ItemData)> = items
        .iter()
        .filter(|(_, data)| {
            data.num.map(|n| n >= 0).unwrap_or(false)
                // We want all items, even "Past" ones, to support older generations
                // && data.is_nonstandard.as_ref().map(|s| s != "Past").unwrap_or(true)
        })
        .collect();
    item_list.sort_by_key(|(_, data)| data.num.unwrap_or(0));

    let count = item_list.len() + 1; // +1 for None

    // Generate enum variants
    let variants: Vec<TokenStream> = item_list
        .iter()
        .enumerate()
        .map(|(i, (key, _))| {
            let ident = format_ident!("{}", to_valid_ident(key));
            let idx = (i + 1) as u16; // Shift by 1
            quote! { #ident = #idx }
        })
        .collect();

    // Generate phf map for string -> ItemId lookup
    let mut phf_map = phf_codegen::Map::new();
    for (key, _) in &item_list {
        let ident = to_valid_ident(key);
        phf_map.entry(key.as_str(), &format!("ItemId::{}", ident));
    }
    let phf_str = phf_map.build().to_string();

    let item_data: Vec<TokenStream> = item_list
        .iter()
        .map(|(_, data)| {
            let fling_power = data.fling.as_ref().map(|f| f.base_power).unwrap_or(0);
            let is_unremovable = data.mega_stone.is_some()
                || data.z_move.is_some()
                || data.on_plate.is_some()
                || data.on_memory.is_some()
                || data.on_drive.is_some()
                || data.forced_forme.is_some()
                || (data.name == "Rusted Sword")
                || (data.name == "Rusted Shield")
                || (data.name == "Booster Energy");

            quote! {
                Item {
                    fling_power: #fling_power,
                    is_unremovable: #is_unremovable,
                }
            }
        })
        .collect();

    let code = quote! {
        /// Item identifier (sorted by game index)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(u16)]
        pub enum ItemId {
            #[default]
            None = 0,
            #(#variants),*
        }

        /// Static item data
        #[derive(Clone, Copy, Debug)]
        pub struct Item {
            /// Fling base power (0 = cannot be flung)
            pub fling_power: u8,
            /// Whether the item can be removed by Knock Off, etc.
            pub is_unremovable: bool,
        }

        impl ItemId {
            /// Total number of items
            pub const COUNT: usize = #count;

            /// Look up item by key string
            #[inline]
            pub fn from_str(s: &str) -> Option<Self> {
                ITEM_LOOKUP.get(s).copied()
            }

            /// Get item data
            #[inline]
            pub fn data(self) -> &'static Item {
                &ITEMS[self as usize]
            }
        }

        /// Static item data array
        pub static ITEMS: [Item; #count] = [
            Item { fling_power: 0, is_unremovable: false }, // None
            #(#item_data),*
        ];
    };

    let dest = out_dir.join("items.rs");
    let mut file = BufWriter::new(File::create(&dest).expect("create items.rs"));
    writeln!(file, "{}", code).unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "static ITEM_LOOKUP: phf::Map<&'static str, ItemId> = {};",
        phf_str
    )
    .unwrap();
}

/// Generate TerrainId enum
fn generate_terrains(out_dir: &Path, data_dir: &Path) {
    let json = fs::read_to_string(data_dir.join("moves.json")).expect("moves.json");
    let moves: BTreeMap<String, MoveData> = serde_json::from_str(&json).expect("parse moves");

    // Extract unique terrains
    let mut terrains: BTreeSet<String> = BTreeSet::new();
    for data in moves.values() {
        if let Some(t) = &data.terrain {
            terrains.insert(t.clone());
        }
    }

    // Generate enum variants
    // 0 is Default (None), others follow
    let variants: Vec<TokenStream> = terrains
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let ident = format_ident!("{}", name.replace("terrain", "").to_pascal_case());
            let idx = (i + 1) as u8; // 0 reserved for None
            quote! { #ident = #idx }
        })
        .collect();

    let from_str_arms: Vec<TokenStream> = terrains
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name.replace("terrain", "").to_pascal_case());
            quote! { #name => Some(TerrainId::#ident) }
        })
        .collect();

    let code = quote! {
        /// Terrain Type
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        #[repr(u8)]
        pub enum TerrainId {
            #[default]
            None = 0,
            #(#variants),*
        }

        impl TerrainId {
            /// Parse terrain from string (e.g., "electricterrain")
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_arms,)*
                    _ => None,
                }
            }
        }
    };

    let dest = out_dir.join("terrains.rs");
    fs::write(&dest, code.to_string()).expect("write terrains.rs");
}
