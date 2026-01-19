//! TerrainId enum generation.

use crate::models::MoveData;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Generate TerrainId enum
pub fn generate(out_dir: &Path, data_dir: &Path) {
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
