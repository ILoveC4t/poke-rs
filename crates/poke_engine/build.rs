use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use heck::ToPascalCase;
use serde::Deserialize;
use serde_json::Value;
use quote::{quote, format_ident};

// Hardcoded Type Chart (Gen 9)
// 0: Normal, 1: Fighting, ... 18: Stellar
// Attacker (Row) vs Defender (Col)
const TYPE_CHART_RAW: [[f32; 19]; 19] = [
    // Nor Fig Fly Poi Gro Roc Bug Gho Ste Fir Wat Gra Ele Psy Ice Dra Dar Fai Ste
    [1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // Normal
    [2.0, 1.0, 0.5, 0.5, 1.0, 2.0, 0.5, 0.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 2.0, 0.5, 1.0], // Fighting
    [1.0, 2.0, 1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // Flying
    [1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0], // Poison
    [1.0, 1.0, 0.0, 2.0, 1.0, 2.0, 0.5, 1.0, 2.0, 2.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // Ground
    [1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 2.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0], // Rock
    [1.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 2.0, 0.5, 1.0], // Bug
    [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5, 1.0, 1.0], // Ghost
    [1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 1.0, 2.0, 1.0, 1.0, 2.0, 1.0], // Steel
    [1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 2.0, 0.5, 0.5, 2.0, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0, 1.0], // Fire
    [1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 2.0, 0.5, 0.5, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0], // Water
    [1.0, 1.0, 0.5, 0.5, 2.0, 2.0, 0.5, 1.0, 0.5, 0.5, 2.0, 0.5, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0], // Grass
    [1.0, 1.0, 2.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 0.5, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0], // Electric
    [1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 0.0, 1.0, 1.0], // Psychic
    [1.0, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 2.0, 1.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0], // Ice
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.0, 1.0], // Dragon
    [1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5, 0.5, 1.0], // Dark
    [1.0, 2.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0], // Fairy
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // Stellar
];

#[derive(Deserialize)]
struct MoveJson {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "basePower")]
    base_power: u8,
    accuracy: Option<Value>,
    pp: u8,
    priority: Option<i8>,
    target: Option<String>,
    category: Option<String>,
    flags: Option<HashMap<String, u8>>,
    #[serde(rename = "isMax")]
    is_max: Option<Value>,
    #[serde(rename = "isZ")]
    is_z: Option<Value>,
    num: Option<i32>,
}

#[derive(Deserialize)]
struct SpeciesJson {
    name: String,
    types: Option<Vec<String>>,
    #[serde(rename = "baseStats")]
    base_stats: Option<HashMap<String, u16>>,
    abilities: Option<HashMap<String, String>>,
    #[serde(rename = "weightkg")]
    weightkg: Option<f32>,
    num: Option<i32>,
}

fn sanitize(name: &str) -> String {
    let mut s = name.to_string();
    s = s.replace("Type: Null", "TypeNull");
    s = s.replace("Nidoran♀", "NidoranF");
    s = s.replace("Nidoran♂", "NidoranM");
    s = s.replace("Farfetch'd", "Farfetchd");
    s = s.replace("Sirfetch'd", "Sirfetchd");
    s = s.replace("Mr. Mime", "MrMime");
    s = s.replace("Mr. Rime", "MrRime");
    s = s.replace("Mime Jr.", "MimeJr");
    s = s.replace("Porygon-Z", "PorygonZ");
    s = s.replace("Ho-Oh", "HoOh");
    s = s.replace("Jangmo-o", "JangmoO");
    s = s.replace("Hakamo-o", "HakamoO");
    s = s.replace("Kommo-o", "KommoO");
    s = s.replace("Tapu Koko", "TapuKoko");
    s = s.replace("Tapu Lele", "TapuLele");
    s = s.replace("Tapu Bulu", "TapuBulu");
    s = s.replace("Tapu Fini", "TapuFini");
    s = s.replace("Wo-Chien", "WoChien");
    s = s.replace("Chien-Pao", "ChienPao");
    s = s.replace("Ting-Lu", "TingLu");
    s = s.replace("Chi-Yu", "ChiYu");
    s = s.replace("Great Tusk", "GreatTusk");
    s = s.replace("Scream Tail", "ScreamTail");
    s = s.replace("Brute Bonnet", "BruteBonnet");
    s = s.replace("Flutter Mane", "FlutterMane");
    s = s.replace("Slither Wing", "SlitherWing");
    s = s.replace("Sandy Shocks", "SandyShocks");
    s = s.replace("Iron Treads", "IronTreads");
    s = s.replace("Iron Bundle", "IronBundle");
    s = s.replace("Iron Hands", "IronHands");
    s = s.replace("Iron Jugulis", "IronJugulis");
    s = s.replace("Iron Moth", "IronMoth");
    s = s.replace("Iron Thorns", "IronThorns");
    s = s.replace("Roaring Moon", "RoaringMoon");
    s = s.replace("Iron Valiant", "IronValiant");
    s = s.replace("Walking Wake", "WalkingWake");
    s = s.replace("Iron Leaves", "IronLeaves");
    s = s.replace("Gouging Fire", "GougingFire");
    s = s.replace("Raging Bolt", "RagingBolt");
    s = s.replace("Iron Boulder", "IronBoulder");
    s = s.replace("Iron Crown", "IronCrown");
    s = s.replace("Will-O-Wisp", "WillOWisp");
    s = s.replace("Flabébé", "Flabebe");
    s = s.replace([':', '.', '\'', '-', ' '], "");
    s = s.replace("♀", "F").replace("♂", "M");
    s.to_pascal_case()
}

fn map_type(t: &str) -> proc_macro2::TokenStream {
    let ident = format_ident!("{}", t);
    quote! { crate::core_data::Type::#ident }
}

fn map_category(c: &str) -> proc_macro2::TokenStream {
    let ident = format_ident!("{}", c);
    quote! { crate::core_data::MoveCategory::#ident }
}

fn map_target(t: &str) -> proc_macro2::TokenStream {
    let variant = match t {
        "normal" => "Normal",
        "self" => "Self_",
        "any" => "Any",
        "allAdjacent" => "AllAdjacent",
        "allAdjacentFoes" => "AllAdjacentFoes",
        "allySide" => "AllySide",
        "foeSide" => "FoeSide",
        "all" => "All",
        "randomNormal" => "RandomNormal",
        "scripted" => "Scripted",
        "allAllies" => "AllAllies",
        "allyTeam" => "AllyTeam",
        "adjacentAlly" => "Any",
        "adjacentAllyOrSelf" => "Any",
        "adjacentFoe" => "Any",
        _ => "Normal",
    };
    let ident = format_ident!("{}", variant);
    quote! { crate::core_data::MoveTarget::#ident }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_db.rs");

    // URLs
    let moves_url = "https://play.pokemonshowdown.com/data/moves.json";
    let pokedex_url = "https://play.pokemonshowdown.com/data/pokedex.json";

    // Fetch Data
    let moves_json: HashMap<String, MoveJson> = reqwest::blocking::get(moves_url)
        .expect("Failed to fetch moves")
        .json()
        .expect("Failed to parse moves json");

    let pokedex_json: HashMap<String, SpeciesJson> = reqwest::blocking::get(pokedex_url)
        .expect("Failed to fetch pokedex")
        .json()
        .expect("Failed to parse pokedex json");

    // Process Moves
    let mut moves: Vec<(String, MoveJson)> = moves_json.into_iter()
        .filter(|(_, m)| {
            m.num.unwrap_or(9999) <= 1000 &&
            m.is_max.is_none() &&
            m.is_z.is_none()
        })
        .collect();

    moves.sort_by(|a, b| {
        let num_a = a.1.num.unwrap_or(9999);
        let num_b = b.1.num.unwrap_or(9999);
        if num_a != num_b {
            num_a.cmp(&num_b)
        } else {
            a.0.cmp(&b.0)
        }
    });

    let mut move_variants = vec![quote! { None = 0 }];
    let mut move_data_entries = vec![quote! {
        crate::core_data::MoveData {
            name: "None",
            type_: crate::core_data::Type::Unknown,
            power: 0,
            accuracy: None,
            pp: 0,
            priority: 0,
            target: crate::core_data::MoveTarget::Normal,
            category: crate::core_data::MoveCategory::Status,
            flags: crate::core_data::MoveFlags::empty(),
        }
    }];

    for (i, (_, m)) in moves.iter().enumerate() {
        let id_str = sanitize(&m.name);
        if id_str.is_empty() { continue; }

        let id_ident = format_ident!("{}", id_str);
        let idx = (i + 1) as u16;

        move_variants.push(quote! { #id_ident = #idx });

        let name_str = &m.name;
        let type_ts = map_type(&m.type_);
        let power = m.base_power;
        let accuracy = match &m.accuracy {
            Some(Value::Bool(true)) => quote! { None },
            Some(Value::Number(n)) => {
                let val = n.as_u64().unwrap() as u8;
                quote! { Some(#val) }
            },
            _ => quote! { None },
        };
        let pp = m.pp;
        let priority = m.priority.unwrap_or(0);
        let target_ts = map_target(m.target.as_deref().unwrap_or("normal"));
        let category_ts = map_category(m.category.as_deref().unwrap_or("Status"));

        let mut flags_ts = vec![];
        if let Some(flags) = &m.flags {
            for (k, _) in flags {
                let flag_name = match k.as_str() {
                    "contact" => "CONTACT",
                    "protect" => "PROTECT",
                    "mirror" => "MIRROR",
                    "heal" => "HEAL",
                    "bypasssub" => "BYPASS_SUB",
                    "bite" => "BITE",
                    "punch" => "PUNCH",
                    "sound" => "SOUND",
                    "powder" => "POWDER",
                    "bullet" => "BULLET",
                    "pulse" => "PULSE",
                    "wind" => "WIND",
                    "slicing" => "SLICING",
                    "dance" => "DANCE",
                    "gravity" => "GRAVITY",
                    "defrost" => "DEFROST",
                    "distance" => "DISTANCE",
                    "charge" => "CHARGE",
                    "recharge" => "RECHARGE",
                    "nonsky" => "NONSKY",
                    "allyanim" => "ALLY_ANIM",
                    "noassist" => "NO_ASSIST",
                    "failcopycat" => "FAIL_COPYCAT",
                    "failencore" => "FAIL_ENCORE",
                    "failinstruct" => "FAIL_INSTRUCT",
                    "failmimic" => "FAIL_MIMIC",
                    "failsketch" => "FAIL_SKETCH",
                    "futuremove" => "FUTURE_MOVE",
                    "snatch" => "SNATCH",
                    "authentic" => "BYPASS_SUB",
                    _ => continue,
                };
                let flag_ident = format_ident!("{}", flag_name);
                flags_ts.push(quote! { crate::core_data::MoveFlags::#flag_ident });
            }
        }

        let flags_code = if flags_ts.is_empty() {
            quote! { crate::core_data::MoveFlags::empty() }
        } else {
            // Using from_bits_retain for const context compatibility
            let first = &flags_ts[0];
            let rest = &flags_ts[1..];
            if rest.is_empty() {
                quote! { #first }
            } else {
                quote! {
                    crate::core_data::MoveFlags::from_bits_retain(
                        #first.bits() #(| #rest.bits())*
                    )
                }
            }
        };

        move_data_entries.push(quote! {
            crate::core_data::MoveData {
                name: #name_str,
                type_: #type_ts,
                power: #power,
                accuracy: #accuracy,
                pp: #pp,
                priority: #priority,
                target: #target_ts,
                category: #category_ts,
                flags: #flags_code,
            }
        });
    }

    let move_count = move_data_entries.len();

    // Process Species
    let mut species: Vec<(String, SpeciesJson)> = pokedex_json.into_iter()
        .filter(|(_, s)| {
            s.num.unwrap_or(0) > 0 &&
            s.types.is_some() &&
            s.base_stats.is_some() &&
            s.abilities.is_some() &&
            s.weightkg.is_some()
        })
        .collect();

    species.sort_by(|a, b| {
        let num_a = a.1.num.unwrap_or(0);
        let num_b = b.1.num.unwrap_or(0);
        if num_a != num_b {
            num_a.cmp(&num_b)
        } else {
            a.0.cmp(&b.0)
        }
    });

    let mut species_variants = vec![quote! { None = 0 }];
    let mut species_data_entries = vec![quote! {
        crate::core_data::SpeciesData {
            name: "None",
            types: [crate::core_data::Type::Unknown, crate::core_data::Type::Unknown],
            base_stats: crate::core_data::Stats { hp: 0, atk: 0, def: 0, spa: 0, spd: 0, spe: 0 },
            abilities: &[],
            weight_kg: 0.0,
        }
    }];

    for (i, (_, s)) in species.iter().enumerate() {
        let id_str = sanitize(&s.name);
        if id_str.is_empty() { continue; }

        let id_ident = format_ident!("{}", id_str);
        let idx = (i + 1) as u16;

        species_variants.push(quote! { #id_ident = #idx });

        let name_str = &s.name;
        let types = s.types.as_ref().unwrap();
        let t1 = map_type(&types[0]);
        let t2 = if types.len() > 1 {
            map_type(&types[1])
        } else {
            quote! { crate::core_data::Type::Unknown }
        };

        let bs = s.base_stats.as_ref().unwrap();
        let hp = bs.get("hp").unwrap_or(&0);
        let atk = bs.get("atk").unwrap_or(&0);
        let def = bs.get("def").unwrap_or(&0);
        let spa = bs.get("spa").unwrap_or(&0);
        let spd = bs.get("spd").unwrap_or(&0);
        let spe = bs.get("spe").unwrap_or(&0);

        // Abilities
        let mut abilities = vec![];
        let mut abil_values: Vec<&String> = s.abilities.as_ref().unwrap().values().collect();
        abil_values.sort();
        for abil in abil_values {
            abilities.push(quote! { #abil });
        }

        let weight = s.weightkg.unwrap();

        species_data_entries.push(quote! {
            crate::core_data::SpeciesData {
                name: #name_str,
                types: [#t1, #t2],
                base_stats: crate::core_data::Stats {
                    hp: #hp, atk: #atk, def: #def, spa: #spa, spd: #spd, spe: #spe
                },
                abilities: &[#(#abilities),*],
                weight_kg: #weight,
            }
        });
    }

    let species_count = species_data_entries.len();

    let type_chart_rows: Vec<proc_macro2::TokenStream> = TYPE_CHART_RAW.iter().map(|row| {
        quote! { [#(#row),*] }
    }).collect();

    let output = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum MoveId {
            #(#move_variants),*
        }

        pub const MOVES: [crate::core_data::MoveData; #move_count] = [
            #(#move_data_entries),*
        ];

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum SpeciesId {
            #(#species_variants),*
        }

        pub const SPECIES: [crate::core_data::SpeciesData; #species_count] = [
            #(#species_data_entries),*
        ];

        pub const TYPE_CHART: [[f32; 19]; 19] = [
            #(#type_chart_rows),*
        ];
    };

    fs::write(dest_path, output.to_string()).unwrap();
}
