//! Species data and lookup map generation.

use crate::models::{AbilityData, PokedexEntry};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, writeln};

/// Generate Species data and lookup maps
pub fn generate(out_dir: &Path, data_dir: &Path) {
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

    // Generate species data array
    let count = valid_species.len();

    let species_data: Vec<TokenStream> = valid_species
        .iter()
        .map(|(_key, entry)| {
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

            // Forme Lookups
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
