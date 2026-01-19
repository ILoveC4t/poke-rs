//! Type enum and type chart generation.

use crate::models::TypeChartEntry;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Path;

/// Generate Type enum and type chart
pub fn generate(out_dir: &Path, data_dir: &Path) {
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
