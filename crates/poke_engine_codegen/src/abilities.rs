//! AbilityId enum generation.

use crate::models::AbilityData;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, writeln};

/// Generate AbilityId enum (IDs only, no hooks yet)
pub fn generate(out_dir: &Path, data_dir: &Path) {
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
