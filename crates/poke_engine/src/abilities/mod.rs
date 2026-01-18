//! Ability system hooks and registry.

// Include generated ability identifiers
include!(concat!(env!("OUT_DIR"), "/abilities.rs"));

pub mod hooks;
pub mod registry;
pub mod weather;
pub mod implementations;

pub use hooks::AbilityHooks;
pub use registry::ABILITY_REGISTRY;
pub use weather::{Weather, Terrain};

#[cfg(test)]
mod tests;
