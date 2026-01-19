//! Build script for poke_engine.
//!
//! Calls out to poke_engine_codegen to generate Rust types from JSON data.

use std::path::Path;
use std::{env, println};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = Path::new(&manifest_dir).join("../../data");

    // Additional rerun triggers for build.rs itself
    println!("cargo:rerun-if-changed=build.rs");

    poke_engine_codegen::generate_all(Path::new(&out_dir), &data_dir);
}
