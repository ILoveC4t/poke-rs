//! Ability system hooks and registry.

// Include generated ability identifiers
include!(concat!(env!("OUT_DIR"), "/abilities.rs"));

pub mod hooks;
pub mod implementations;
pub mod registry;
pub mod weather;

pub use hooks::AbilityHooks;
pub use registry::ABILITY_REGISTRY;
pub use weather::{Terrain, Weather};

#[cfg(test)]
mod tests;
