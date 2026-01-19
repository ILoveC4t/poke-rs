//! ItemId enum generation.

use crate::helpers::to_valid_ident;
use crate::models::ItemData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, writeln};

/// Generate ItemId enum
pub fn generate(out_dir: &Path, data_dir: &Path) {
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
