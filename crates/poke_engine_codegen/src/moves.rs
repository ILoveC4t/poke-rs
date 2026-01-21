//! MoveId enum and move data generation.

use crate::helpers::{has_secondary_effects, to_valid_ident};
use crate::models::MoveData;
use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, writeln};

/// Generate MoveId enum and move data
pub fn generate(out_dir: &Path, data_dir: &Path) {
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

    let breaks_screens_moves = ["Brick Break", "Psychic Fangs"];
    let variable_power_moves = [
        "Eruption", "Water Spout", "Flail", "Reversal", "Low Kick", "Grass Knot", "Heavy Slam",
        "Heat Crash", "Gyro Ball", "Electro Ball", "Crush Grip", "Wring Out",
    ];

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

        if breaks_screens_moves.contains(&data.name.as_str()) {
            flag_names.insert("BreaksScreens".to_string());
        }

        if variable_power_moves.contains(&data.name.as_str()) {
            flag_names.insert("VariablePower".to_string());
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

         if breaks_screens_moves.contains(&data.name.as_str()) {
             if let Some(pos) = flag_names.iter().position(|x| x == "BreaksScreens") {
                 flag_bits |= 1 << pos;
             }
         }

         if variable_power_moves.contains(&data.name.as_str()) {
             if let Some(pos) = flag_names.iter().position(|x| x == "VariablePower") {
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
