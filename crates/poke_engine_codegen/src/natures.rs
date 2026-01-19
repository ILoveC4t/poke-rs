//! Nature enum generation.

use crate::models::NatureData;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Generate Nature enum with inline stat modifier calculation
pub fn generate(out_dir: &Path, data_dir: &Path) {
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
