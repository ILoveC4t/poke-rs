//! Code generation helpers for poke_engine.
//!
//! This crate parses JSON data files from smogon/pokemon-showdown
//! and generates optimized Rust types for the battle engine.

mod abilities;
mod helpers;
mod items;
mod models;
mod moves;
mod natures;
mod species;
mod terrains;
mod types;

use std::path::Path;
use std::println;

/// Generate all code from the data directory into the output directory.
///
/// This is the main entry point called from poke_engine's build.rs.
pub fn generate_all(out_dir: &Path, data_dir: &Path) {
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

    // Generate each module
    types::generate(out_dir, data_dir);
    natures::generate(out_dir, data_dir);
    abilities::generate(out_dir, data_dir);
    species::generate(out_dir, data_dir);
    moves::generate(out_dir, data_dir);
    items::generate(out_dir, data_dir);
    terrains::generate(out_dir, data_dir);
}
